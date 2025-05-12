use std::time::Duration;

use bevy::input::mouse::MouseScrollUnit;
use bevy::picking::hover::HoverMap;
use bevy::picking::pointer::{PointerId, PointerPress};
use bevy::prelude::*;
use bevy::render::camera::NormalizedRenderTarget;
use bevy::window::PrimaryWindow;
use bevy_cobweb::prelude::*;

use super::*;
use crate::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Default, Copy, Clone)]
struct DragState
{
    /// Time from the `Time` resource when the drag began.
    start_time: Duration,
    /// The starting cursor position when the drag began in 2D world coordinates.
    start_pos: Vec2,
    /// The starting cursor position when the drag began in logical pixel coordinates.
    #[allow(dead_code)]
    start_window: Vec2,
    /// The cursor position in the previous frame in 2D world coordinates *calculated from the previous frame*.
    ///
    /// If the camera moved between frames, then this will not point to the correct current-world position of the
    /// previous cursor position.
    ///
    /// Will be `None` if the cursor was outside the window in the previous frame.
    prev_pos: Option<Vec2>,
    /// The cursor position in the previous frame in 2D world coordinates *calculated from the current frame*.
    ///
    /// Will be `None` if the cursor was outside the window in the previous frame.
    prev_pos_corrected: Option<Vec2>,
    /// The cursor position in the previous frame in logical pixel coordinates.
    ///
    /// Will be `None` if the cursor was outside the window in the previous frame.
    prev_window: Option<Vec2>,
    /// The current cursor position in 2D world coordinates.
    ///
    /// Will be `None` if the cursor is outside the window.
    current_pos: Option<Vec2>,
    /// The current cursor position in logical pixel coordinates.
    ///
    /// Will be `None` if the cursor is outside the window.
    current_window: Option<Vec2>,
}

impl DragState
{
    /// Returns the translation of the cursor in 2D world coordinates between the previous frame and this frame.
    ///
    /// This is calculated based on the camera position in the current frame.
    #[allow(dead_code)]
    fn world_delta(&self) -> Vec2
    {
        let current = self.current_pos.unwrap_or_default();
        let prev = self.prev_pos_corrected.unwrap_or_default();

        current - prev
    }

    fn update(&mut self, cursor_pos: CursorPosition)
    {
        self.prev_pos_corrected = cursor_pos.prev_world_pos();
        self.prev_pos = self.current_pos;
        self.prev_window = self.current_window;
        self.current_pos = cursor_pos.world_pos();
        self.current_window = cursor_pos.window_pos();
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Default, Debug, Copy, Clone)]
enum MapDrag
{
    #[default]
    Idle,
    Tracking
    {
        current: DragState, pointer_id: PointerId
    },
    Dragging
    {
        start: DragState, current: DragState, pointer_id: PointerId
    },
}

impl MapDrag
{
    fn pointer_id(&self) -> Option<PointerId>
    {
        match self {
            Self::Idle => None,
            Self::Tracking { pointer_id, .. } | Self::Dragging { pointer_id, .. } => Some(*pointer_id),
        }
    }

    fn get_command(&self) -> Option<CameraCommand>
    {
        match self {
            Self::Idle | Self::Tracking { .. } => None,
            Self::Dragging { start, current, .. } => {
                let window_pos = current.current_window?;
                // When a new 'map drag phase' starts, we start the map drag from the new drag state's previous
                // position. This may differ from the drag state's start position if map drag began
                // at some point after the drag started.
                let target_world_pos = start.prev_pos?;
                Some(CameraCommand::Drag { window_pos, target_world_pos })
            }
        }
    }

    /// Initializes a drag process.
    ///
    /// The map may not be dragged immediately if the cursor is inside its buffer.
    fn initialize(&mut self, pointer_id: PointerId, cursor_pos: CursorPosition, current_time: Duration)
    {
        let current = DragState {
            start_time: current_time,
            start_pos: cursor_pos.world_pos().unwrap(),
            start_window: cursor_pos.window_pos().unwrap(),
            prev_pos: cursor_pos.world_pos(),
            prev_pos_corrected: cursor_pos.world_pos(),
            prev_window: cursor_pos.window_pos(),
            current_pos: cursor_pos.world_pos(),
            current_window: cursor_pos.window_pos(),
        };
        *self = Self::Tracking { current, pointer_id };
    }

