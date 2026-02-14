//! FlyCamera: WASD movement, arrow keys / trackpad scroll to look around.

use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

/// Optional target for animated camera jumps (set by timeline, cleared on arrival or WASD).
#[derive(Resource, Default)]
pub struct CameraTarget {
    pub target: Option<Vec3>,
    pub look_at: Option<Vec3>,
}

pub fn fly_camera_plugin(app: &mut App) {
    app.init_resource::<CameraTarget>()
        .add_systems(Update, fly_camera_system);
}

const MOVE_SPEED: f32 = 8.0;
const SPRINT_MULTIPLIER: f32 = 3.0;
const KEY_LOOK_SPEED: f32 = 1.5;
const SCROLL_LOOK_SPEED: f32 = 0.03;
const CAMERA_LERP_SPEED: f32 = 4.0;

fn fly_camera_system(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut scroll_events: EventReader<MouseWheel>,
    mut query: Query<&mut Transform, With<Camera3d>>,
    mut camera_target: ResMut<CameraTarget>,
) {
    let Ok(mut transform) = query.get_single_mut() else {
        return;
    };
    let dt = time.delta_secs();

    // WASD input cancels any animated camera target
    let wasd_pressed = keys.pressed(KeyCode::KeyW)
        || keys.pressed(KeyCode::KeyA)
        || keys.pressed(KeyCode::KeyS)
        || keys.pressed(KeyCode::KeyD)
        || keys.pressed(KeyCode::KeyQ)
        || keys.pressed(KeyCode::KeyE);

    if wasd_pressed {
        camera_target.target = None;
        camera_target.look_at = None;
    }

    // Animated camera jump toward target
    if let Some(target_pos) = camera_target.target {
        let t = (CAMERA_LERP_SPEED * dt).min(1.0);
        transform.translation = transform.translation.lerp(target_pos, t);

        if let Some(look_at_pos) = camera_target.look_at {
            let desired =
                Transform::from_translation(transform.translation).looking_at(look_at_pos, Vec3::Y);
            transform.rotation = transform.rotation.slerp(desired.rotation, t);
        }

        if transform.translation.distance(target_pos) < 0.1 {
            camera_target.target = None;
            camera_target.look_at = None;
        }
        return;
    }

    // --- Look: arrow keys ---
    let mut yaw = 0.0_f32;
    let mut pitch = 0.0_f32;

    if keys.pressed(KeyCode::ArrowLeft) {
        yaw += KEY_LOOK_SPEED * dt;
    }
    if keys.pressed(KeyCode::ArrowRight) {
        yaw -= KEY_LOOK_SPEED * dt;
    }
    if keys.pressed(KeyCode::ArrowUp) {
        pitch += KEY_LOOK_SPEED * dt;
    }
    if keys.pressed(KeyCode::ArrowDown) {
        pitch -= KEY_LOOK_SPEED * dt;
    }

    // --- Look: trackpad / mouse scroll ---
    for event in scroll_events.read() {
        yaw -= event.x * SCROLL_LOOK_SPEED;
        pitch += event.y * SCROLL_LOOK_SPEED;
    }

    if yaw != 0.0 || pitch != 0.0 {
        let (current_yaw, current_pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
        let new_pitch = (current_pitch + pitch).clamp(-1.5_f32, 1.5_f32);
        transform.rotation = Quat::from_euler(EulerRot::YXZ, current_yaw + yaw, new_pitch, 0.0);
    }

    // --- Movement: WASD + Q/E ---
    let mut direction = Vec3::ZERO;
    let forward = transform.forward();
    let right = transform.right();

    if keys.pressed(KeyCode::KeyW) {
        direction += *forward;
    }
    if keys.pressed(KeyCode::KeyS) {
        direction -= *forward;
    }
    if keys.pressed(KeyCode::KeyA) {
        direction -= *right;
    }
    if keys.pressed(KeyCode::KeyD) {
        direction += *right;
    }
    if keys.pressed(KeyCode::KeyQ) {
        direction += Vec3::Y;
    }
    if keys.pressed(KeyCode::KeyE) {
        direction -= Vec3::Y;
    }

    // --- Reset: Home or Space resets camera to start ---
    if keys.just_pressed(KeyCode::Home) || keys.just_pressed(KeyCode::Space) {
        *transform = Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y);
        return;
    }

    if direction != Vec3::ZERO {
        let speed = if keys.pressed(KeyCode::ShiftLeft) {
            MOVE_SPEED * SPRINT_MULTIPLIER
        } else {
            MOVE_SPEED
        };
        transform.translation += direction.normalize() * speed * dt;
    }
}
