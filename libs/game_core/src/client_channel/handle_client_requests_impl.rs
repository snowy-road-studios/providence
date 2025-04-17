use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_girk_game_fw::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn notify_request_rejected(
    In((client_id, request, reason)): In<(ClientId, ClientRequest, RejectionReason)>,
    mut sender: GameSender,
)
{
    sender.send_to_client(GameMsg::RequestRejected { reason, request }, client_id);
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn handle_game_state_request(In(client_id): In<ClientId>, world: &mut World)
{
    world.syscall(client_id, notify_game_state_single);
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn handle_player_input(In((_player_entity, input)): In<(Entity, PlayerInput)>, _world: &mut World)
{
    match input {
        PlayerInput::Placeholder => (),
    }
}

//-------------------------------------------------------------------------------------------------------------------
