use bevy::prelude::*;
use game_core::*;
use serde::{Deserialize, Serialize};

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum ClientType
{
    Player,
    // Currently not supported.
    //Watcher,
}

//-------------------------------------------------------------------------------------------------------------------

/// Static information in a client app.
#[derive(Resource, Debug, Serialize, Deserialize)]
pub struct ClientContext
{
    /// This client's id
    pub client_id: ClientId,
    /// This client's type.
    pub client_type: ClientType,

    /// The game duration config.
    pub duration_config: GameDurationConfig,
    /// PRNG for generating the map deterministically.
    pub map_gen_prng: u64,
}

//-------------------------------------------------------------------------------------------------------------------
