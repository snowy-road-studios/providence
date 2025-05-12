use bevy::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

fn handle_new_tile_pressed(
    event: Trigger<OnAdd, TilePressed>,
    mut c: Commands,
    pressed: Query<Entity, With<TilePressed>>,
)
{
    for entity in pressed.iter().filter(|e| *e != event.target()) {
        c.entity(entity).remove::<TilePressed>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn handle_new_tile_selected(
    event: Trigger<OnAdd, TileSelected>,
    mut c: Commands,
    selected: Query<Entity, With<TileSelected>>,
)
{
    for entity in selected.iter().filter(|e| *e != event.target()) {
        c.entity(entity).remove::<TileSelected>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Marker component for tiles that have been pressed but not selected.
///
/// Only one tile may be pressed at a time.
#[derive(Component, Debug, Copy, Clone)]
#[component(immutable)]
pub(crate) struct TilePressed;

/// Marker component for tiles that have been selected.
///
/// Only one tile may be selected at a time.
#[derive(Component, Debug)]
#[component(immutable)]
pub(crate) struct TileSelected;

//-------------------------------------------------------------------------------------------------------------------

pub(super) struct TileStatesPlugin;

impl Plugin for TileStatesPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_observer(handle_new_tile_pressed)
            .add_observer(handle_new_tile_selected);
    }
}

//-------------------------------------------------------------------------------------------------------------------
