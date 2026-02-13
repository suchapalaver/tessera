//! FlyCamera component and system: WASD + mouse look.

use bevy::prelude::*;

pub fn fly_camera_plugin(app: &mut App) {
    app.add_systems(Update, fly_camera_system);
}

fn fly_camera_system(_query: Query<&Transform, With<Camera3d>>) {
    // TODO: apply FlyCamera component to main camera, move/rotate from input
}
