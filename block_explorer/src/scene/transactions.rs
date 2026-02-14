//! Transaction cubes: grid layout, gas-price color, TxCube component.

use crate::data::{BlockPayload, TxPayload};
use bevy::prelude::*;

use super::materials;

#[derive(Component)]
pub struct TxCube {
    pub hash: Option<String>,
    pub gas: u64,
    pub gas_price: u64,
    pub value_eth: f64,
    pub from: Option<String>,
    pub to: Option<String>,
}

const GRID_SPACING: f32 = 0.25;
const CUBE_BASE: f32 = 0.2;
const MIN_HEIGHT: f32 = 0.1;
const MAX_HEIGHT: f32 = 0.6;
const SLAB_HEIGHT: f32 = 1.0;
const SLAB_DEPTH: f32 = 2.0;

pub fn spawn_tx_cubes(
    commands: &mut Commands,
    payload: &BlockPayload,
    z: f32,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials_res: &mut ResMut<Assets<StandardMaterial>>,
) {
    if payload.transactions.is_empty() {
        return;
    }

    let fullness = if payload.gas_limit > 0 {
        payload.gas_used as f32 / payload.gas_limit as f32
    } else {
        0.0
    };
    let slab_width = 2.0 + 10.0 * fullness;
    let positions = grid_positions(payload.transactions.len(), slab_width, SLAB_DEPTH);

    for (i, pos) in positions.iter().enumerate() {
        let tx = &payload.transactions[i];
        let height = tx_height(tx);
        let y = SLAB_HEIGHT / 2.0 + height / 2.0;
        let material = materials::tx_cube_material(materials_res, tx);

        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(CUBE_BASE, height, CUBE_BASE))),
            MeshMaterial3d(material),
            Transform::from_xyz(pos.0, y, z + pos.1),
            Visibility::Visible,
            TxCube {
                hash: tx.hash.clone(),
                gas: tx.gas,
                gas_price: tx.gas_price,
                value_eth: tx.value_eth,
                from: tx.from.clone(),
                to: tx.to.clone(),
            },
        ));
    }
}

fn grid_positions(count: usize, slab_width: f32, slab_depth: f32) -> Vec<(f32, f32)> {
    let cols = ((slab_width - CUBE_BASE) / GRID_SPACING).floor().max(1.0) as usize;
    let max_rows = ((slab_depth - CUBE_BASE) / GRID_SPACING).floor().max(1.0) as usize;
    let half_w = (cols as f32 * GRID_SPACING) / 2.0;
    let half_d = (max_rows as f32 * GRID_SPACING) / 2.0;

    let mut positions = Vec::with_capacity(count);
    for i in 0..count {
        let col = i % cols;
        let row = i / cols;
        if row >= max_rows {
            break;
        }
        let x = -half_w + GRID_SPACING / 2.0 + col as f32 * GRID_SPACING;
        let dz = -half_d + GRID_SPACING / 2.0 + row as f32 * GRID_SPACING;
        positions.push((x, dz));
    }
    positions
}

fn tx_height(tx: &TxPayload) -> f32 {
    let t = (tx.gas as f32 / 500_000.0).clamp(0.0, 1.0);
    MIN_HEIGHT + (MAX_HEIGHT - MIN_HEIGHT) * t
}
