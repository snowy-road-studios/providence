use serde::{Deserialize, Serialize};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Player report for the game over report.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProvPlayerReport
{
    /// Client id within the game.
    pub client_id: ClientId,
}

//-------------------------------------------------------------------------------------------------------------------

/// Report emitted at the end of a game.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProvGameOverReport
{
    /// Id of the game used for this report.
    pub game_id: u64,

    /// How long the game took.
    pub game_duration_ms: u128,

    /// Number of rounds in the game.
    pub rounds: u32,

    /// Each player's individual report.
    pub player_reports: Vec<ProvPlayerReport>,
}

//-------------------------------------------------------------------------------------------------------------------
