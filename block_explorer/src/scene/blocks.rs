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

/// Entry in the block registry for timeline navigation.
pub struct BlockEntry {
    pub number: u64,
    pub z_position: f32,
    pub timestamp: u64,
    pub gas_fullness: f32,
    pub gas_used: u64,
    pub gas_limit: u64,
    pub tx_count: u32,
    pub base_fee_per_gas: Option<u64>,
    pub blob_gas_used: Option<u64>,
}

/// Registry of ingested blocks for timeline navigation.
#[derive(Resource, Default)]
pub struct BlockRegistry {
    pub entries: Vec<BlockEntry>,
}

/// Stores both original and heatmap materials for a slab.
#[derive(Component)]
pub struct HeatmapMaterial {
    pub original: Handle<StandardMaterial>,
    pub heatmap: Handle<StandardMaterial>,
}

/// Global toggle for heatmap mode.
#[derive(Resource, Default)]
pub struct HeatmapState {
    pub enabled: bool,
}

pub fn heatmap_plugin(app: &mut App) {
    app.init_resource::<HeatmapState>()
        .add_systems(Update, heatmap_toggle_system);
}

fn heatmap_toggle_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<HeatmapState>,
    mut commands: Commands,
    slabs: Query<(Entity, &HeatmapMaterial)>,
    tx_cubes: Query<Entity, With<crate::scene::TxCube>>,
) {
    if !keys.just_pressed(KeyCode::KeyH) {
        return;
    }

    state.enabled = !state.enabled;

    for (entity, heatmap_mat) in &slabs {
        let mat = if state.enabled {
            heatmap_mat.heatmap.clone()
        } else {
            heatmap_mat.original.clone()
        };
        commands.entity(entity).insert(MeshMaterial3d(mat));
    }

    let visibility = if state.enabled {
        Visibility::Hidden
    } else {
        Visibility::Visible
    };
    for entity in &tx_cubes {
        commands.entity(entity).insert(visibility);
    }
}

const MAX_BLOCKS_PER_FRAME: usize = 5;

pub fn setup_scene(mut commands: Commands) {
    commands.insert_resource(ExplorerState::default());
    commands.insert_resource(BlockRegistry::default());
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

#[allow(clippy::too_many_arguments)]
pub fn ingest_blocks(
    mut commands: Commands,
    channel: Res<BlockChannel>,
    mut state: ResMut<ExplorerState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials_res: ResMut<Assets<StandardMaterial>>,
    mut hud_state: ResMut<HudState>,
    mut images: ResMut<Assets<Image>>,
    mut registry: ResMut<BlockRegistry>,
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
                    &mut images,
                    &mut registry,
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
    images: &mut ResMut<Assets<Image>>,
    registry: &mut ResMut<BlockRegistry>,
) {
    let fullness = if payload.gas_limit > 0 {
        payload.gas_used as f32 / payload.gas_limit as f32
    } else {
        0.0
    };
    let width = 2.0 + 10.0 * fullness;
    let original_material = materials::block_slab_material_with_fullness(materials_res, fullness);
    let heatmap_image = materials::generate_heatmap_image(&payload.transactions);
    let heatmap_img_handle = images.add(heatmap_image);
    let heatmap_material = materials_res.add(StandardMaterial {
        base_color_texture: Some(heatmap_img_handle),
        unlit: true,
        ..default()
    });
    state.z_cursor -= 4.0;
    state.blocks_rendered += 1;
    registry.entries.push(BlockEntry {
        number: payload.number,
        z_position: state.z_cursor,
        timestamp: payload.timestamp,
        gas_fullness: fullness,
        gas_used: payload.gas_used,
        gas_limit: payload.gas_limit,
        tx_count: payload.tx_count,
        base_fee_per_gas: payload.base_fee_per_gas,
        blob_gas_used: payload.blob_gas_used,
    });
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(width, 1.0, 2.0))),
        MeshMaterial3d(original_material.clone()),
        Transform::from_xyz(0.0, 0.0, state.z_cursor),
        Visibility::Visible,
        HeatmapMaterial {
            original: original_material,
            heatmap: heatmap_material,
        },
        BlockSlab {
            number: payload.number,
            gas_used: payload.gas_used,
            gas_limit: payload.gas_limit,
            timestamp: payload.timestamp,
            tx_count: payload.tx_count,
        },
    ));
    crate::scene::labels::spawn_block_labels(
        commands,
        images,
        materials_res,
        meshes,
        payload.number,
        state.z_cursor,
        width,
    );
    crate::scene::transactions::spawn_tx_cubes(
        commands,
        payload,
        state.z_cursor,
        meshes,
        materials_res,
        images,
    );
}
