//! Block slabs: ingest_blocks system, ExplorerState, BlockSlab component.

use crate::data::{BlockChannel, BlockPayload};
use crate::scene::materials;
use crate::ui::HudState;
use bevy::prelude::*;

/// Tracks how many blocks we've rendered and current Z position.
#[derive(Resource, Default)]
pub struct ExplorerState {
    pub blocks_rendered: u64,
    pub z_cursor: f32,
}

/// Marker + data for slab entities.
#[derive(Component)]
pub struct BlockSlab {
    pub number: u64,
    pub gas_used: u64,
    pub gas_limit: u64,
    pub timestamp: u64,
    pub tx_count: u32,
}

const MAX_BLOCKS_PER_FRAME: usize = 5;

pub fn setup_scene(mut commands: Commands) {
    commands.insert_resource(ExplorerState::default());
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0., 5., 10.).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(4., 8., 4.).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.3,
    });
}

pub fn ingest_blocks(
    mut commands: Commands,
    channel: Res<BlockChannel>,
    mut state: ResMut<ExplorerState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials_res: ResMut<Assets<StandardMaterial>>,
    mut hud_state: ResMut<HudState>,
) {
    let mut received = 0usize;
    while received < MAX_BLOCKS_PER_FRAME {
        match channel.0.try_recv() {
            Ok(payload) => {
                hud_state.update_from_payload(&payload);
                spawn_block_slab(
                    &mut commands,
                    &payload,
                    &mut state,
                    &mut meshes,
                    &mut materials_res,
                );
                received += 1;
            }
            Err(_) => break,
        }
    }
}

fn spawn_block_slab(
    commands: &mut Commands,
    payload: &BlockPayload,
    state: &mut ResMut<ExplorerState>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials_res: &mut ResMut<Assets<StandardMaterial>>,
) {
    let fullness = if payload.gas_limit > 0 {
        payload.gas_used as f32 / payload.gas_limit as f32
    } else {
        0.0
    };
    let width = 2.0 + 10.0 * fullness;
    let material = materials::block_slab_material_with_fullness(materials_res, fullness);
    state.z_cursor -= 4.0;
    state.blocks_rendered += 1;
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(width, 1.0, 2.0))),
        MeshMaterial3d(material),
        Transform::from_xyz(0.0, 0.0, state.z_cursor),
        Visibility::Visible,
        PickingBehavior::default(),
        BlockSlab {
            number: payload.number,
            gas_used: payload.gas_used,
            gas_limit: payload.gas_limit,
            timestamp: payload.timestamp,
            tx_count: payload.tx_count,
        },
    ));
    crate::scene::transactions::spawn_tx_cubes(
        commands,
        payload,
        state.z_cursor,
        meshes,
        materials_res,
    );
}
