use bevy::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Static information in a game app.
#[derive(Resource)]
pub struct GameContext
{
    /// Id for this game.
    pub game_id: u64,
    /// Seed for the game's deterministic random number generator.
    pub seed: u128,
    /// Game duration config.
    pub duration_config: GameDurationConfig,
}

//-------------------------------------------------------------------------------------------------------------------
