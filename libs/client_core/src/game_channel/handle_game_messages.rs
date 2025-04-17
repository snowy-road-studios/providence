use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_girk_game_fw::*;
use game_core::*;

use super::*;
use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn handle_request_rejected(request: ClientRequest, reason: RejectionReason)
{
    tracing::warn!("game request {request:?} rejected: {reason:?}");
}

//-------------------------------------------------------------------------------------------------------------------

/// Handle a message sent to the client from the game.
///
/// Callback for [`GameMessageHandler`].
pub(crate) fn handle_game_message(world: &mut World, _tick: Tick, message: GameMsg)
{
    let _state = **world.resource::<State<ClientState>>();

    match message {
        GameMsg::RequestRejected { reason, request } => handle_request_rejected(request, reason),
        GameMsg::CurrentGameState(game_state) => world.syscall(game_state, handle_game_state),
        GameMsg::TileSelectInfo { remaining_ms } => world.syscall(remaining_ms, handle_tile_select_info),
        GameMsg::RoundInfo { round, remaining_ms } => world.syscall((round, remaining_ms), handle_round_info),
    }
}

//-------------------------------------------------------------------------------------------------------------------
