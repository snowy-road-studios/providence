use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bevy_girk_game_instance::*;
use bevy_girk_utils::*;
use clap::Parser;
use enfync::{AdoptOrDefault, Handle};
use game_core::GameData;
use renet2_setup::ConnectionType;
use utils::RootConfigs;
use wiring_backend::*;
use wiring_game_instance::*;

//-------------------------------------------------------------------------------------------------------------------

fn config_paths() -> Vec<PathBuf>
{
    let mut paths = Vec::default();
    paths.push("game/game.toml".into());
    paths.push("game/hq.toml".into());
    paths.push("game_client/game_client.toml".into());

    paths
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Parser, Debug)]
struct PlaytestCli
{
    /// Specify the number of clients (defaults to 1, minimum is 1).
    #[arg(long)]
    clients: Option<usize>,
    /// Specify the location of the game instance binary (will use the debug build directory by default).
    game: Option<String>,
    /// Specify the location of the game client binary (will use the debug build directory by default).
    client: Option<String>,
}

impl PlaytestCli
{
    fn extract(self) -> PlaytestCliResolved
    {
        let num_clients = self.clients.unwrap_or(1usize).max(1usize);
        let game_instance_path = self
            .game
            .unwrap_or_else(|| String::from(GAME_INSTANCE_PATH));
        let game_client_path = self
            .client
            .unwrap_or_else(|| String::from(GAME_CLIENT_PATH));

        PlaytestCliResolved { num_clients, game_instance_path, game_client_path }
    }
}

struct PlaytestCliResolved
{
    num_clients: usize,
    game_instance_path: String,
    game_client_path: String,
}

//-------------------------------------------------------------------------------------------------------------------

fn get_systime() -> Duration
{
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
}

//-------------------------------------------------------------------------------------------------------------------

fn run_playtest(
    launch_pack: GameLaunchPack,
    game_instance_path: String,
    game_client_path: String,
    configs: &RootConfigs,
)
{
    let spawner = enfync::builtin::native::TokioHandle::adopt_or_default();

    let renet2_client_resend_time: u64 = configs
        .get_integer("game_client", "RENET2_RESEND_TIME_MILLIS")
        .unwrap();
    let game_data = GameData::new(configs).unwrap();

    // launch game
    tracing::trace!("launching game instance for playtest");
    let (game_report_sender, mut game_report_receiver) = new_io_channel::<GameInstanceReport>();
    let game_launcher = GameInstanceLauncherProcess::new(game_instance_path, spawner.clone());
    let mut game_instance = game_launcher.launch(launch_pack, game_report_sender);

    // launch in task
    let task = spawner.spawn(async move {
        // wait for game start report
        let Some(GameInstanceReport::GameStart(game_id, report)) = game_report_receiver.recv().await else {
            tracing::error!("failed getting game start report for playtest");
            return;
        };

        // prepare to launch the clients
        let Some(meta) = &report.metas.native else {
            tracing::error!("missing native meta for setting up playtest renet client");
            return;
        };

        // launch game clients
        let mut client_processes = Vec::default();
        for start_info in report.start_infos {
            let token = match meta.new_connect_token(get_systime(), start_info.client_id) {
                Ok(token) => token,
                Err(err) => {
                    tracing::error!("failed producing connect token for playtest: {err:?}");
                    continue;
                }
            };

            let Ok(token_ser) = serde_json::to_string(&token) else {
                tracing::error!(game_id, "failed serializing server connect token for playtest game client");
                continue;
            };

            let Ok(start_info_ser) = serde_json::to_string(&start_info) else {
                tracing::error!(game_id, "failed serializing game start info for playtest game client");
                continue;
            };

            let Ok(resend_time_ser) = serde_json::to_string(&renet2_client_resend_time) else {
                tracing::error!(game_id, "failed serializing renet2 resend time for playtest game client");
                continue;
            };

            let Ok(game_data_ser) = serde_json::to_string(&game_data) else {
                tracing::error!(game_id, "failed serializing renet2 game data for playtest game client");
                continue;
            };

            tracing::trace!(start_info.client_id, "launching game client for playtest");

            let Ok(child_process) = tokio::process::Command::new(&game_client_path)
                .args(["-T", &token_ser, "-S", &start_info_ser, "-R", &resend_time_ser, "-D", &game_data_ser])
                .spawn()
            else {
                tracing::error!("failed launching game client for playtest at {:?}", game_client_path);
                continue;
            };

            client_processes.push(child_process);
        }

        // wait for clients to close
        // - we must wait for client closure to avoid zombie process leak
        for mut client_process in client_processes {
            if client_process.wait().await.is_err() {
                tracing::warn!("playtest client instance closed with error");
                let _ = client_process.kill().await;
            }
        }

        // command game instance to abort
        // - we assume if the clients are closed then the game should die
        // - this will do nothing if the game instance already closed
        let _ = game_instance.send_command(GameInstanceCommand::Abort);

        // wait for game instance to close
        if !game_instance.get().await {
            tracing::warn!("playtest instance closed with error");
        }

        // get game instance report
        let Some(GameInstanceReport::GameOver(_, _game_over_report)) = game_report_receiver.recv().await else {
            tracing::error!("did not receive game over report for playtest");
            return;
        };
    });

    let _ = enfync::blocking::extract(task);
}

//-------------------------------------------------------------------------------------------------------------------

const DEFAULT_CONFIG_DIR: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../config");
#[cfg(feature = "dev")]
const CONFIGS_OVERRIDE_DIR: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/config");
const GAME_INSTANCE_PATH: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/debug/game_instance");
const GAME_CLIENT_PATH: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/debug/game_client");

//-------------------------------------------------------------------------------------------------------------------

fn main()
{
    // logging
    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing_subscriber::filter::LevelFilter::TRACE.into())
        .from_env()
        .unwrap();
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();

    // set asset directory location
    #[cfg(not(target_family = "wasm"))]
    {
        if let Err(err) = bevy_girk_utils::try_set_bevy_asset_root(2) {
            panic!("Could not set bevy asset root: {}", err.to_string());
        }
    }

    // env
    let args = PlaytestCli::parse();
    tracing::trace!(?args);
    let args = args.extract();

    // configs
    let mut configs = RootConfigs::default();
    configs
        .read(DEFAULT_CONFIG_DIR.into(), config_paths())
        .unwrap();
    #[cfg(feature = "dev")]
    configs
        .read(CONFIGS_OVERRIDE_DIR.into(), config_paths())
        .unwrap();

    // lobby contents
    let mut players = Vec::default();
    for idx in 0..args.num_clients {
        players.push((ConnectionType::Native, idx as u128));
    }

    let lobby_contents = ProvLobbyContents {
        id: 0u64,
        owner_id: 0u128,
        config: ProvLobbyConfig { max_players: args.num_clients as u16 },
        players,
    };

    // launch pack
    let game_configs = make_prov_game_configs(None, None, None, None, &configs).unwrap();
    let Ok(launch_pack) = get_launch_pack(game_configs, lobby_contents) else {
        tracing::error!("failed getting launch pack for playtest");
        return;
    };

    // run it
    run_playtest(launch_pack, args.game_instance_path, args.game_client_path, &configs);
}

//-------------------------------------------------------------------------------------------------------------------
