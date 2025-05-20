use bevy::prelude::*;

use super::*;

//-------------------------------------------------------------------------------------------------------------------

/// Marker component for the main camera (orthographic).
#[derive(Component)]
#[component(immutable)]
pub struct MainCamera;

//-------------------------------------------------------------------------------------------------------------------

/// Plugin for setting up a client skin.
///
/// Prerequisites:
/// - Use `make_game_client_core()` to set up a client app.
pub struct ClientSkinPlugin;

impl Plugin for ClientSkinPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(UiPlugin)
            .add_plugins(FpsTrackerPlugin)
            .add_plugins(EventsPlugin)
            .add_plugins(MapPlugin)
            .add_plugins(SpriteLayerImplPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------
