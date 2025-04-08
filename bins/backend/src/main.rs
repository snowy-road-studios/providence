use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use bevy::prelude::*;
use bevy_girk_backend_public::*;
use bevy_girk_game_hub_server::*;
use bevy_girk_game_instance::*;
use bevy_girk_host_server::*;
use bevy_girk_utils::*;
use clap::Parser;
use enfync::AdoptOrDefault;
use renet2_setup::GameServerSetupConfig;
use utils::RootConfigs;
use wiring_backend::*;
use wiring_game_instance::*;

//-------------------------------------------------------------------------------------------------------------------

fn make_host_server_configs(configs: &RootConfigs) -> Result<HostServerStartupPack, String>
{
    // configs
    let host_server_config = HostServerConfig {
        ticks_per_sec: Some(configs.get_integer("host_backend", "TICKS_PER_SEC")?),
        ongoing_game_purge_period_ticks: configs
            .get_integer("host_backend", "ONGOING_GAMES_PURGE_PERIOD_TICKS")?,
    };
    let lobbies_cache_config = LobbiesCacheConfig {
        max_request_size: configs.get_integer("host_frontend", "LOBBY_LIST_SIZE")?,
        lobby_checker: Box::new(ProvLobbyChecker {
            max_lobby_players: configs.get_integer("lobby", "MAX_LOBBY_PLAYERS")?,
            min_players_to_launch: configs.get_integer("lobby", "MIN_PLAYERS_TO_LAUNCH")?,
        }),
    };
    let pending_lobbies_cache_config = PendingLobbiesConfig {
        ack_timeout: Duration::from_millis(configs.get_integer("host_frontend", "ACK_TIMEOUT_MILLIS")?),
        start_buffer: Duration::from_secs(configs.get_integer("host_backend", "GAME_START_ALLOWED_DELAY_SECS")?),
    };
    let ongoing_games_cache_config = OngoingGamesCacheConfig {
        expiry_duration: Duration::from_secs(configs.get_integer("host_backend", "ONGOING_GAME_EXPIRY_SECS")?),
    };
    let game_hub_disconnect_buffer_config = GameHubDisconnectBufferConfig {
        expiry_duration: Duration::from_secs(configs.get_integer("host_backend", "GAME_HUB_DC_EXPRIY_SECS")?),
    };

    Ok(HostServerStartupPack {
        host_server_config,
        lobbies_cache_config,
        pending_lobbies_cache_config,
        ongoing_games_cache_config,
        game_hub_disconnect_buffer_config,
    })
}

//-------------------------------------------------------------------------------------------------------------------

fn make_hub_server_configs(configs: &RootConfigs) -> Result<GameHubServerStartupPack, String>
{
    let game_hub_server_config = GameHubServerConfig {
        ticks_per_sec: Some(configs.get_integer("game_hub", "TICKS_PER_SEC")?),
        initial_max_capacity: configs.get_integer("game_hub", "INITIAL_MAX_CAPACITY")?,
        running_game_purge_period_ticks: configs.get_integer("game_hub", "RUNNING_GAME_PURGE_PERIOD_TICKS")?,
    };
    let pending_games_cache_config = PendingGamesCacheConfig {
        expiry_duration: Duration::from_secs(configs.get_integer("game_hub", "PENDING_GAME_EXPIRY_SECS")?),
    };
    let running_games_cache_config = RunningGamesCacheConfig {
        expiry_duration: Duration::from_secs(configs.get_integer("game_hub", "RUNNING_GAME_EXPIRY_SECS")?),
    };

    Ok(GameHubServerStartupPack {
        game_hub_server_config,
        pending_games_cache_config,
        running_games_cache_config,
    })
}

//-------------------------------------------------------------------------------------------------------------------

