use bevy::prelude::*;
use bevy_girk_client_fw::*;
use game_core::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Request the current game mode.
fn request_game_state(mut sender: ClientSender)
{
    sender.send(ClientRequest::GetGameState);
}

//-------------------------------------------------------------------------------------------------------------------

pub struct ClientCorePlugin;

impl Plugin for ClientCorePlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(GameReplicationPlugin)
            .add_plugins(ClientSetsPlugin)
            .add_plugins(ClientSetupPlugin)
            // Note: watcher clients re-use the player skin.
            .add_plugins(PlayerInputPlugin)
            .add_systems(OnEnter(ClientInitState::Done), request_game_state)
            .configure_sets(Update, PlayerInputSet.in_set(ClientLogicSet::Admin));
    }
}

//-------------------------------------------------------------------------------------------------------------------
