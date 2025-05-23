use bevy::prelude::*;

use super::*;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct GameUiPlugin;

impl Plugin for GameUiPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(GameUiHudPlugin)
            .add_plugins(GameUiSettingsPlugin)
            .add_plugins(GameUiTileSelectPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------
