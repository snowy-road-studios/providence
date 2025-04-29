use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_girk_client_fw::ClientFwState;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Reactive event broadcast `OnExit(ClientFwState::Init)`.
#[derive(Default)]
pub struct ExitingInit;

//-------------------------------------------------------------------------------------------------------------------

/// Reactive event broadcast every tick in [`ClientLogicSet::End`].
#[derive(Default)]
pub struct AppUpdateEnd;

//-------------------------------------------------------------------------------------------------------------------

/// System that broadcasts `T`.
pub fn broadcast_system<T: Default + Send + Sync + 'static>(mut c: Commands)
{
    c.react().broadcast(T::default());
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) struct EventsPlugin;

impl Plugin for EventsPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_systems(OnExit(ClientFwState::Init), broadcast_system::<ExitingInit>)
            .add_systems(Update, broadcast_system::<AppUpdateEnd>.in_set(ClientLogicSet::End));
    }
}

//-------------------------------------------------------------------------------------------------------------------
