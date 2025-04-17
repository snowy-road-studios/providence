use bevy::prelude::*;
use bevy_girk_client_fw::{client_is_initializing, ClientAppState, ClientFwState};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Client core sets.
///
/// These sets are modal in schedule `Update`.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum ClientSet
{
    /// Runs initialization logic.
    Init,
    /// Runs in game mode 'tile select' (but not when initializing).
    TileSelect,
    /// Runs in game mode 'play' (but not when initializing).
    Play,
    /// Runs in game mode 'end' (but not when initializing).
    End,
}

//-------------------------------------------------------------------------------------------------------------------

/// Client core logic sets.
///
/// These sets are ordinal in schedule `Update`.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum ClientLogicSet
{
    Admin,
    Update,
    End,
}

//-------------------------------------------------------------------------------------------------------------------

/// Configures root-level system sets.
pub(crate) struct ClientSetsPlugin;

impl Plugin for ClientSetsPlugin
{
    fn build(&self, app: &mut App)
    {
        app.configure_sets(
            Update,
            (ClientLogicSet::Admin, ClientLogicSet::Update, ClientLogicSet::End)
                .chain()
                .run_if(in_state(ClientAppState::Game)),
        )
        .configure_sets(
            Update,
            ClientSet::Init
                .run_if(client_is_initializing)
                .run_if(in_state(ClientState::Init))
                .in_set(ClientLogicSet::Update),
        )
        .configure_sets(
            Update,
            ClientSet::TileSelect
                .run_if(in_state(ClientFwState::Game))
                .run_if(in_state(ClientState::TileSelect)),
        )
        .configure_sets(
            Update,
            ClientSet::Play
                .run_if(in_state(ClientFwState::Game))
                .run_if(in_state(ClientState::Play)),
        )
        .configure_sets(
            Update,
            ClientSet::End
                .run_if(in_state(ClientFwState::End))
                .run_if(in_state(ClientState::End)),
        );
    }
}

//-------------------------------------------------------------------------------------------------------------------
