//! Shared material and color helpers for slabs and tx cubes.

use crate::data::TxPayload;
use bevy::prelude::*;

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

pub fn tx_cube_material(
    materials: &mut ResMut<Assets<StandardMaterial>>,
    tx: &TxPayload,
) -> Handle<StandardMaterial> {
    let gwei = tx.gas_price as f64 / 1e9;
    let color = gas_price_color(gwei);

    let emissive = if tx.value_eth > 1.0 {
        let lin = color.to_linear();
        Color::linear_rgb(lin.red * 5.0, lin.green * 5.0, lin.blue * 5.0)
    } else {
        Color::BLACK
    };

    materials.add(StandardMaterial {
        base_color: color,
        emissive: emissive.into(),
        ..default()
    })
}

/// Blue → Cyan → Yellow → Red gradient mapped to 0–200 gwei.
fn gas_price_color(gwei: f64) -> Color {
    let t = (gwei / 200.0).clamp(0.0, 1.0) as f32;

    if t < 0.33 {
        let s = t / 0.33;
        Color::srgb(0.0, s, 1.0 - s * 0.5)
    } else if t < 0.66 {
        let s = (t - 0.33) / 0.33;
        Color::srgb(s, 1.0, 0.5 * (1.0 - s))
    } else {
        let s = (t - 0.66) / 0.34;
        Color::srgb(1.0, 1.0 - s, 0.0)
    }
}
