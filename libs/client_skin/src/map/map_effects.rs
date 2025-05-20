use bevy::prelude::*;
use bevy_aseprite_ultra::prelude::AseSlice;
use utils_gui::AsepriteMap;

use super::*;
use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn handle_new_tile_pressed(
    event: Trigger<OnAdd, TilePressed>,
    map_settings: Res<MapSettings>,
    mut tiles: Query<(&mut Sprite, Option<&AttachedMeta>), With<MapTile>>,
    mut attached: Query<&mut Sprite, Without<MapTile>>,
)
{
    let Ok((mut sprite, maybe_attachment)) = tiles.get_mut(event.target()) else { return };
    sprite.color = map_settings.press_color;

    let Some(attachment) = maybe_attachment else { return };
    let Ok(mut sprite) = attached.get_mut(**attachment) else { return };
    sprite.color = map_settings.press_color;
}

//-------------------------------------------------------------------------------------------------------------------

fn handle_tile_unpressed(
    event: Trigger<OnRemove, TilePressed>,
    mut tiles: Query<(&mut Sprite, Option<&AttachedMeta>), With<MapTile>>,
    mut attached: Query<&mut Sprite, Without<MapTile>>,
)
{
    let Ok((mut sprite, maybe_attachment)) = tiles.get_mut(event.target()) else { return };
    sprite.color = Color::WHITE;

    let Some(attachment) = maybe_attachment else { return };
    let Ok(mut sprite) = attached.get_mut(**attachment) else { return };
    sprite.color = Color::WHITE;
}

//-------------------------------------------------------------------------------------------------------------------

fn handle_new_tile_selected(
    event: Trigger<OnAdd, TileSelected>,
    mut c: Commands,
    aseprites: Res<AsepriteMap>,
    grid: Res<HexGrid>,
    map_settings: Res<MapSettings>,
    tiles: Query<(), With<MapTile>>,
)
{
    debug_assert!(tiles.contains(event.target()));

    let aseprite = aseprites.get(&map_settings.tile_aseprite);
    let sprite_size = grid.layout.rect_size();

    c.spawn((
        TileSelectedEffect,
        ChildOf(event.target()),
        SpriteLayer::TileSelectEffect,
        AseSlice {
            aseprite: aseprite.clone(),
            name: map_settings.select_effect_slice.clone(),
        },
        Sprite { custom_size: Some(sprite_size), ..default() },
    ));
}

//-------------------------------------------------------------------------------------------------------------------

fn handle_tile_unselected(
    event: Trigger<OnRemove, TileSelected>,
    mut c: Commands,
    effects: Query<(Entity, &ChildOf), With<TileSelectedEffect>>,
)
{
    for (entity, childof) in effects.iter() {
        if childof.parent() != event.target() {
            continue;
        }
        c.entity(entity).despawn();
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Marker component for effect entities attached to TileSelected entities.
#[derive(Component)]
struct TileSelectedEffect;

//-------------------------------------------------------------------------------------------------------------------

pub(super) struct MapEffectsPlugin;

impl Plugin for MapEffectsPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_observer(handle_new_tile_pressed)
            .add_observer(handle_tile_unpressed)
            .add_observer(handle_new_tile_selected)
            .add_observer(handle_tile_unselected);
    }
}

//-------------------------------------------------------------------------------------------------------------------
