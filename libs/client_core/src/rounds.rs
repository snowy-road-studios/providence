use std::time::Duration;

use bevy::prelude::*;
use wasm_timer::{SystemTime, UNIX_EPOCH};

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Default, Debug)]
pub struct TileSelectTimer
{
    /// Approximately when the timer will end.
    ///
    /// Initialized by a message from the server, which will have some latency.
    end_time: Duration,
}

impl TileSelectTimer
{
    pub(crate) fn set(&mut self, remaining_ms: u128)
    {
        let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let remaining = Duration::from_millis(remaining_ms as u64);
        self.end_time = time.saturating_add(remaining);
    }

    pub fn end_time(&self) -> Duration
    {
        self.end_time
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Default, Debug)]
pub struct RoundTimer
{
    /// The current round.
    round: u32,
    /// Approximately when the timer for the current round will end.
    ///
    /// Initialized by a message from the server, which will have some latency.
    round_end_time: Duration,
}

impl RoundTimer
{
    pub(crate) fn set(&mut self, round: u32, remaining_ms: u128)
    {
        let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let remaining = Duration::from_millis(remaining_ms as u64);

        self.round = round;
        self.round_end_time = time.saturating_add(remaining);
    }

    pub fn round(&self) -> u32
    {
        self.round
    }

    pub fn round_end_time(&self) -> Duration
    {
        self.round_end_time
    }

    /// Time remaining in the current round.
    pub fn remaining_time(&self) -> Duration
    {
        self.round_end_time
            .saturating_sub(SystemTime::now().duration_since(UNIX_EPOCH).unwrap())
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct RoundsPlugin;

impl Plugin for RoundsPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<TileSelectTimer>()
            .init_resource::<RoundTimer>();
    }
}

//-------------------------------------------------------------------------------------------------------------------
