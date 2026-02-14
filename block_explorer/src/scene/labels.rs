//! Floating block-number labels above each slab.

use bevy::prelude::*;

/// Marker for label entities.
#[derive(Component)]
pub struct BlockLabel;

const LABEL_Y: f32 = 2.0;
const CULL_DISTANCE: f32 = 80.0;

/// Spawns a `Text2d` label above a slab at the given Z position.
pub fn spawn_block_label(commands: &mut Commands, block_number: u64, slab_z: f32) {
    commands.spawn((
        BlockLabel,
        Text2d::new(format!("#{block_number}")),
        TextFont {
            font_size: 36.0,
            ..default()
        },
        TextColor(Color::srgba(0.6, 0.9, 0.75, 0.85)),
        TextLayout::new_with_justify(JustifyText::Center),
        Transform::from_xyz(0.0, LABEL_Y, slab_z).with_scale(Vec3::splat(0.02)),
    ));
}

/// Rotates every `BlockLabel` to face the camera each frame.
pub fn billboard_labels_system(
    camera_query: Query<&Transform, With<Camera3d>>,
    mut labels: Query<&mut Transform, (With<BlockLabel>, Without<Camera3d>)>,
) {
    let Ok(cam_tf) = camera_query.get_single() else {
        return;
    };
    let cam_pos = cam_tf.translation;
    for mut tf in &mut labels {
        tf.look_at(cam_pos, Vec3::Y);
    }
}

/// Hides labels beyond `CULL_DISTANCE` from the camera.
#[allow(clippy::type_complexity)]
pub fn label_distance_cull_system(
    camera_query: Query<&Transform, With<Camera3d>>,
    mut labels: Query<(&Transform, &mut Visibility), (With<BlockLabel>, Without<Camera3d>)>,
) {
    let Ok(cam_tf) = camera_query.get_single() else {
        return;
    };
    let cam_pos = cam_tf.translation;
    for (tf, mut vis) in &mut labels {
        *vis = if tf.translation.distance(cam_pos) > CULL_DISTANCE {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }
}