fn make_test_host_server(
    host_addr: String,
    rustls_config: Option<Arc<rustls::ServerConfig>>,
    configs: HostServerStartupPack,
) -> (App, url::Url, url::Url)
{
    // host-user server
    // - Make this first so host-hub server's wildcard port doesn't steal the server's pre-specified port.
    let acceptor = match rustls_config {
        Some(rustls_config) => bevy_simplenet::AcceptorConfig::Rustls(
            axum_server::tls_rustls::RustlsConfig::from_config(rustls_config),
        ),
        None => bevy_simplenet::AcceptorConfig::Default,
    };
    let host_user_server = host_user_server_factory().new_server(
        enfync::builtin::native::TokioHandle::adopt_or_default(),
        host_addr,
        acceptor,
        bevy_simplenet::Authenticator::None,
        bevy_simplenet::ServerConfig::default(),
    );
    let host_user_url = host_user_server.url();

    // host-hub server
    let host_hub_server = host_hub_server_factory().new_server(
        enfync::builtin::native::TokioHandle::adopt_or_default(),
        "127.0.0.1:0",
        bevy_simplenet::AcceptorConfig::Default,
        bevy_simplenet::Authenticator::None,
        bevy_simplenet::ServerConfig::default(),
    );
    let hub_server_url = host_hub_server.url();

    (
        make_host_server(configs, host_hub_server, host_user_server),
        hub_server_url,
        host_user_url,
    )
}

//-------------------------------------------------------------------------------------------------------------------

fn make_test_host_hub_client_with_id(client_id: u128, hub_server_url: url::Url) -> HostHubClient
{
    host_hub_client_factory().new_client(
        enfync::builtin::native::TokioHandle::adopt_or_default(),
        hub_server_url,
        bevy_simplenet::AuthRequest::None { client_id },
        bevy_simplenet::ClientConfig::default(),
        (),
    )
}

//-------------------------------------------------------------------------------------------------------------------

fn make_test_game_hub_server(
    game_instance_path: String,
    hub_server_url: url::Url,
    startup_pack: GameHubServerStartupPack,
    game_factory_config: ProvGameFactoryConfig,
) -> (Sender<GameHubCommand>, App)
{
    // setup
    let (command_sender, command_receiver) = new_channel::<GameHubCommand>();
    let host_hub_client = make_test_host_hub_client_with_id(0u128, hub_server_url);
    let game_launch_pack_source = GameLaunchPackSource::new(ProvGameLaunchPackSource::new(game_factory_config));
    let game_launcher = GameInstanceLauncher::new(GameInstanceLauncherProcess::new(
        game_instance_path,
        enfync::builtin::native::TokioHandle::adopt_or_default(),
    ));

    // server app
    let server_app = make_game_hub_server(
        startup_pack,
        command_receiver,
        host_hub_client,
        game_launch_pack_source,
        game_launcher,
    );

    (command_sender, server_app)
}

//-------------------------------------------------------------------------------------------------------------------

fn config_paths() -> Vec<PathBuf>
{
    let mut paths = Vec::default();
    paths.push("/backend/game_hub.toml".into());
    paths.push("/backend/host_backend.toml".into());
    paths.push("/frontend/host_frontend.toml".into());
    paths.push("/frontend/lobby.toml".into());
    paths.push("/game/game.toml".into());

    paths
}

//-------------------------------------------------------------------------------------------------------------------

//todo: include log level
#[derive(Parser, Debug)]
struct BackendCli
{
    /// Specify the location of the game instance binary (will use the debug build directory by default).
    /// Requires '--game-instance'.
    #[arg(long)]
    game_instance: Option<String>,
    /// Specify the directory where config files are stored.
    #[arg(long)]
    config_dir: Option<String>,
    /// Address of user-host server.
    #[arg(long)]
    host_addr: Option<String>,
    /// Local IP for game servers.
    #[arg(long)]
    local_ip: Option<IpAddr>,
    /// Proxy IP for game servers.
    #[arg(long)]
    proxy_ip: Option<IpAddr>,
    /// Domain name for websocket game servers.
    #[arg(long)]
    ws_domain: Option<String>,
    /// File locations of tls certificates for websockets. See GameServerSetupConfig.
    ///
    /// Cert chain for websocket certs, should be `PEM` encoded.
    #[arg(long)]
    wss_certs: Option<String>,
    /// Privkey for websocket certs, should be `PEM` encoded.
    #[arg(long)]
    wss_certs_privkey: Option<String>,
}

