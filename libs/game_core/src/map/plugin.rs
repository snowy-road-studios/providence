use bevy::prelude::*;

use super::MapGenPlugin;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct MapPlugin;

impl Plugin for MapPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(MapGenPlugin)
            .register_type::<super::TileId>();
    }
}

//-------------------------------------------------------------------------------------------------------------------
