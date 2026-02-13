//! Transaction cubes: grid layout, gas-price color, TxCube component.

use crate::data::BlockPayload;
use bevy::prelude::*;

#[derive(Component)]
pub struct TxCube;

/// Placeholder: spawn no cubes until layout/color helpers are implemented.
pub fn spawn_tx_cubes(
    _commands: &mut Commands,
    _payload: &BlockPayload,
    _z: f32,
    _meshes: &mut ResMut<Assets<Mesh>>,
    _materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // TODO: grid_positions, gas_price_color, cube entities with TxCube
}
