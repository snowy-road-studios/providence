use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowResized};
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use bevy_girk_client_fw::ClientFwState;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Applies zoom to a camera.
fn apply_zoom(
    settings: &CameraSettings,
    upper_right: &mut Vec2,
    lower_left: &mut Vec2,
    scale: &mut f32,
    factor: f32,
)
{
    let factor = factor.clamp(
        settings.zoom_range.0 / scale.max(0.01),
        settings.zoom_range.1 / scale.max(0.01),
    );

    let current_width = upper_right.x - lower_left.x;
    let current_height = upper_right.y - lower_left.y;
    *scale *= factor;
    let adj = Vec2 {
        x: current_width * (factor - 1.) / 2.,
        y: current_height * (factor - 1.) / 2.,
    };
    *lower_left -= adj;
    *upper_right += adj;
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Default)]
struct CameraControl
{
    /// The current zoom target in 2D world coordinates.
    zoom_target: Option<Vec2>,
}

/// Applies camera commands while taking into account control limits.
fn apply_camera_command(
    mut control: Local<CameraControl>,
    command: BroadcastEvent<CameraCommand>,
    mut cameras: Query<(&Camera, &GlobalTransform, &mut Transform, &mut Projection), With<MainCamera>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    boundary: Res<CameraBoundary>,
    settings: Res<CameraSettings>,
)
{
    let Ok((camera, cam_global, mut cam_transform, cam_projection)) = cameras.single_mut() else {
        tracing::error!("unexpectedly not exactly one MainCamera in camera commands");
        return;
    };
    let Projection::Orthographic(cam_projection) = cam_projection.into_inner() else {
        tracing::error!("orthographic camera missing in camera commands");
        return;
    };

    let Ok(window) = windows.single() else {
        tracing::error!("window missing in camera commands");
        return;
    };

    // get starting position of camera
    let window_dims = Vec2 { x: window.width().max(0.01), y: window.height().max(0.01) };

    let mut translation = cam_transform.translation;
    let mut scale = cam_projection.scale;
    let mut lower_left = camera
        .viewport_to_world_2d(cam_global, Vec2 { x: 0., y: window_dims.y })
        .unwrap();
    let mut upper_right = camera
        .viewport_to_world_2d(cam_global, Vec2 { x: window_dims.x, y: 0. })
        .unwrap();

    let mut zoom_target = control.zoom_target.unwrap_or_else(|| {
        // start at the center of the screen
        // - note: this assumes no correction is needed in the first frame; if the first frame corrects then this
        //   could theoretically cause unintended shifting
        Vec2 {
            x: (upper_right.x + lower_left.x) * 0.5,
            y: (upper_right.y + lower_left.y) * 0.5,
        }
    });
    let prev_zoom_target = zoom_target;

    // Handle the command.
    let mut drag_adjustment = None;
    let camera_command = *command
        .try_read()
        .expect("system should be tied to BroadcastEvent<CameraCommand>");
    match camera_command {
        CameraCommand::WindowResize => (),
        CameraCommand::Center { focus_point } => {
            let current_width = upper_right.x - lower_left.x;
            let current_height = upper_right.y - lower_left.y;
            translation.x = focus_point.x;
            translation.y = focus_point.y;
            lower_left = Vec2 {
                x: focus_point.x - (current_width / 2.),
                y: focus_point.y - (current_height / 2.),
            };
            upper_right = Vec2 {
                x: focus_point.x + (current_width / 2.),
                y: focus_point.y + (current_height / 2.),
            };
            zoom_target = focus_point;
        }
        CameraCommand::SetZoom { zoom } => {
            let factor = zoom / scale.max(0.01);
            apply_zoom(&settings, &mut upper_right, &mut lower_left, &mut scale, factor);
        }
        CameraCommand::MultiplyZoom { factor } => {
            apply_zoom(&settings, &mut upper_right, &mut lower_left, &mut scale, factor);
        }
        CameraCommand::Drag { window_pos, target_world_pos } => {
            let current_width = upper_right.x - lower_left.x;
            let current_height = upper_right.y - lower_left.y;

            // translate world position in adjusted camera view
            let mut normalized_window_pos = window_pos;
            normalized_window_pos.x /= window_dims.x;
            normalized_window_pos.y /= window_dims.y;
            if !normalized_window_pos.x.is_normal() {
                normalized_window_pos.x = 0.
            }
            if !normalized_window_pos.y.is_normal() {
                normalized_window_pos.y = 0.
            }

            let translated_window_pos = Vec2 {
                x: lower_left.x + current_width * normalized_window_pos.x,
                y: lower_left.y + current_height * (1. - normalized_window_pos.y),
            };

            // translate from target world position to translated end position
            let adj = target_world_pos - translated_window_pos;
            translation.x += adj.x;
            translation.y += adj.y;
            lower_left += adj;
            upper_right += adj;
            zoom_target = translation.truncate();
            drag_adjustment = Some(adj);
        }
    }

    // apply zoom target
    // - this undoes translation-corrections from the previous frame
    let target_diff = zoom_target - translation.truncate();
    translation.x = zoom_target.x;
    translation.y = zoom_target.y;
    lower_left += target_diff;
    upper_right += target_diff;

    // apply corrections
    let max_width = boundary.upper_right.x - boundary.lower_left.x;
    let max_height = boundary.upper_right.y - boundary.lower_left.y;

    // too wide
    let current_width = (upper_right.x - lower_left.x).max(0.01);

    if current_width > max_width {
        // translate to midpoint
        lower_left.x -= translation.x;
        upper_right.x -= translation.x;
        translation.x = 0.;

        // add artificial zoom-in
        let factor = max_width / current_width;
        apply_zoom(&settings, &mut upper_right, &mut lower_left, &mut scale, factor);

        // repair: vertical position of drag target needs to be maintained
        // TODO: this never runs because drag_adjustment isn only set for drag commands
        // TODO: what about SetZoom?
        if let (CameraCommand::MultiplyZoom { factor: zoom }, Some(adj)) = (camera_command, drag_adjustment) {
            let vertical_correction = adj.y * factor * zoom;
            zoom_target.y -= vertical_correction;
            translation.y -= vertical_correction;
            lower_left.y -= vertical_correction;
            upper_right.y -= vertical_correction;
        }
    }

    // too tall
    let current_height = (upper_right.y - lower_left.y).max(0.01);

    if current_height > max_height {
        // translate to midpoint
        upper_right.y -= translation.y;
        lower_left.y -= translation.y;
        translation.y = 0.;

        // add artificial zoom-in
        let factor = max_height / current_height;
        apply_zoom(&settings, &mut upper_right, &mut lower_left, &mut scale, factor);

        // repair: horizontal position of zoom target needs to be maintained
        // - if this 'cancels' the too-wide x-displacement, it will be fixed by side-constraints below
        // TODO: this never runs because drag_adjustment isn only set for drag commands
        // TODO: what about SetZoom?
        if let (CameraCommand::MultiplyZoom { factor: zoom }, Some(adj)) = (camera_command, drag_adjustment) {
            let horizontal_correction = adj.x * factor * zoom;
            zoom_target.x -= horizontal_correction;
            translation.x -= horizontal_correction;
            lower_left.x -= horizontal_correction;
            upper_right.x -= horizontal_correction;
        }
    }

    // left side
    if lower_left.x < boundary.lower_left.x {
        // translate to the right
        let diff = boundary.lower_left.x - lower_left.x;
        lower_left.x += diff;
        upper_right.x += diff;
        translation.x += diff;
    }

    // right side
    if upper_right.x > boundary.upper_right.x {
        // translate to the left
        let diff = upper_right.x - boundary.upper_right.x;
        lower_left.x -= diff;
        upper_right.x -= diff;
        translation.x -= diff;
    }

    // top
    if upper_right.y > boundary.upper_right.y {
        // translate down
        let diff = upper_right.y - boundary.upper_right.y;
        upper_right.y -= diff;
        lower_left.y -= diff;
        translation.y -= diff;
    }

    // bottom
    if lower_left.y < boundary.lower_left.y {
        // translate up
        let diff = boundary.lower_left.y - lower_left.y;
        upper_right.y += diff;
        lower_left.y += diff;
        translation.y += diff;
    }

    // if there were changes to the zoom target, refresh it post-corrections
    // - this avoids situations where the zoom target doesn't match visual feedback from drags that run into
    //   boundaries
    if prev_zoom_target != zoom_target {
        zoom_target.x = translation.x;
        zoom_target.y = translation.y;
    }
    control.zoom_target = Some(zoom_target);

    // update camera
    cam_transform.translation = translation;
    cam_projection.scale = scale;
}

