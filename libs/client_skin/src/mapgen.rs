use bevy::prelude::*;
use bevy_aseprite_ultra::prelude::AseSpriteSlice;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::CobLoadableRegistrationAppExt;
use client_core::MapGenerated;
use game_core::*;
use utils_gui::{AsepriteMap, LoadAsepriteFiles};

//-------------------------------------------------------------------------------------------------------------------

fn add_tile_components(
    mut c: Commands,
    aseprites: Res<AsepriteMap>,
    grid: Res<HexGrid>,
    file: Res<TileFile>,
    tiles: Query<(Entity, &TileType)>,
)
{
    let aseprite = aseprites.get(&file.0);
    let sprite_size = grid.layout.rect_size();

    for (entity, tile) in tiles.iter() {
        let Ok(mut ec) = c.get_entity(entity) else { continue };
        let tag = match *tile {
            TileType::Mountain => "mountain",
            TileType::Water => "water",
            TileType::Rocky => "rocky",
            TileType::Ore => "ore",
            TileType::Forest => "forest",
            TileType::Grass => "grass",
        };

        ec.insert((
            AseSpriteSlice { aseprite: aseprite.clone(), name: tag.into() },
            Sprite { custom_size: Some(sprite_size), ..default() },
        ));
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Stores the asset path of aseprite tile textures.
///
/// Tiles are accessed via aseprite tag.
#[derive(Resource, Default, Debug, Reflect, PartialEq)]
pub struct TileFile(String);

impl Command for TileFile
{
    fn apply(self, w: &mut World)
    {
        LoadAsepriteFiles(vec![self.0.clone()]).apply(w);
        w.insert_resource(self);
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) struct MapgenPlugin;

impl Plugin for MapgenPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_command_type::<TileFile>()
            .add_reactor(broadcast::<MapGenerated>(), add_tile_components);
    }
}

//-------------------------------------------------------------------------------------------------------------------
