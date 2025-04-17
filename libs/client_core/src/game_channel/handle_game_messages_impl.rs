use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_girk_client_fw::*;
use bevy_girk_utils::apply_state_transitions;
use game_core::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Use current game state to update client state.
fn update_client_state(
    In(game_state): In<GameState>,
    client_init_state: Res<State<ClientInitState>>,
    current_client_state: Res<State<ClientState>>,
    mut next_client_state: ResMut<NextState<ClientState>>,
)
{
    // do not update game state if we are in the process of initializing the client
    if *client_init_state != ClientInitState::Done {
        return;
    }

    // update game state
    let new_client_state = match game_state {
        GameState::Startup | GameState::Init => ClientState::Init,
        GameState::TileSelect => ClientState::TileSelect,
        GameState::Play => ClientState::Play,
        GameState::End => ClientState::End,
    };

    if new_client_state == **current_client_state {
        return;
    }
    next_client_state.set(new_client_state);
    tracing::info!(?new_client_state, "new client state");
}

//-------------------------------------------------------------------------------------------------------------------

/// Handle current game state.
pub(super) fn handle_game_state(In(current_game_state): In<GameState>, world: &mut World)
{
    world.syscall(current_game_state, update_client_state);
    world.syscall((), apply_state_transitions);
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn handle_tile_select_info(In(remaining_ms): In<u128>, mut select_timer: ResMut<TileSelectTimer>)
{
    select_timer.set(remaining_ms);
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn handle_round_info(In((round, remaining_ms)): In<(u32, u128)>, mut round_timer: ResMut<RoundTimer>)
{
    round_timer.set(round, remaining_ms);
}

//-------------------------------------------------------------------------------------------------------------------
