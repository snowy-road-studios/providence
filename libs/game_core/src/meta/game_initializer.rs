use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use bevy_replicon::prelude::*;

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
    /// Watchers.
    pub watchers: HashSet<ClientId>,
}

//-------------------------------------------------------------------------------------------------------------------
