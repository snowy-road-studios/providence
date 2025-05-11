use bevy::prelude::*;
use bevy_aseprite_ultra::prelude::AseSlice;
use bevy_cobweb::prelude::*;
use client_core::MapGenerated;
use game_core::*;
use hexx::Hex;
use utils_gui::AsepriteMap;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn add_tile_components(
    mut c: Commands,
    aseprites: Res<AsepriteMap>,
    grid: Res<HexGrid>,
    map_settings: Res<MapSettings>,
    tiles: Query<(Entity, &TileType)>,
)
{
    let aseprite = aseprites.get(&map_settings.aseprite);
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
            AseSlice { aseprite: aseprite.clone(), name: tag.into() },
            Sprite { custom_size: Some(sprite_size), ..default() },
            Transform::from_translation(Vec3::default().with_z(map_settings.sorting.tile)),
        ));
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn set_camera_boundary(mut c: Commands, grid: Res<HexGrid>)
{
    let upper_right = grid
        .layout
        .hex_to_world_pos(Hex { x: grid.dimension, y: -(grid.dimension / 2) });
    let lower_left = grid.layout.hex_to_world_pos(Hex {
        x: -grid.dimension,
        y: (grid.dimension / 2) + (grid.dimension % 2),
    });

    c.insert_resource(CameraBoundary { upper_right, lower_left });
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) struct MapgenPlugin;

impl Plugin for MapgenPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_reactor(broadcast::<MapGenerated>(), add_tile_components)
            .add_reactor(broadcast::<MapGenerated>(), set_camera_boundary);
    }
}

//-------------------------------------------------------------------------------------------------------------------
