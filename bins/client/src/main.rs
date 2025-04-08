use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use bevy::prelude::*;
use bevy_girk_backend_public::*;
use bevy_girk_client_instance::*;
use bevy_girk_game_instance::GameFactory;
use bevy_girk_utils::Rand64;
use clap::Parser;
use user_client::*;
use utils::RootConfigs;
use wasm_timer::{SystemTime, UNIX_EPOCH};
use wiring_client_instance::ProvClientFactory;
use wiring_game_instance::ProvGameFactory;

//-------------------------------------------------------------------------------------------------------------------

fn timer_configs(configs: &RootConfigs) -> Result<TimerConfigs, String>
{
    Ok(TimerConfigs {
        host_reconstruct_loop_ms: configs.get_integer("user_client", "HOST_RECONSTRUCT_LOOP_MILLIS")?,
        token_request_loop_ms: configs.get_integer("user_client", "TOKEN_REQUEST_LOOP_MILLIS")?,
        ack_request_timeout_ms: configs.get_integer::<u64>("host_frontend", "ACK_TIMEOUT_MILLIS")? + 1_000,
        ack_request_timer_buffer_ms: configs.get_integer("user_client", "ACK_TIMER_BUFFER_MILLIS")?,
        lobby_list_refresh_ms: configs.get_integer("user_client", "LOBBY_LIST_REFRESH_MILLIS")?,
    })
}

//-------------------------------------------------------------------------------------------------------------------

fn config_paths() -> Vec<PathBuf>
{
    let mut paths = Vec::default();
    paths.push("/frontend/host_frontend.toml".into());
    paths.push("/frontend/lobby.toml".into());
    paths.push("/game/game.toml".into());
    paths.push("/game_client/game_client.toml".into());
    paths.push("/user_client/user_client.toml".into());

    paths
}

//-------------------------------------------------------------------------------------------------------------------

//todo: specify app data file path (e.g. contains auth keys [temp solution before 'login'-style auth], logs,
// settings)
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct ClientCli
{
    /// The location of the configs directory on native clients.
    ///
    /// Alt: PROV_CONFIG_DIR env variable (required for WASM clients)
    #[arg(long)]
    config_dir: Option<String>,
    /// Specify the client id (will be random if unspecified).
    #[arg(long = "id")]
    client_id: Option<u128>,
    /// Alt: PROV_HOST_ADDR env variable (required for WASM clients)
    #[arg(long = "addr")]
    server_addr: Option<String>,
    /// Alt: PROV_HOST_IS_WSS env variable (required for WASM clients)
    #[arg(long)]
    host_is_wss: Option<bool>,
}

impl ClientCli
{
    fn extract(self) -> ClientCliResolved
    {
        let config_dir: PathBuf = self
            .config_dir
            .or_else(|| std::option_env!("PROV_CONFIG_DIR").map(|s| s.into()))
            .unwrap_or_else(|| DEFAULT_CONFIG_DIR.into())
            .into();
        //TODO: obtain client id from backend on login
        let client_id = self.client_id.unwrap_or_else(get_systime_millis);
        //TODO: obtain server addr from backend on login
        let server_addr = self
            .server_addr
            .or_else(|| std::option_env!("PROV_HOST_ADDR").map(|s| s.into()))
            .unwrap_or_else(|| "127.0.0.1:48888".into());
        let host_is_wss = self
            .host_is_wss
            .or_else(|| std::option_env!("PROV_HOST_IS_WSS").map(|s| bool::from_str(s).unwrap_or_default()))
            .unwrap_or_default();

        ClientCliResolved { config_dir, client_id, server_addr, host_is_wss }
    }
}

struct ClientCliResolved
{
    config_dir: PathBuf,
    client_id: u128,
    server_addr: String,
    host_is_wss: bool,
}

//-------------------------------------------------------------------------------------------------------------------