    fn update(&mut self, cursor_pos: CursorPosition)
    {
        match self {
            Self::Idle => (),
            Self::Tracking { current, .. } | Self::Dragging { current, .. } => {
                current.update(cursor_pos);
            }
        }
    }

    fn try_start(
        &mut self,
        no_pressed: bool,
        settings: &MapSettings,
        projection_scale: f32,
        time_elapsed: Duration,
    ) -> bool
    {
        match self {
            Self::Idle | Self::Dragging { .. } => false,
            Self::Tracking { current, pointer_id } => {
                let pointer_id = *pointer_id;

                // Begin map drag if the cursor leaves the window.
                let Some(current_pos) = current.current_pos else {
                    //tracing::trace!("starting map drag; cursor outside window");
                    let current = *current;
                    *self = Self::Dragging { start: current, current, pointer_id };
                    return true;
                };

                // If drag has gone outside the drag bounds then we need to end tile press and start map drag.
                let delta = current_pos - current.start_pos;
                let drag_time = time_elapsed.saturating_sub(current.start_time);

                // TODO: add mobile-calibrated cursor buffer settings
                if no_pressed
                    || cursor_outside_buffer(
                        delta.length(),
                        drag_time,
                        settings.cursor_buffer_min * projection_scale,
                        settings.cursor_buffer_start * projection_scale,
                        settings.cursor_buffer_decayrate_secs,
                    )
                {
                    //tracing::trace!("starting map drag; cursor outside buffer {current:?}");
                    let current = *current;
                    *self = Self::Dragging { start: current, current, pointer_id };
                    return true;
                }

                false
            }
        }
    }

