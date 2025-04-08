use std::net::{IpAddr, Ipv6Addr};
use std::path::PathBuf;
use std::time::Duration;

use bevy_girk_game_fw::*;
use bevy_girk_utils::Rand64;
use game_core::*;
use renet2_setup::GameServerSetupConfig;
use serde::{Deserialize, Serialize};
use utils::RootConfigs;

//-------------------------------------------------------------------------------------------------------------------

/// Configuration for setting up a game with a game factory.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProvGameFactoryConfig
{
    pub server_setup_config: GameServerSetupConfig,
    pub game_fw_config: GameFwConfig,
    pub duration_config: GameDurationConfig,
    pub resend_time: Duration,
}

//-------------------------------------------------------------------------------------------------------------------

pub fn make_prov_game_configs(
    local_ip: Option<IpAddr>,
    proxy_ip: Option<IpAddr>,
    ws_domain: Option<String>,
    wss_certs: Option<(PathBuf, PathBuf)>,
    configs: &RootConfigs,
) -> Result<ProvGameFactoryConfig, String>
{
    // versioning
    //todo: use hasher directly?
    let protocol_id = Rand64::new(env!("CARGO_PKG_VERSION"), 0u128).next();

    // server setup config
    let server_setup_config = GameServerSetupConfig {
        protocol_id,
        expire_secs: configs.get_integer("game", "RENET2_CONNECTION_EXPIRY_SECS")?,
        timeout_secs: configs.get_integer("game", "RENET2_CONNECTION_TIMEOUT_SECS")?,
        server_ip: local_ip.unwrap_or(Ipv6Addr::LOCALHOST.into()),
        native_port: 0,
        wasm_wt_port: 0,
        wasm_ws_port: 0,
        proxy_ip,
        ws_domain,
        wss_certs,
        native_port_proxy: 0,
        wasm_wt_port_proxy: 0,
        wasm_ws_port_proxy: 0,
        has_wss_proxy: false,
    };

    // game framework config
    let tps = configs.get_integer("game", "TICKS_PER_SEC")?;
    let max_init_ticks = configs.get_integer::<u32>("game", "MAX_INIT_DURATION_SECS")? * tps;
    let game_fw_config = GameFwConfig::new(tps, max_init_ticks, 0);

    // game duration config
    let duration_config = GameDurationConfig {
        tile_select_duration_ms: configs.get_integer("game", "TILE_SELECT_DURATION_MILLIS")?,
        round_duration_ms: configs.get_integer("game", "ROUND_DURATION_MILLIS")?,
        num_rounds: configs.get_integer("game", "NUM_ROUNDS")?,
    };

    // prov game factory config
    let game_factory_config = ProvGameFactoryConfig {
        server_setup_config,
        game_fw_config,
        duration_config,
        resend_time: Duration::from_millis(configs.get_integer("game", "RENET2_RESEND_TIME_MILLIS")?),
    };

    Ok(game_factory_config)
}

//-------------------------------------------------------------------------------------------------------------------
