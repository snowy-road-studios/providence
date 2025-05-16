use bevy::prelude::*;
use bevy_aseprite_ultra::prelude::AseSlice;
use bevy_cobweb::prelude::*;
use client_core::MapGenerated;
use game_core::*;
use hexx::Hex;
use utils_gui::AsepriteMap;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Use separate observer for UI tiles which are spawned ad hoc and in low quantities.
fn update_ui_tile(
    event: Trigger<OnAdd, UiTile>,
    mut c: Commands,
    aseprites: Res<AsepriteMap>,
    settings: Res<MapSettings>,
    tiles: Query<&TileId>,
)
{
    let aseprite = aseprites.get(&settings.tile_aseprite);

    let Ok(tile) = tiles.get(event.target()) else {
        tracing::error!("UI tile missing TileType on add; insert TileType before UiTile component");
        return;
    };
    let Ok(mut ec) = c.get_entity(event.target()) else { return };
    ec.insert((
        AseSlice {
            aseprite,
            name: settings.tiles.get(tile).unwrap().aseprite_tag.clone(),
        },
        ImageNode::default(),
    ));
}

//-------------------------------------------------------------------------------------------------------------------

fn update_map_tiles(
    mut c: Commands,
    aseprites: Res<AsepriteMap>,
    grid: Res<HexGrid>,
    settings: Res<MapSettings>,
    tiles: Query<(Entity, &TileId), With<MapTile>>,
)
{
    let aseprite = aseprites.get(&settings.tile_aseprite);
    let sprite_size = grid.layout.rect_size();

    for (entity, tile_id) in tiles.iter() {
        let Ok(mut ec) = c.get_entity(entity) else { continue };
        let Some(info) = settings.tiles.get(tile_id) else { continue };
        ec.insert((
            AseSlice { aseprite: aseprite.clone(), name: info.aseprite_tag.clone() },
            Sprite { custom_size: Some(sprite_size), ..default() },
            Pickable::default(),
        ));
    }
}

//-------------------------------------------------------------------------------------------------------------------

// Includes an offset upwards by half a hex so more edge tiles show at top than bottom.
fn set_camera_boundary(mut c: Commands, grid: Res<HexGrid>)
{
    let mut upper_right = grid.layout.hex_to_world_pos(Hex {
        x: grid.dimension,
        y: (grid.dimension / 2) + (grid.dimension % 2),
    });
    upper_right.y += grid.layout.rect_size().y / 2.0;

    let mut lower_left = grid
        .layout
        .hex_to_world_pos(Hex { x: -grid.dimension, y: -grid.dimension / 2 });
    lower_left.y += grid.layout.rect_size().y / 2.0;

    c.insert_resource(CameraBoundary { upper_right, lower_left });
}

//-------------------------------------------------------------------------------------------------------------------

/// Marker component for tiles rendered in UI.
#[derive(Component, Debug)]
pub(crate) struct UiTile;

//-------------------------------------------------------------------------------------------------------------------

pub(super) struct MapgenPlugin;

impl Plugin for MapgenPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_reactor(broadcast::<MapGenerated>(), update_map_tiles)
            .add_reactor(broadcast::<MapGenerated>(), set_camera_boundary)
            .add_observer(update_ui_tile);
    }
}

//-------------------------------------------------------------------------------------------------------------------
