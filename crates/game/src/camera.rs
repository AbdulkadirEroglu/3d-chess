//! @file camera.rs
//! @brief Orbit camera around the board: hold the right mouse button and drag to rotate, scroll to zoom.

use std::f32::consts::PI;

use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;
use chess_core::Color as Side;

use crate::menu::MatchSettings;
use crate::turn::GameStarted;

/// @brief Point the camera orbits around and looks at: the board center.
const FOCUS: Vec3 = Vec3::ZERO;

/// @brief Starting pitch in radians (about 50° above the board).
const START_PITCH: f32 = 0.87;

/// @brief Starting distance from the board center.
const START_DISTANCE: f32 = 12.5;

/// @brief Pitch limits: never below the board, never straight down.
const PITCH_RANGE: (f32, f32) = (0.2, 1.45);

/// @brief Zoom limits.
const DISTANCE_RANGE: (f32, f32) = (6.0, 25.0);

/// @brief Radians of rotation per pixel of mouse movement.
const ROTATE_SPEED: f32 = 0.005;

/// @brief Distance change per scroll line.
const ZOOM_SPEED: f32 = 1.0;

/// @brief Pixel-based scrolling (touchpads) counts this many pixels as one line.
const PIXELS_PER_LINE: f32 = 40.0;

/// @brief Background color behind the scene.
const BACKGROUND_COLOR: Color = Color::srgb(0.08, 0.08, 0.10);

/// @brief Spawns the camera and runs the orbit controls.
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(BACKGROUND_COLOR))
            .add_systems(Startup, spawn_camera)
            .add_systems(Update, (orbit_camera, face_player_side));
    }
}

/// @brief Orbit state: angles around the board center and distance from it.
#[derive(Component)]
struct OrbitCamera {
    /// @brief Rotation around the vertical axis, in radians. 0 = behind White.
    yaw: f32,
    /// @brief Angle above the board, in radians.
    pitch: f32,
    /// @brief Distance from the board center.
    distance: f32,
}

/// @brief Camera transform for an orbit state, looking at the board center.
fn orbit_transform(orbit: &OrbitCamera) -> Transform {
    let rotation = Quat::from_euler(EulerRot::YXZ, orbit.yaw, -orbit.pitch, 0.0);
    let position = FOCUS + rotation * Vec3::Z * orbit.distance;
    Transform::from_translation(position).looking_at(FOCUS, Vec3::Y)
}

/// @brief Spawns the camera behind White, looking down at the board.
fn spawn_camera(mut commands: Commands) {
    let orbit = OrbitCamera {
        yaw: 0.0,
        pitch: START_PITCH,
        distance: START_DISTANCE,
    };
    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        orbit_transform(&orbit),
        orbit,
    ));
    debug!("camera spawned");
}

/// @brief When a game starts, turns the camera to the human's side: Black's side when playing Black vs the AI.
fn face_player_side(
    mut started: MessageReader<GameStarted>,
    settings: Res<MatchSettings>,
    mut cameras: Query<(&mut OrbitCamera, &mut Transform)>,
) {
    if started.read().count() == 0 {
        return;
    }
    let yaw = if settings.vs_ai && settings.human_side == Side::Black {
        PI
    } else {
        0.0
    };
    for (mut orbit, mut transform) in &mut cameras {
        orbit.yaw = yaw;
        *transform = orbit_transform(&orbit);
    }
    debug!(yaw, "camera faces the player's side");
}

/// @brief Rotates the camera while the right mouse button is held, and zooms on scroll.
fn orbit_camera(
    buttons: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    mut cameras: Query<(&mut OrbitCamera, &mut Transform)>,
) {
    let rotate = if buttons.pressed(MouseButton::Right) {
        motion.delta
    } else {
        Vec2::ZERO
    };
    let zoom_lines = match scroll.unit {
        MouseScrollUnit::Line => scroll.delta.y,
        MouseScrollUnit::Pixel => scroll.delta.y / PIXELS_PER_LINE,
    };
    // Nothing to do on most frames; skip so the transform isn't marked changed.
    if rotate == Vec2::ZERO && zoom_lines == 0.0 {
        return;
    }
    trace!(?rotate, zoom_lines, "camera orbit input");

    for (mut orbit, mut transform) in &mut cameras {
        orbit.yaw -= rotate.x * ROTATE_SPEED;
        orbit.pitch = (orbit.pitch + rotate.y * ROTATE_SPEED).clamp(PITCH_RANGE.0, PITCH_RANGE.1);
        orbit.distance =
            (orbit.distance - zoom_lines * ZOOM_SPEED).clamp(DISTANCE_RANGE.0, DISTANCE_RANGE.1);
        *transform = orbit_transform(&orbit);
    }
}
