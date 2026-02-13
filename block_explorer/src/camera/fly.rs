//! FlyCamera: WASD movement, arrow keys / trackpad scroll to look around.

use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

pub fn fly_camera_plugin(app: &mut App) {
    app.add_systems(Update, fly_camera_system);
}

const MOVE_SPEED: f32 = 8.0;
const SPRINT_MULTIPLIER: f32 = 3.0;
const KEY_LOOK_SPEED: f32 = 1.5;
const SCROLL_LOOK_SPEED: f32 = 0.03;

fn fly_camera_system(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut scroll_events: EventReader<MouseWheel>,
    mut query: Query<&mut Transform, With<Camera3d>>,
) {
    let Ok(mut transform) = query.get_single_mut() else {
        return;
    };
    let dt = time.delta_secs();

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

    if direction != Vec3::ZERO {
        let speed = if keys.pressed(KeyCode::ShiftLeft) {
            MOVE_SPEED * SPRINT_MULTIPLIER
        } else {
            MOVE_SPEED
        };
        transform.translation += direction.normalize() * speed * dt;
    }
}