impl BackendCli
{
    fn extract(self) -> BackendCliResolved
    {
        let game_instance_path = self
            .game_instance
            .unwrap_or_else(|| DEFAULT_GAME_INSTANCE_PATH.into());
        let config_dir: PathBuf = self
            .config_dir
            .unwrap_or_else(|| DEFAULT_CONFIG_DIR.into())
            .into();
        let host_addr = self.host_addr.unwrap_or_else(|| "127.0.0.1:48888".into());

        let wss_certs = match (self.wss_certs, self.wss_certs_privkey) {
            (Some(certs), Some(privkey)) => Some((PathBuf::from(certs), PathBuf::from(privkey))),
            (None, None) => None,
            (Some(_), None) => {
                tracing::error!("wss_certs arg found but wss_certs_privkey is missing");
                None
            }
            (None, Some(_)) => {
                tracing::error!("wss_certs_privkey arg found but wss_certs is missing");
                None
            }
        };

        BackendCliResolved {
            game_instance_path,
            config_dir,
            host_addr,
            local_ip: self.local_ip,
            proxy_ip: self.proxy_ip,
            ws_domain: self.ws_domain,
            wss_certs,
        }
    }
}

struct BackendCliResolved
{
    game_instance_path: String,
    config_dir: PathBuf,
    host_addr: String,
    local_ip: Option<IpAddr>,
    proxy_ip: Option<IpAddr>,
    ws_domain: Option<String>,
    wss_certs: Option<(PathBuf, PathBuf)>,
}

//-------------------------------------------------------------------------------------------------------------------

const DEFAULT_GAME_INSTANCE_PATH: &'static str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/debug/game_instance");
const DEFAULT_CONFIG_DIR: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../config");
#[cfg(feature = "dev")]
const CONFIGS_OVERRIDE_DIR: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/config");

//-------------------------------------------------------------------------------------------------------------------

fn main()
{
    // logging
    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
        .from_env()
        .unwrap()
        .add_directive("bevy_simplenet=trace".parse().unwrap())
        .add_directive("renet2=info".parse().unwrap())
        .add_directive("renet2_netcode=info".parse().unwrap())
        .add_directive("renetcode2=info".parse().unwrap())
        .add_directive("bevy_girk_host_server=trace".parse().unwrap())
        .add_directive("bevy_girk_game_hub_server=trace".parse().unwrap())
        .add_directive("bevy_girk_wiring=trace".parse().unwrap())
        .add_directive("game_core=trace".parse().unwrap())
        .add_directive("bevy_girk_game_fw=trace".parse().unwrap());
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("launching backend: {:?}", std::env::args_os());

    // env
    let args = BackendCli::parse();
    tracing::info!(?args);
    let args = args.extract();

    // prep rustls
    let maybe_rustls = if let Some((certs, privkey)) = &args.wss_certs {
        GameServerSetupConfig::get_rustls_server_config(certs, privkey).ok()
    } else {
        None
    };

    // extract configs
    let mut configs = RootConfigs::default();
    configs.read(args.config_dir, config_paths()).unwrap();
    #[cfg(feature = "dev")]
    configs
        .read(CONFIGS_OVERRIDE_DIR.into(), config_paths())
        .unwrap();

    // launch host server
    let (mut host_server, hub_server_url, host_user_url) = make_test_host_server(
        args.host_addr,
        maybe_rustls,
        make_host_server_configs(&configs).unwrap(),
    );
    tracing::info!("host-user server running at {}", host_user_url.as_str());

    // run the servers
    std::thread::spawn(move || {
        // launch game hub server attached to host server
        let startup_pack = make_hub_server_configs(&configs).unwrap();
        let game_factory_config =
            make_prov_game_configs(args.local_ip, args.proxy_ip, args.ws_domain, args.wss_certs, &configs)
                .unwrap();
        let (_hub_command_sender, mut hub_server) = make_test_game_hub_server(
            args.game_instance_path,
            hub_server_url,
            startup_pack,
            game_factory_config,
        );
        hub_server.run()
    });
    host_server.run();
}

//-------------------------------------------------------------------------------------------------------------------
