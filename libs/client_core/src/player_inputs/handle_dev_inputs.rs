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

/// Marshals dev inputs from the client to the server.
pub(crate) fn handle_dev_inputs(world: &mut World)
{
    let Some(state) = world.get_resource::<State<ClientState>>() else {
        return;
    };
    let state: ClientState = **state;

    let ctx = world.resource::<ClientContext>();
    let client_id = ctx.id();
    let is_player = ctx.client_type() == ClientType::Player;
    let is_playstate = state == ClientState::Play;

    let Some(inputs) = world.remove_resource::<Receiver<DevInput>>() else {
        return;
    };

    while let Some(input) = inputs.try_recv() {
        if !is_player {
            tracing::warn!("ignoring dev input sent by non-player client {client_id}: {input:?}");
            continue;
        }
        if !is_playstate {
            tracing::warn!("ignoring invalid dev input sent during {state:?}: {input:?}");
            continue;
        }
        world.syscall(ClientRequest::DevInput(input), send_client_request);
    }

    world.insert_resource(inputs);
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn clear_dev_inputs(world: &mut World)
{
    let Some(inputs) = world.get_resource_mut::<Receiver<DevInput>>() else {
        return;
    };

    while let Some(input) = inputs.try_recv() {
        tracing::debug!("discarding dev input: {input:?}")
    }
}

//-------------------------------------------------------------------------------------------------------------------
