use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy_aseprite_ultra::prelude::Aseprite;
use bevy_cobweb_ui::prelude::*;
use game_core::{TileData, TileId};
use utils_gui::{AsepriteMap, LoadAsepriteFiles};

use super::*;

//-------------------------------------------------------------------------------------------------------------------

// Inserting MapSettings causes the tile aseprite to be loaded, so this function will never 'miss' asset events
// for that file.
fn validate_aseprite(
    mut events: EventReader<AssetEvent<Aseprite>>,
    settings: Res<MapSettings>,
    assets: Res<Assets<Aseprite>>,
    aseprites: Res<AsepriteMap>,
)
{
    let handle = aseprites.get(&settings.tile_aseprite);

    for event in events.read() {
        match event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                if handle.id() != *id {
                    continue;
                }
                let Some(asset) = assets.get(*id) else { continue };

                if let Err(err) = settings.validate_aseprite(asset) {
                    tracing::error!("failed validating tile aseprite: {err:?}");
                }
            }
            _ => (),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Default, Debug, Reflect, PartialEq)]
pub(crate) struct GuiTileInfo
{
    pub(crate) aseprite_slice: String,
    pub(crate) display_name: String,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Default, Debug, Reflect, PartialEq)]
pub(crate) struct MapSettings
{
    /// The asset path of aseprite tile textures and tile effects.
    ///
    /// Tiles are accessed via aseprite tag.
    pub(crate) tile_aseprite: String,

    pub(crate) tiles: HashMap<TileId, GuiTileInfo>,
    pub(crate) select_effect_slice: String,

    pub(crate) press_color: Color,

    /// Minimum radius of the cursor buffer region.
    pub(crate) cursor_buffer_min: f32,
    /// Starting radius of the cursor buffer region.
    pub(crate) cursor_buffer_start: f32,
    /// Exponential decay rate of the cursor buffer.
    /// r_start / e^(time / rate)
    pub(crate) cursor_buffer_decayrate_secs: f32,
}

impl MapSettings
{
    pub(crate) fn validate(&self, tile_data: &TileData) -> Result<(), String>
    {
        for (id, _info) in self.tiles.iter() {
            if !tile_data.contains_key(id) {
                return Err(format!("MapSettings has {id:?} that is not registered in TileData"));
            }
        }
        if self.tiles.len() != tile_data.len() {
            return Err(format!("MapSettings does not have same number of tiles as TileData"));
        }

        Ok(())
    }

    fn validate_aseprite(&self, aseprite: &Aseprite) -> Result<(), String>
    {
        for (tile_id, tile_info) in self.tiles.iter() {
            if !aseprite.slices.contains_key(&tile_info.aseprite_slice) {
                return Err(
                    format!("MapSettings {tile_id:?} has slice {:?} not present in aseprite file",
                    tile_info.aseprite_slice),
                );
            }
        }

        Ok(())
    }
}

impl Command for MapSettings
{
    fn apply(self, w: &mut World)
    {
        let tile_data = w
            .get_resource::<TileData>()
            .expect("TileData should be inserted before app startup");
        self.validate(tile_data).unwrap();

        LoadAsepriteFiles(vec![self.tile_aseprite.clone()]).apply(w);
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
            .add_plugins(CameraControlPlugin)
            .add_plugins(CursorPlugin)
            .add_plugins(MapgenPlugin)
            .add_plugins(MapControlPlugin)
            .add_plugins(MapEffectsPlugin)
            .add_plugins(TileStatesPlugin)
            .add_systems(First, validate_aseprite.run_if(resource_exists::<MapSettings>));
    }
}

//-------------------------------------------------------------------------------------------------------------------
