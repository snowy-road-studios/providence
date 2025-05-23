use std::time::Duration;

use bevy::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn initialize_game_time(time: Res<Time>, mut gametime: ResMut<GameTime>)
{
    gametime.initialize(time.elapsed());
}

//-------------------------------------------------------------------------------------------------------------------

fn update_game_time(time: Res<Time>, mut gametime: ResMut<GameTime>)
{
    gametime.update(time.elapsed());
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Default, Debug)]
pub(crate) struct GameTime
{
    /// Time the game started relative to when the app was constructed.
    start_time: Duration,
    /// Time elapsed since the game started.
    game_time: Duration,
    /// Amount of time the clock was accelerated.
    #[cfg(feature = "commands")]
    time_skip: Duration,
    #[cfg(feature = "commands")]
    paused: bool,
    #[cfg(feature = "commands")]
    pause_elapsed: Duration,
}

impl GameTime
{
    pub(crate) fn initialize(&mut self, start_time: Duration)
    {
        self.start_time = start_time;
    }

    pub(crate) fn update(&mut self, app_time: Duration)
    {
        #[cfg(feature = "commands")]
        {
            if self.paused {
                self.pause_elapsed = app_time.saturating_sub(
                    self.game_time
                        .saturating_sub(self.time_skip)
                        .saturating_add(self.start_time),
                );
                return;
            }
        }

        self.game_time = app_time.saturating_sub(self.start_time);

        #[cfg(feature = "commands")]
        {
            self.game_time = self.game_time.saturating_add(self.time_skip);
        }
    }

    #[cfg(feature = "commands")]
    pub(crate) fn add_timeskip(&mut self, skip: Duration)
    {
        self.time_skip = self.time_skip.saturating_add(skip);
        self.game_time = self.game_time.saturating_add(skip);
    }

    #[cfg(feature = "commands")]
    pub(crate) fn pause(&mut self)
    {
        self.paused = true;
        self.pause_elapsed = Duration::default();
    }

    #[cfg(feature = "commands")]
    pub(crate) fn unpause(&mut self)
    {
        self.paused = false;
        self.start_time = self.start_time.saturating_add(self.pause_elapsed);
        self.pause_elapsed = Duration::default();
    }

    pub(crate) fn elapsed(&self) -> Duration
    {
        self.game_time
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct TimeUpdateSet;

//-------------------------------------------------------------------------------------------------------------------

/// Game tick plugin.
pub(crate) struct GameTimePlugin;

impl Plugin for GameTimePlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<GameTime>()
            .configure_sets(Update, TimeUpdateSet.in_set(PostInitSet))
            .add_systems(OnExit(GameState::Init), initialize_game_time)
            .add_systems(Update, update_game_time.in_set(TimeUpdateSet));
    }
}

//-------------------------------------------------------------------------------------------------------------------
