use std::net::Ipv6Addr;
use std::time::Duration;

use bevy_girk_game_fw::*;
use game_core::*;
use renet2_setup::GameServerSetupConfig;
use utils::RootConfigs;

use crate::{protocol_id, ProvGameFactoryConfig};

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub struct ProvGameConfig
{
    pub server_setup_config: GameServerSetupConfig,
    pub resend_time: Duration,
    pub game_fw_config: GameFwConfig,
    pub duration_config: GameDurationConfig,
    pub game_data: GameData,
}

//-------------------------------------------------------------------------------------------------------------------

pub fn extract_game_configs(
    factory_config: ProvGameFactoryConfig,
    configs: &RootConfigs,
) -> Result<ProvGameConfig, String>
{
    // server setup config
    let server_setup_config = GameServerSetupConfig {
        protocol_id: protocol_id(),
        expire_secs: configs.get_integer("game", "RENET2_CONNECTION_EXPIRY_SECS")?,
        timeout_secs: configs.get_integer("game", "RENET2_CONNECTION_TIMEOUT_SECS")?,
        server_ip: factory_config
            .local_ip
            .unwrap_or(Ipv6Addr::LOCALHOST.into()),
        native_port: 0,
        wasm_wt_port: 0,
        wasm_ws_port: 0,
        proxy_ip: factory_config.proxy_ip,
        ws_domain: factory_config.ws_domain,
        wss_certs: factory_config.wss_certs,
        native_port_proxy: 0,
        wasm_wt_port_proxy: 0,
        wasm_ws_port_proxy: 0,
        has_wss_proxy: false,
    };

    // game framework config
    let tps = configs.get_integer("game", "TICKS_PER_SEC")?;
    let max_init_ticks = configs.get_integer::<u32>("game", "MAX_INIT_DURATION_SECS")? * tps;
    let end_ticks = (configs.get_integer::<u32>("game", "END_DURATION_MILLIS")? * tps) / 1000;
    let game_fw_config = GameFwConfig::new(tps, max_init_ticks, end_ticks);

    // game duration config
    let duration_config = GameDurationConfig {
        tile_select_duration_ms: configs.get_integer("game", "TILE_SELECT_DURATION_MILLIS")?,
        round_duration_ms: configs.get_integer("game", "ROUND_DURATION_MILLIS")?,
        num_rounds: configs.get_integer("game", "NUM_ROUNDS")?,
    };

    // misc configs
    let game_data = GameData::new(configs)?;

    // prov game factory config
    let game_config = ProvGameConfig {
        server_setup_config,
        resend_time: Duration::from_millis(configs.get_integer("game", "RENET2_RESEND_TIME_MILLIS")?),
        game_fw_config,
        duration_config,
        game_data,
    };

    Ok(game_config)
}

//-------------------------------------------------------------------------------------------------------------------
