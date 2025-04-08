use bevy::prelude::*;
use bevy_girk_client_fw::ClientAppState;

//-------------------------------------------------------------------------------------------------------------------

/// Client core mode
#[derive(SubStates, Debug, Default, Eq, PartialEq, Hash, Copy, Clone)]
#[source(ClientAppState = ClientAppState::Game)]
pub enum ClientState
{
    #[default]
    Init,
    TileSelect,
    Play,
    End,
}

//-------------------------------------------------------------------------------------------------------------------
