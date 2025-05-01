use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_girk_client_fw::ClientSender;
use bevy_girk_utils::Receiver;
use game_core::*;
use wiring_game_instance::{ClientContext, ClientType};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn send_client_request(In(request): In<ClientRequest>, mut sender: ClientSender)
{
    sender.send(request);
}

//-------------------------------------------------------------------------------------------------------------------

/// Marshals player inputs from the client to the server.
pub(crate) fn handle_player_inputs(world: &mut World)
{
    let Some(state) = world.get_resource::<State<ClientState>>() else {
        return;
    };
    let state: ClientState = **state;

    let ctx = world.resource::<ClientContext>();
    let client_id = ctx.client_id;
    let is_player = ctx.client_type == ClientType::Player;
    let is_playstate = state == ClientState::Play;

    let Some(inputs) = world.remove_resource::<Receiver<PlayerInput>>() else {
        return;
    };

    while let Some(input) = inputs.try_recv() {
        if !is_player {
            tracing::warn!("ignoring input sent by non-player client {client_id}: {input:?}");
            continue;
        }
        if !is_playstate {
            tracing::warn!("ignoring invalid input sent during {state:?}: {input:?}");
            continue;
        }
        world.syscall(ClientRequest::PlayerInput(input), send_client_request);
    }

    world.insert_resource(inputs);
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn clear_player_inputs(world: &mut World)
{
    let Some(inputs) = world.get_resource_mut::<Receiver<PlayerInput>>() else {
        return;
    };

    while let Some(input) = inputs.try_recv() {
        tracing::debug!("discarding player input: {input:?}");
    }
}

//-------------------------------------------------------------------------------------------------------------------
