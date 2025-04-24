use std::time::Duration;

use bevy::prelude::*;
use wasm_timer::{SystemTime, UNIX_EPOCH};

//-------------------------------------------------------------------------------------------------------------------

fn time_now() -> Duration
{
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Default, Debug)]
pub struct TileSelectTimer
{
    /// Approximately when the timer will end.
    ///
    /// Initialized by a message from the server, which will have some latency.
    end_time: Duration,

    paused: bool,
    pause_start_time: Duration,
}

impl TileSelectTimer
{
    pub(crate) fn set(&mut self, remaining_ms: u128)
    {
        let now = time_now();
        let remaining = Duration::from_millis(remaining_ms as u64);
        self.end_time = now.saturating_add(remaining);
        self.pause_start_time = now;
    }

    pub(crate) fn pause(&mut self)
    {
        if self.paused {
            return;
        }
        self.paused = true;
        self.pause_start_time = time_now();
    }

    pub(crate) fn unpause(&mut self)
    {
        if !self.paused {
            return;
        }
        self.paused = false;
        self.set(
            self.end_time
                .saturating_sub(self.pause_start_time)
                .as_millis(),
        );
    }

    pub fn remaining_time(&self) -> Duration
    {
        let ref_time = match self.paused {
            true => self.pause_start_time,
            false => time_now(),
        };
        self.end_time.saturating_sub(ref_time)
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

    paused: bool,
    pause_start_time: Duration,
}

impl RoundTimer
{
    pub(crate) fn set(&mut self, round: u32, remaining_ms: u128)
    {
        let now = time_now();
        let remaining = Duration::from_millis(remaining_ms as u64);

        self.round = round;
        self.round_end_time = now.saturating_add(remaining);
        self.pause_start_time = now;
    }

    pub(crate) fn pause(&mut self)
    {
        if self.paused {
            return;
        }
        self.paused = true;
        self.pause_start_time = time_now();
    }

    pub(crate) fn unpause(&mut self)
    {
        if !self.paused {
            return;
        }
        self.paused = false;
        self.set(
            self.round,
            self.round_end_time
                .saturating_sub(self.pause_start_time)
                .as_millis(),
        );
    }

    /// Time remaining in the current round.
    pub fn remaining_time(&self) -> Duration
    {
        let ref_time = match self.paused {
            true => self.pause_start_time,
            false => time_now(),
        };
        self.round_end_time.saturating_sub(ref_time)
    }

    pub fn round(&self) -> u32
    {
        self.round
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