//-------------------------------------------------------------------------------------------------------------------

fn check_window_resize(mut resize_events: EventReader<WindowResized>, mut c: Commands)
{
    if resize_events.read().count() == 0 {
        return;
    }
    c.react().broadcast(CameraCommand::WindowResize);
}

//-------------------------------------------------------------------------------------------------------------------

/// Reactive event broadcasted to update the camera.
#[derive(Debug, Copy, Clone)]
pub(crate) enum CameraCommand
{
    /// Correct the camera after the window was resized.
    WindowResize,
    /// Set the focus-point of the camera in 2D world coordinates.
    #[allow(dead_code)]
    Center
    {
        focus_point: Vec2
    },
    /// Drag the camera so `target_world_pos` lines up with `window_pos`.
    Drag
    {
        /// The position on the screen in pixels where the drag should end.
        window_pos: Vec2,
        /// The world position that should be moved to the target screen position.
        target_world_pos: Vec2,
    },
    /// Set camera zoom to the given value.
    #[allow(dead_code)]
    SetZoom
    {
        zoom: f32
    },
    /// Multiply camera zoom by the given factor.
    MultiplyZoom
    {
        factor: f32
    },
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Default, Debug, Reflect, PartialEq)]
pub(crate) struct CameraSettings
{
    /// Range of allowed zoom scales.
    pub(crate) zoom_range: (f32, f32),
    /// Proportion of current zoom scale to change the scale for each line-based scroll event.
    pub(crate) line_zoom_factor: f32,
    /// Proportion of current zoom scale to change the scale for each pixel-based scroll event.
    pub(crate) pixel_zoom_factor: f32,
}

impl Command for CameraSettings
{
    fn apply(self, w: &mut World)
    {
        w.insert_resource(self);
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Debug, Default)]
pub(crate) struct CameraBoundary
{
    pub(crate) upper_right: Vec2,
    pub(crate) lower_left: Vec2,
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) struct CameraControlPlugin;

impl Plugin for CameraControlPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<CameraBoundary>()
            .init_resource::<CameraSettings>()
            .register_command_type::<CameraSettings>()
            .add_reactor(broadcast::<CameraCommand>(), apply_camera_command)
            .add_systems(
                Update,
                check_window_resize
                    .in_set(ClientLogicSet::Admin)
                    .run_if(in_state(ClientFwState::Game)),
            );
    }
}

//-------------------------------------------------------------------------------------------------------------------
