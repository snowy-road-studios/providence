//! Independent client binary. Can be used to launch games directly from another binary without an intermediating
//! user client.

use std::path::PathBuf;

use bevy::prelude::*;
use bevy_cobweb_ui::prelude::LoadState;
use bevy_girk_client_fw::ClientAppState;
use bevy_girk_client_instance::*;
use bevy_girk_game_instance::GameStartInfo;
use bevy_girk_utils::*;
use clap::Parser;
use renet2_setup::ServerConnectToken;
use utils::RootConfigs;
use wiring_client_instance::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Parser, Debug)]
struct GameClientCli
{
    /// ServerConnectToken
    #[arg(short = 'T', value_parser = parse_json::<ServerConnectToken>)]
    token: Option<ServerConnectToken>,
    /// GameStartInfo
    #[arg(short = 'S', value_parser = parse_json::<GameStartInfo>)]
    start_info: Option<GameStartInfo>,
    /// Location of config files.
    #[arg(short = 'C', value_parser = parse_json::<String>)]
    config_dir: Option<String>,
}

impl GameClientCli
{
    fn extract(self) -> GameClientCliResolved
    {
        let config_dir: PathBuf = self
            .config_dir
            .or_else(|| std::option_env!("PROV_CONFIG_DIR").map(|s| s.into()))
            .unwrap_or_else(|| DEFAULT_CONFIG_DIR.into())
            .into();

        GameClientCliResolved { token: self.token, start_info: self.start_info, config_dir }
    }
}

struct GameClientCliResolved
{
    token: Option<ServerConnectToken>,
    start_info: Option<GameStartInfo>,
    config_dir: PathBuf,
}

//-------------------------------------------------------------------------------------------------------------------

fn start_game_system(
    mut token: Option<ServerConnectToken>,
    mut start_info: Option<GameStartInfo>,
) -> impl IntoSystem<(), (), ()>
{
    IntoSystem::into_system(move |mut c: Commands| {
        let Some(token) = token.take() else { return };
        let Some(start_info) = start_info.take() else { return };
        c.queue(ClientInstanceCommand::Start(token, start_info));
    })
}

//-------------------------------------------------------------------------------------------------------------------

const DEFAULT_CONFIG_DIR: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../config");
const CONFIGS_OVERRIDE_DIR: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/config");

//-------------------------------------------------------------------------------------------------------------------

fn main()
{
    // log to stderr (not stdout, which is piped to the parent process for sending game instance reports)
    //todo: log to file instead (use CLI configs?)
    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing_subscriber::filter::LevelFilter::TRACE.into())
        .from_env()
        .unwrap()
        .add_directive("bevy=trace".parse().unwrap())
        .add_directive("bevy_cobweb=info".parse().unwrap())
        .add_directive("bevy_cobweb_ui=info".parse().unwrap())
        .add_directive("bevy_app=warn".parse().unwrap())
        .add_directive("bevy_core=warn".parse().unwrap())
        .add_directive("bevy_winit=warn".parse().unwrap())
        .add_directive("bevy_render=warn".parse().unwrap())
        .add_directive("blocking=warn".parse().unwrap())
        .add_directive("naga_oil=warn".parse().unwrap())
        .add_directive("bevy_replicon=info".parse().unwrap())
        .add_directive("winit=warn".parse().unwrap());
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    tracing::info!("game client started");

    // cli
    let args = GameClientCli::parse();
    let args = args.extract();

    // extract configs
    let sub_dirs = ["client", "game"];

    #[cfg(not(feature = "dev"))]
    let configs = RootConfigs::new(&args.config_dir, &sub_dirs).unwrap();
    #[cfg(feature = "dev")]
    let configs =
        RootConfigs::new_with_overrides(&args.config_dir, &CONFIGS_OVERRIDE_DIR.into(), &sub_dirs).unwrap();

    // make client factory
    let protocol_id = Rand64::new(env!("CARGO_PKG_VERSION"), 0u128).next();
    let client_factory = ProvClientFactory::new(protocol_id, &configs).unwrap();

    let mut app = App::new();
    app.add_plugins(ClientInstancePlugin::new(client_factory, None))
        // Can't do this in OnEnter because it internally forces a state transition. State transitions can't be
        // executed recursively.
        // Can't do this in Update otherwise we might skip past ClientInitState::InProgress.
        .add_systems(
            PreUpdate,
            start_game_system(args.token, args.start_info)
                .run_if(in_state(ClientAppState::Client))
                .run_if(in_state(LoadState::Done)),
        );

    #[cfg(feature = "egui")]
    {
        app.add_plugins(bevy_egui::EguiPlugin { enable_multipass_for_primary_context: true })
            .add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new());
    }

    app.run();
}

//-------------------------------------------------------------------------------------------------------------------
