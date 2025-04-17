use bevy::prelude::*;
use bevy_girk_game_fw::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn round_event_ocurred(mut events: EventReader<RoundChange>) -> bool
{
    events.read().count() > 0
}

//-------------------------------------------------------------------------------------------------------------------

/// Runs in [`Update`] when not in [`GameFwState::Init`].
///
/// This set is modal.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct PostInitSet;

//-------------------------------------------------------------------------------------------------------------------

/// System sets that contain game logic. These don't run during initialization.
///
/// These sets are modal.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameSet
{
    TileSelect,
    Play,
    End,
}

//-------------------------------------------------------------------------------------------------------------------

/// Game core logic sets.
///
/// These sets are ordinal in schedule `Update`.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameLogicSet
{
    Admin,
    /// Set for systems that run when the round counter increments during GameState::Play.
    RoundChangeover,
    Update,
}

//-------------------------------------------------------------------------------------------------------------------

/// Configures root-level system sets.
pub struct GameSetsPlugin;

impl Plugin for GameSetsPlugin
{
    fn build(&self, app: &mut App)
    {
        app.configure_sets(
            Update,
            (GameLogicSet::Admin, GameLogicSet::RoundChangeover, GameLogicSet::Update).chain(),
        )
        .configure_sets(Update, PostInitSet.run_if(not(in_state(GameFwState::Init))))
        .configure_sets(Update, GameLogicSet::RoundChangeover.run_if(round_event_ocurred))
        .configure_sets(
            Update,
            GameSet::TileSelect
                .run_if(in_state(GameFwState::Game))
                .run_if(in_state(GameState::TileSelect))
                .in_set(GameLogicSet::Update),
        )
        .configure_sets(
            Update,
            GameSet::Play
                .run_if(in_state(GameFwState::Game))
                .run_if(in_state(GameState::Play))
                .in_set(GameLogicSet::Update),
        )
        // - This will only run in the span between entering 'game over' and the GameFwState moving to 'End', which
        //   is controlled by `GameFwConfig::max_end_ticks()`.
        //todo: allow End to last indefinitely?
        .configure_sets(
            Update,
            GameSet::End
                .run_if(in_state(GameFwState::Game))
                .run_if(in_state(GameState::End))
                .in_set(GameLogicSet::Update),
        );
    }
}

//-------------------------------------------------------------------------------------------------------------------
