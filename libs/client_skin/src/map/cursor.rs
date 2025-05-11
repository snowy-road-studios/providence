use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Updates the cursor position.
fn update_cursor_position(
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut cursor_pos: ResMut<CursorPosition>,
) -> Result
{
    // Current cursor position
    let window = windows.single()?;
    let window_pos = window.cursor_position();

    // Previous cursor position
    let prev_window_pos = cursor_pos.window_pos();

    // Get current world position of current cursor and previous cursor
    let (camera, cam_transform) = cameras.single()?;
    let world_pos = window_pos.and_then(|p| camera.viewport_to_world_2d(cam_transform, p).ok());
    let prev_world_pos = prev_window_pos.and_then(|p| camera.viewport_to_world_2d(cam_transform, p).ok());

    cursor_pos.set(window_pos, world_pos, prev_world_pos);

    Ok(())
}

//-------------------------------------------------------------------------------------------------------------------

/// Records the position of the cursor in the current frame.
#[derive(Resource, Debug, Default, Copy, Clone)]
pub(crate) struct CursorPosition
{
    window_pos: Option<Vec2>,
    world_pos: Option<Vec2>,

    prev_window_pos: Option<Vec2>,
    /// World position of previous cursor position *computed using the current frame's camera*.
    prev_world_pos: Option<Vec2>,
}

impl CursorPosition
{
    /// Sets the cursor position.
    fn set(&mut self, window: Option<Vec2>, world: Option<Vec2>, prev_world: Option<Vec2>)
    {
        self.prev_window_pos = self.window_pos;
        self.prev_world_pos = prev_world;
        self.window_pos = window;
        self.world_pos = world;
    }

    /// Gets the cursor position in the window frame in logical pixels.
    pub(crate) fn window_pos(&self) -> Option<Vec2>
    {
        self.window_pos
    }

    /// Gets the cursor position in the XY plane (only valid if the primary camera is 2D).
    pub(crate) fn world_pos(&self) -> Option<Vec2>
    {
        self.world_pos
    }

    /// Gets the cursor position of the previous frame in the window frame in logical pixels.
    #[allow(dead_code)]
    pub(crate) fn prev_window_pos(&self) -> Option<Vec2>
    {
        self.prev_window_pos
    }

    /// Gets the cursor position of the previous frame in the XY plane (only valid if the primary camera is 2D).
    ///
    /// This is calculated within the camera viewport of the current frame.
    /// It is intended to allow calculating the magnitude of world-space translation of the cursor on the screen.
    /// This is useful for calculating the velocity of cursor movement.
    pub(crate) fn prev_world_pos(&self) -> Option<Vec2>
    {
        self.prev_world_pos
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) struct CursorPlugin;

impl Plugin for CursorPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<CursorPosition>()
            .add_systems(First, update_cursor_position);
    }
}

//-------------------------------------------------------------------------------------------------------------------
