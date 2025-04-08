use std::collections::HashMap;

use bevy::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Data used on startup to initialize a game app.
///
/// This resource is consumed during initialization.
#[derive(Resource)]
pub struct ProvGameInitializer
{
    /// Game context.
    pub game_context: ProvGameContext,
    /// Player states.
    pub players: HashMap<ClientId, PlayerState>,
}

//-------------------------------------------------------------------------------------------------------------------
