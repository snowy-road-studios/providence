use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Configuration details for game duration.
/*
TODO:
server
- send time remaining in current phase every 5 seconds in reliable unordered messages
    - also on client connect
    - also on phase change
- store current round as resource
    - send current round as message
        - on phase change
        - on client connect
        - TODO: replicate this resource

client
- store phase timer as resource
- update phase timer when phase timer message received
*/
#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug, Serialize, Deserialize)]
pub struct GameDurationConfig
{
    /// Duration of tile selection phase.
    pub tile_select_duration_ms: u64,
    /// Duration of each game round.
    pub round_duration_ms: u64,
    /// Number of rounds in the game.
    pub num_rounds: u32,
}

impl GameDurationConfig
{
    pub fn expected_state(&self, game_time: Duration) -> GameState
    {
        // prep
        if game_time.as_millis() <= self.tile_select_duration_ms as u128 {
            return GameState::TileSelect;
        }

        // play
        let total_game_duration = self.tile_select_duration_ms + self.round_duration_ms * (self.num_rounds as u64);
        if game_time.as_millis() <= total_game_duration as u128 {
            return GameState::Play;
        }

        // game over
        GameState::End
    }

    pub fn select_remaining_ms(&self, game_time: Duration) -> Option<u128>
    {
        if game_time.as_millis() > self.tile_select_duration_ms as u128 {
            return None;
        }

        Some(self.tile_select_duration_ms as u128 - game_time.as_millis())
    }

    pub fn round_and_remaining_ms(&self, game_time: Duration) -> Option<(u32, u128)>
    {
        if game_time.as_millis() <= self.tile_select_duration_ms as u128 {
            return None;
        }
        let play_time_ms = game_time
            .as_millis()
            .saturating_sub(self.tile_select_duration_ms as u128);
        let round_time_ms = self.round_duration_ms.max(1) as u128;
        let rounds_complete = play_time_ms / round_time_ms;
        let remaining_ms = play_time_ms % round_time_ms;

        if rounds_complete as u32 == self.num_rounds {
            return Some((self.num_rounds, 0));
        }

        Some((rounds_complete as u32 + 1, remaining_ms))
    }
}

//-------------------------------------------------------------------------------------------------------------------
