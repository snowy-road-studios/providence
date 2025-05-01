use bevy::prelude::*;
use bevy_girk_game_fw::*;
use bevy_girk_utils::apply_state_transitions;
use serde::{Deserialize, Serialize};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Check the game duration conditions and update the game state.
fn update_game_state(
    game_ctx: Res<GameContext>,
    game_time: Res<GameTime>,
    current_game_state: Res<State<GameState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
)
{
    if **current_game_state == GameState::End {
        return;
    }

    // get expected state based on elapsed ticks
    let duration_config = game_ctx.duration_config;
    let new_game_state = duration_config.expected_state(game_time.elapsed());

    // update the game state
    if new_game_state == **current_game_state {
        return;
    }
    next_game_state.set(new_game_state);
    tracing::info!(?new_game_state, "new game state");
}

//-------------------------------------------------------------------------------------------------------------------

fn set_game_end_flag(
    ctx: Res<GameContext>,
    game_time: Res<GameTime>,
    round: Res<GameRound>,
    players: Query<&PlayerId>,
    mut game_end_flag: ResMut<GameEndFlag>,
)
{
    // collect player reports
    let player_reports = players
        .iter()
        .map(|&player_id| ProvPlayerReport { client_id: player_id.id })
        .collect();

    // build game over report
    let game_over_report = ProvGameOverReport {
        game_id: ctx.game_id,
        game_duration_ms: game_time.elapsed().as_millis(),
        rounds: **round,
        player_reports,
    };

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
    sender.send_to_client(GameMsg::CurrentGameState(**game_state), client_id);
}

//-------------------------------------------------------------------------------------------------------------------

/// Game state
#[derive(States, Debug, Default, Eq, PartialEq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum GameState
{
    #[default]
    Startup,
    Init,
    TileSelect,
    Play,
    End,
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
            .configure_sets(Update, GameStateUpdateSet.in_set(PostInitSet))
            .add_systems(PostStartup, set_init_state)
            .add_systems(
                Update,
                (update_game_state, apply_state_transitions)
                    .chain()
                    .in_set(GameStateUpdateSet),
            )
            .add_systems(OnEnter(GameState::Init), notify_game_state_all)
            .add_systems(OnEnter(GameState::TileSelect), notify_game_state_all)
            .add_systems(OnEnter(GameState::Play), notify_game_state_all)
            .add_systems(OnEnter(GameState::End), (notify_game_state_all, set_game_end_flag));
    }
}

//-------------------------------------------------------------------------------------------------------------------