    fn end(&mut self)
    {
        *self = Self::Idle;
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Reactive event broadcasted when TilePressed should be removed from tiles.
#[derive(Debug, Copy, Clone)]
struct RemoveTilePressed;

/// Reactive event broadcasted when TileSelected should be removed from tiles.
#[derive(Debug, Copy, Clone)]
struct RemoveTileSelected;

//-------------------------------------------------------------------------------------------------------------------

fn cursor_outside_buffer(
    drag_distance: f32,
    drag_time: Duration,
    min_drag_radius: f32,
    start_drag_radius: f32,
    radius_decayrate_secs: f32,
) -> bool
{
    if drag_distance < min_drag_radius {
        return false;
    }

    // r = r_start / e^(time / rate)
    let mut divisor = std::f32::consts::E.powf(drag_time.as_secs_f32() / radius_decayrate_secs);
    if !divisor.is_normal() {
        divisor = 1.0;
    }
    let decayed_radius = start_drag_radius / divisor;

    if !decayed_radius.is_normal() {
        tracing::error!(decayed_radius, "abnormal decayed max radius");
        return true;
    }

    drag_distance > decayed_radius
}

//-------------------------------------------------------------------------------------------------------------------

fn cleanup_pressed_tile(mut c: Commands, pressed: Query<Entity, With<TilePressed>>)
{
    for pressed in pressed.iter() {
        c.entity(pressed).remove::<TilePressed>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn cleanup_selected_tile(mut c: Commands, pressed: Query<Entity, With<TileSelected>>)
{
    for pressed in pressed.iter() {
        c.entity(pressed).remove::<TileSelected>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn handle_scroll_event(
    event: Trigger<Pointer<Scroll>>,
    mut c: Commands,
    settings: Res<CameraSettings>,
    tiles: Query<(), With<TileType>>,
)
{
    let scroll_event = &event.event().event;
    if scroll_event.y == 0.0 {
        return;
    }
    if !tiles.contains(event.target()) {
        return;
    }

    // Cleanup pressed tile
    c.react().broadcast(RemoveTilePressed);

    // Send zoom command
    let zoom_factor = match scroll_event.unit {
        MouseScrollUnit::Line => settings.line_zoom_factor,
        MouseScrollUnit::Pixel => settings.pixel_zoom_factor,
    };

    if scroll_event.y > 0. {
        c.react()
            .broadcast(CameraCommand::MultiplyZoom { factor: 1. / zoom_factor });
    } else if scroll_event.y < 0. {
        c.react()
            .broadcast(CameraCommand::MultiplyZoom { factor: zoom_factor });
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn handle_press_event(
    event: Trigger<Pointer<Pressed>>,
    mut c: Commands,
    time: Res<Time>,
    window: Query<(), With<PrimaryWindow>>,
    cursor_pos: Res<CursorPosition>,
    mut drag: ResMut<MapDrag>,
    tiles: Query<(), With<TileType>>,
)
{
    // Only presses on tiles in the main window matter.
    let NormalizedRenderTarget::Window(window_ref) = event.event().pointer_location.target else {
        tracing::trace!("map drag: ignoring press event on non-window target");
        return;
    };
    if !window.contains(window_ref.entity()) {
        tracing::trace!("map drag: ignoring press event on non-primary window");
        return;
    }
    if !tiles.contains(event.target()) {
        //tracing::trace!("map drag: ignoring press event on non-tile entity {:?}", event.target());
        return;
    }

    // Sanity check
    if cursor_pos.world_pos().is_none() || cursor_pos.window_pos().is_none() {
        tracing::error!("received Pointer<Pressed> event on primary window but cursor position is None");
        return;
    }

    match event.event().event.button {
        // left press
        PointerButton::Primary => {
            c.entity(event.target()).try_insert(TilePressed);
            drag.initialize(event.event().pointer_id, *cursor_pos, time.elapsed());
        }
        // right press
        PointerButton::Secondary => {
            c.react().broadcast(RemoveTilePressed);
            c.react().broadcast(RemoveTileSelected);
            drag.end();
        }
        PointerButton::Middle => (),
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn handle_click_event(
    event: Trigger<Pointer<Click>>,
    mut c: Commands,
    mut drag: ResMut<MapDrag>,
    pressed: Query<(), With<TilePressed>>,
)
{
    if event.event().event.button != PointerButton::Primary {
        return;
    }
    if !pressed.contains(event.target()) {
        return;
    }

    c.entity(event.target())
        .remove::<TilePressed>()
        .try_insert(TileSelected);
    drag.end();
}

//-------------------------------------------------------------------------------------------------------------------

/// Detects if the pointer that initiated a TilePressed process has become unpressed or unavailable without
/// generating a Click event on the TilePressed entity.
fn detect_press_aborted(
    mut c: Commands,
    pointers: Query<(&PointerId, &PointerPress)>,
    hovermap: Res<HoverMap>,
    mut drag: ResMut<MapDrag>,
    tiles: Query<(), With<TileType>>,
)
{
    let Some(pointer_id) = drag.pointer_id() else { return };

    let hover_intersects_map = hovermap
        .get(&pointer_id)
        // Note: empty if intersecting empty space, which we treat as 'part of the map'
        .map(|i| i.is_empty() || i.iter().any(|(e, _)| tiles.contains(*e)))
        .unwrap_or_default();
    let pointer_primary_pressed = pointers
        .iter()
        .filter(|(id, _)| **id == pointer_id)
        .any(|(_, press)| press.is_primary_pressed());

    // TODO: this probably won't work on touch devices (e.g. mobile) where pointer ids change constantly
    if hover_intersects_map && pointer_primary_pressed {
        return;
    }

    //tracing::trace!("aborting map drag; pointer no longer available");
    c.react().broadcast(RemoveTilePressed);
    drag.end();
}

//-------------------------------------------------------------------------------------------------------------------

fn handle_map_drag(
    mut c: Commands,
    time: Res<Time>,
    cursor: Res<CursorPosition>,
    mut drag: ResMut<MapDrag>,
    cameras: Query<&Projection, With<MainCamera>>,
    settings: Res<MapSettings>,
    pressed: Query<(), With<TilePressed>>,
)
{
    let Projection::Orthographic(projection) = cameras.single().unwrap() else { return };

    // Update drag state.
    drag.update(*cursor);

    // Try to start map drag.
    if drag.try_start(pressed.is_empty(), &settings, projection.scale, time.elapsed()) {
        c.react().broadcast(RemoveTilePressed);

        // TODO: If on mobile, also remove TileSelected so the tile info card will go away while moving around the
        // map.
    }

    // Apply current drag state to camera if map drag has started.
    if let Some(command) = drag.get_command() {
        c.react().broadcast(command);
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) struct MapControlPlugin;

impl Plugin for MapControlPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<MapDrag>()
            .add_reactor(broadcast::<RemoveTilePressed>(), cleanup_pressed_tile)
            .add_reactor(broadcast::<RemoveTileSelected>(), cleanup_selected_tile)
            .add_observer(handle_scroll_event)
            .add_observer(handle_press_event)
            .add_observer(handle_click_event)
            .add_systems(Update, (detect_press_aborted, handle_map_drag).chain());
    }
}

//-------------------------------------------------------------------------------------------------------------------
