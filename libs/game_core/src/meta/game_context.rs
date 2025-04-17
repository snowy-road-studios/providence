use bevy::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Static information in a game app.
#[derive(Resource)]
pub struct GameContext
{
    /// Id for this game.
    game_id: u64,
    /// Seed for the game's deterministic random number generator.
    seed: u128,
    /// Game duration config.
    duration_config: GameDurationConfig,
}

impl GameContext
{
    /// New game context
    pub fn new(game_id: u64, seed: u128, duration_config: GameDurationConfig) -> GameContext
    {
        GameContext { game_id, seed, duration_config }
    }

    pub fn game_id(&self) -> u64
    {
        self.game_id
    }

    pub fn seed(&self) -> u128
    {
        self.seed
    }

    pub fn duration_config(&self) -> &GameDurationConfig
    {
        &self.duration_config
    }
}

//-------------------------------------------------------------------------------------------------------------------
