use std::collections::{HashMap, HashSet};
use std::vec::Vec;

use bevy::prelude::*;
use bevy_girk_client_fw::*;
use bevy_girk_game_fw::*;
use bevy_girk_game_instance::*;
use bevy_girk_wiring_server::*;
use game_core::*;
use renet2_setup::{ClientCounts, ConnectionType};
use serde::{Deserialize, Serialize};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

struct GameStartupHelper
{
    client_set: GameFwClients,
    prov_init: ProvGameInitializer,
    start_infos: Vec<GameStartInfo>,
    client_counts: ClientCounts,
}

//-------------------------------------------------------------------------------------------------------------------

/// Prepare information to use when setting up the game app.
fn prepare_game_startup(
    game_id: u64,
    config: &GameFwConfig,
    client_init_data: Vec<ClientGameInit>,
    duration_config: GameDurationConfig,
) -> Result<GameStartupHelper, String>
{
    // prepare each client
    let mut client_set = HashSet::with_capacity(client_init_data.len());
    let mut players = HashMap::with_capacity(client_init_data.len());
    let mut start_infos = Vec::with_capacity(client_init_data.len());
    let mut client_counts = ClientCounts::default();

    for client_init in client_init_data {
        let client_id = client_init.client_id;

        // handle client type
        let initializer = match client_init.client_type {
            ClientTypeInfo::Player { player_name } => {
                players.insert(
                    client_id,
                    PlayerState {
                        id: PlayerId { id: client_id },
                        name: PlayerName { name: player_name },
                        ..Default::default()
                    },
                );
                ClientInitializer {
                    context: ClientContext::new(client_id, ClientType::Player, duration_config),
                }
            }
        };

        // save client id for the game
        client_set.insert(client_id);

        // count client type
        client_counts.add(client_init.connection, client_id);

        // Prep start info for the client.
        let client_fw_config = ClientFwConfig::new(config.ticks_per_sec(), game_id, client_id);
        let client_start_pack = ClientStartPack { client_fw_config, initializer };
        let start_info = GameStartInfo::new(game_id, client_init.user_id, client_id, client_start_pack);
        start_infos.push(start_info)
    }
    debug_assert_eq!(client_set.len(), start_infos.len());

    // finalize
    let seed = {
        // Seed is only needed on WASM when making a local-player game.
        #[cfg(target_family = "wasm")]
        {
            wasm_timer::SystemTime::now()
                .duration_since(wasm_timer::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        }

        #[cfg(not(target_family = "wasm"))]
        bevy_girk_utils::gen_rand128()
    };
    let game_context = GameContext::new(game_id, seed, duration_config);

    Ok(GameStartupHelper {
        client_set: GameFwClients::new(client_set),
        prov_init: ProvGameInitializer { game_context, players },
        start_infos,
        client_counts,
    })
}

//-------------------------------------------------------------------------------------------------------------------

/// Client init data used when setting up a game.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientTypeInfo
{
    Player
    {
        player_name: String
    }, // Watcher,
}

//-------------------------------------------------------------------------------------------------------------------

/// Used by a game factory to initialize a client in the game.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientGameInit
{
    /// Indicates the type of connection the client wants to have with the server.
    pub connection: ConnectionType,
    /// The client's server-side user id.
    pub user_id: u128,
    /// The client's in-game id.
    pub client_id: ClientId,
    /// Client init data for use in initializing a game.
    pub client_type: ClientTypeInfo,
}

//-------------------------------------------------------------------------------------------------------------------

/// Used by a game factory to initialize a game.
///
/// Produced by a launch pack source.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LaunchData
{
    /// Game config.
    pub config: ProvGameFactoryConfig,

    /// Client init data for use in initializing a game.
    pub clients: Vec<ClientGameInit>,
}

//-------------------------------------------------------------------------------------------------------------------

/// Start-up pack for clients.
#[derive(Serialize, Deserialize)]
pub struct ClientStartPack
{
    /// Client framework config.
    pub client_fw_config: ClientFwConfig,
    /// Client initializer.
    pub initializer: ClientInitializer,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub struct ProvGameFactory;

impl GameFactoryImpl for ProvGameFactory
{
    type Launch = LaunchData;

    fn new_game(&self, app: &mut App, game_id: u64, data: LaunchData) -> Result<GameStartReport, String>
    {
        // initialize clients and game config
        let config = data.config;
        let startup = prepare_game_startup(game_id, &config.game_fw_config, data.clients, config.duration_config)?;

        // girk server config
        let server_config = GirkServerConfig {
            clients: startup.client_set,
            config: config.game_fw_config,
            game_server_config: config.server_setup_config,
            resend_time: config.resend_time,
            client_counts: startup.client_counts,
        };

        // prepare game app
        let metas = prepare_girk_game_app(app, server_config)?;
        prepare_game_app_core(app, startup.prov_init, config.game_data);

        Ok(GameStartReport { metas, start_infos: startup.start_infos })
    }
}

//-------------------------------------------------------------------------------------------------------------------
