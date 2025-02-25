use bevy::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Static information in a game app.
#[derive(Resource)]
pub struct ProvGameContext
{
    /// Seed for the game's deterministic random number generator.
    seed: u128,
    /// Game duration config.
    duration_config: GameDurationConfig,
}

impl ProvGameContext
{
    /// New game context
    pub fn new(seed: u128, duration_config: GameDurationConfig) -> ProvGameContext
    {
        ProvGameContext { seed, duration_config }
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
