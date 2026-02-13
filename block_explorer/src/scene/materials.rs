//! Shared material and color helpers for slabs and tx cubes.

use bevy::prelude::*;

pub fn block_slab_material(
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Handle<StandardMaterial> {
    block_slab_material_with_fullness(materials, 0.5)
}

pub fn block_slab_material_with_fullness(
    materials: &mut ResMut<Assets<StandardMaterial>>,
    fullness: f32,
) -> Handle<StandardMaterial> {
    let g = 0.2 + 0.5 * fullness;
    materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, g, 0.3),
        ..default()
    })
}
