use bevy::prelude::*;
use bevy_cobweb_ui::prelude::*;
use utils_gui::LoadAsepriteFiles;

use super::*;

//-------------------------------------------------------------------------------------------------------------------

// TODO: replace this with sprite sorting (required if NPCs introduced, since they can walk in front of and
// behind buildings)
#[derive(Default, Debug, Reflect, PartialEq)]
pub(crate) struct MapZSorting
{
    pub(crate) tile: f32,
    pub(crate) select_effect: f32,
    pub(crate) building: f32,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Default, Debug, Reflect, PartialEq)]
pub(crate) struct MapSettings
{
    /// The asset path of aseprite tile textures.
    ///
    /// Tiles are accessed via aseprite tag.
    pub(crate) aseprite: String,

    pub(crate) sorting: MapZSorting,

    pub(crate) press_color: Color,

    /// Minimum radius of the cursor buffer region.
    pub(crate) cursor_buffer_min: f32,
    /// Starting radius of the cursor buffer region.
    pub(crate) cursor_buffer_start: f32,
    /// Exponential decay rate of the cursor buffer.
    /// r_start / e^(time / rate)
    pub(crate) cursor_buffer_decayrate_secs: f32,
}

impl Command for MapSettings
{
    fn apply(self, w: &mut World)
    {
        LoadAsepriteFiles(vec![self.aseprite.clone()]).apply(w);
        w.insert_resource(self);
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct MapPlugin;

impl Plugin for MapPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_command_type::<MapSettings>()
            .init_resource::<MapSettings>()
            .add_plugins(CameraControlPlugin)
            .add_plugins(CursorPlugin)
            .add_plugins(MapgenPlugin)
            .add_plugins(MapControlPlugin)
            .add_plugins(MapEffectsPlugin)
            .add_plugins(TileStatesPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------