fn get_systime_millis() -> u128
{
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

//-------------------------------------------------------------------------------------------------------------------

const DEFAULT_CONFIG_DIR: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../config");
#[cfg(feature = "dev")]
const CONFIGS_OVERRIDE_DIR: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/config");

//-------------------------------------------------------------------------------------------------------------------

#[cfg(target_family = "wasm")]
#[wasm_bindgen(main)]
async fn main()
{
    // setup wasm tracing
    console_error_panic_hook::set_once();
    //tracing_wasm::set_as_global_default();

    // cli args
    let args = ClientCli::parse();
    tracing::trace!(?args);
    let args = args.extract();

    //TODO: use hasher directly?
    let protocol_id = Rand64::new(env!("CARGO_PKG_VERSION"), 0u128).next();

    // make URL
    let host = if args.host_is_wss { "wss" } else { "ws" };
    let url = format!("{host}://{}/ws", args.server_addr.as_str());
    tracing::info!("connecting to host server: {}", url.as_str());

    // extract configs
    let mut configs = RootConfigs::default();
    configs.read(args.config_dir, config_paths()).unwrap();
    #[cfg(feature = "dev")]
    configs
        .read(CONFIGS_OVERRIDE_DIR.into(), config_paths())
        .unwrap();

    // prep to launch client
    // - todo: receive URL from HTTP(s) server, and load the HTTP(s) URL from an asset
    let client_id = args.client_id;
    let make_client = move || {
        host_user_client_factory().new_client(
            enfync::builtin::Handle::default(), // automatically selects native/WASM runtime
            // TODO: use auth server to get this?
            url::Url::parse(url.as_str()).unwrap(),
            // TODO: use auth tokens and an auth server
            bevy_simplenet::AuthRequest::None { client_id },
            bevy_simplenet::ClientConfig::default(),
            // auto-detects connection type for games (udp/webtransport/websockets)
            HostUserConnectMsg::new(),
        )
    };

    // timer configs for the user client (TEMPORARY: use asset instead ?)
    let timer_configs = timer_configs(&configs).unwrap();

    // factory for local-player games
    let game_factory = GameFactory::new(ProvGameFactory);

    // client factory for setting up games
    let factory = ProvClientFactory {
        protocol_id,
        resend_time: Duration::from_millis(
            configs
                .get_integer("game_client", "RENET2_RESEND_TIME_MILLIS")
                .unwrap(),
        ),
    };

    // build and launch the bevy app
    App::new()
        .add_plugins(ClientInstancePlugin::new(factory, Some(game_factory)))
        .insert_resource(HostClientConstructor::new(make_client))
        .insert_resource(timer_configs)
        .insert_resource(configs)
        .add_plugins(ProvUserClientPlugin)
        .run();
}

//-------------------------------------------------------------------------------------------------------------------

#[cfg(not(target_family = "wasm"))]
fn main()
{
    // log to stdout
    //todo: log to file?
    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
        .from_env()
        .unwrap()
        .add_directive("ezsockets=debug".parse().unwrap())
        .add_directive("bevy_girk_game_instance=trace".parse().unwrap())
        .add_directive("client_core=trace".parse().unwrap())
        .add_directive("user_client=trace".parse().unwrap())
        .add_directive("bevy_girk_wiring=trace".parse().unwrap())
        .add_directive("bevy_girk_utils=trace".parse().unwrap())
        .add_directive("bevy_simplenet=debug".parse().unwrap())
        .add_directive("renet2=info".parse().unwrap())
        .add_directive("renet2_netcode=info".parse().unwrap())
        .add_directive("renetcode2=info".parse().unwrap());
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(filter)
        .with_writer(std::io::stdout)
        .init();

    // cli args
    let args = ClientCli::parse();
    tracing::trace!(?args);
    let args = args.extract();

    // set asset directory location
    if let Err(err) = bevy_girk_utils::try_set_bevy_asset_root(2) {
        panic!("Could not set bevy asset root: {}", err.to_string());
    }

    // setup crypto for bevy_simplenet websocket connection
    if args.host_is_wss && rustls::crypto::CryptoProvider::get_default().is_none() {
        let _ = rustls::crypto::ring::default_provider().install_default();
    }

    //TODO: use hasher directly?
    let protocol_id = Rand64::new(env!("CARGO_PKG_VERSION"), 0u128).next();

    // make URL
    let host = if args.host_is_wss { "wss" } else { "ws" };
    let url = format!("{host}://{}/ws", args.server_addr.as_str());
    tracing::info!("connecting to host server: {}", url.as_str());

    // extract configs
    let mut configs = RootConfigs::default();
    configs.read(args.config_dir, config_paths()).unwrap();
    #[cfg(feature = "dev")]
    configs
        .read(CONFIGS_OVERRIDE_DIR.into(), config_paths())
        .unwrap();

    // prep to launch client
    // - todo: receive URL from HTTP(s) server, and load the HTTP(s) URL from an asset
    let client_id = args.client_id;
    let make_client = move || {
        host_user_client_factory().new_client(
            enfync::builtin::Handle::default(), // automatically selects native/WASM runtime
            // TODO: use auth server to get this?
            url::Url::parse(url.as_str()).unwrap(),
            // TODO: use auth tokens and an auth server
            bevy_simplenet::AuthRequest::None { client_id },
            bevy_simplenet::ClientConfig::default(),
            // auto-detects connection type for games (udp/webtransport/websockets)
            HostUserConnectMsg::new(),
        )
    };

    // timer configs for the user client (TEMPORARY: use asset instead ?)
    let timer_configs = timer_configs(&configs).unwrap();

    // factory for local-player games
    let game_factory = GameFactory::new(ProvGameFactory);

    // client factory for setting up games
    let factory = ProvClientFactory {
        protocol_id,
        resend_time: Duration::from_millis(
            configs
                .get_integer("game_client", "RENET2_RESEND_TIME_MILLIS")
                .unwrap(),
        ),
    };

    // build and launch the bevy app
    App::new()
        .add_plugins(ClientInstancePlugin::new(factory, Some(game_factory)))
        .insert_resource(HostClientConstructor::new(make_client))
        .insert_resource(timer_configs)
        .insert_resource(configs)
        .add_plugins(ProvUserClientPlugin)
        .run();
}

//-------------------------------------------------------------------------------------------------------------------
