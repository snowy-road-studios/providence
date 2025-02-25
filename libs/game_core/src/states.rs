use bevy::prelude::*;
use bevy_girk_game_fw::*;
use bevy_girk_utils::apply_state_transitions;
use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Check the game duration conditions and update the game state.
fn update_game_state(
    game_ctx: Res<ProvGameContext>,
    game_tick: Res<GameTick>,
    current_game_state: Res<State<GameState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
)
{
    // get expected state based on elapsed ticks
    let duration_config = game_ctx.duration_config();
    let new_game_state = duration_config.expected_state(**game_tick);

    // update the game state
    if new_game_state == **current_game_state {
        return;
    }
    next_game_state.set(new_game_state);
    tracing::info!(?new_game_state, "new game state");
}

//-------------------------------------------------------------------------------------------------------------------

fn set_game_end_flag(
    game_tick: Res<GameTick>,
    players: Query<(&PlayerId, &PlayerScore)>,
    mut game_end_flag: ResMut<GameEndFlag>,
)
{
    // collect player reports
    let player_reports = players
        .iter()
        .map(|(&player_id, &score)| ProvPlayerReport { client_id: player_id.id, score })
        .collect();

    // build game over report
    let game_over_report = ProvGameOverReport { final_game_tick: **game_tick, player_reports };

    // serialize it
    let game_over_report_final = GameOverReport::new(&game_over_report);

    // set the game end flag
    game_end_flag.set(game_over_report_final);
    tracing::info!("game end flag set");
}

//-------------------------------------------------------------------------------------------------------------------

// Must set init state separately because the first state runs before `PreStartup` so OnEnter won't have
// access to anything added by systems.
fn set_init_state(mut next_game_state: ResMut<NextState<GameState>>)
{
    next_game_state.set(GameState::Init);
}

//-------------------------------------------------------------------------------------------------------------------

/// Notify all clients of the current game state.
pub(crate) fn notify_game_state_all(game_state: Res<State<GameState>>, mut sender: GameSender)
{
    sender.send_to_all(GameMsg::CurrentGameState(**game_state));
}

//-------------------------------------------------------------------------------------------------------------------

/// Helper function-system for accessing the game state.
pub(crate) fn get_game_state(game_state: Res<State<GameState>>) -> GameState
{
    **game_state
}

//-------------------------------------------------------------------------------------------------------------------

/// Notify a single client of the current game state.
pub(crate) fn notify_game_state_single(
    In(client_id): In<ClientId>,
    game_state: Res<State<GameState>>,
    mut sender: GameSender,
)
{
    sender.send_to_client(GameMsg::CurrentGameState(**game_state), client_id.get());
}

//-------------------------------------------------------------------------------------------------------------------

/// Game state
#[derive(States, Debug, Default, Eq, PartialEq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum GameState
{
    #[default]
    Startup,
    Init,
    Prep,
    Play,
    GameOver,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct GameStateUpdateSet;

//-------------------------------------------------------------------------------------------------------------------

/// Game tick plugin.
pub(crate) struct GameStatePlugin;

impl Plugin for GameStatePlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_state::<GameState>()
            .enable_state_scoped_entities::<GameState>()
            .add_systems(PostStartup, set_init_state)
            .add_systems(
                Update,
                (
                    // determine which game state the previous tick was in and set it
                    update_game_state,
                    apply_state_transitions,
                )
                    .chain()
                    .in_set(GameSet::PostInit)
                    .in_set(GameStateUpdateSet),
            )
            .add_systems(OnEnter(GameState::Init), notify_game_state_all)
            .add_systems(OnEnter(GameState::Prep), notify_game_state_all)
            .add_systems(OnEnter(GameState::Play), notify_game_state_all)
            .add_systems(OnEnter(GameState::GameOver), (notify_game_state_all, set_game_end_flag));
    }
}

//-------------------------------------------------------------------------------------------------------------------
