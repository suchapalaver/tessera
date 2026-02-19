//! Block slabs: ingest_blocks system, ExplorerState, BlockSlab component.

use std::collections::{HashMap, HashSet};

use alloy_chains::Chain;

use crate::data::{BlockChannel, RecordBuffer};
use crate::render::RendererResource;
use crate::scene::blob_links::BlobLinkRegistry;
use crate::scene::BlockLabel;
use crate::ui::HudState;
use bevy::prelude::*;

const DEFAULT_LANE_SPACING: f32 = 15.0;
/// Z units per second of block time. Converts timestamps to spatial positions
/// so blocks from all chains align temporally. At 2.0, Base blocks (~2s apart)
/// get 4.0 units of spacing, matching the previous fixed z_spacing.
const Z_PER_SECOND: f32 = 2.0;
/// Rolling time window in seconds. Blocks older than this are despawned.
const WINDOW_SECONDS: u64 = 120;

/// Per-chain lane positioning state.
pub struct LaneState {
    pub x_offset: f32,
    pub blocks_rendered: u64,
}

/// Tracks per-chain lane state for multi-chain visualization.
/// Z positions are derived from block timestamps so all chains align temporally.
#[derive(Resource)]
pub struct ExplorerState {
    pub lanes: HashMap<Chain, LaneState>,
    pub lane_spacing: f32,
    next_lane_index: usize,
    reference_timestamp: Option<u64>,
}

impl Default for ExplorerState {
    fn default() -> Self {
        Self {
            lanes: HashMap::new(),
            lane_spacing: DEFAULT_LANE_SPACING,
            next_lane_index: 0,
            reference_timestamp: None,
        }
    }
}

impl ExplorerState {
    /// Returns the lane for the given chain, creating one if it doesn't exist.
    pub fn lane_for(&mut self, chain: Chain) -> &mut LaneState {
        let spacing = self.lane_spacing;
        let next = &mut self.next_lane_index;
        self.lanes.entry(chain).or_insert_with(|| {
            let index = *next;
            *next += 1;
            LaneState {
                x_offset: index as f32 * spacing,
                blocks_rendered: 0,
            }
        })
    }

    /// Computes the Z position for a block based on its timestamp.
    /// The first block received sets the reference; all others are relative to it.
    pub fn z_for_timestamp(&mut self, timestamp: u64) -> f32 {
        let reference = *self.reference_timestamp.get_or_insert(timestamp);
        -((timestamp as f64 - reference as f64) * Z_PER_SECOND as f64) as f32
    }
}

/// Marker + data for slab entities.
#[derive(Component)]
pub struct BlockSlab {
    pub chain: Chain,
    pub number: u64,
    pub gas_used: u64,
    pub gas_limit: u64,
    pub timestamp: u64,
    pub tx_count: u32,
    pub l1_origin_number: Option<u64>,
}

/// Entry in the block registry for timeline navigation.
pub struct BlockEntry {
    pub chain: Chain,
    pub number: u64,
    pub z_position: f32,
    pub x_offset: f32,
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
    let mid_x = DEFAULT_LANE_SPACING / 2.0;
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(mid_x, 8., 15.).looking_at(Vec3::new(mid_x, 0., -10.), Vec3::Y),
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
    renderer: Res<RendererResource>,
    mut state: ResMut<ExplorerState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials_res: ResMut<Assets<StandardMaterial>>,
    mut hud_state: ResMut<HudState>,
    mut images: ResMut<Assets<Image>>,
    mut registry: ResMut<BlockRegistry>,
    blob_links: Option<ResMut<BlobLinkRegistry>>,
    mut record_buffer: Option<ResMut<RecordBuffer>>,
) {
    let mut received = 0usize;
    let mut blob_links = blob_links;
    while received < MAX_BLOCKS_PER_FRAME {
        match channel.0.try_recv() {
            Ok(payload) => {
                if let Some(ref mut buf) = record_buffer {
                    buf.payloads.push(payload.clone());
                }

                hud_state.update_from_payload(&payload);

                if let (Some(l1_origin), Some(ref mut links)) =
                    (payload.l1_origin_number, blob_links.as_mut())
                {
                    links.register(l1_origin, payload.chain, payload.number);
                }

                let x_offset = state.lane_for(payload.chain).x_offset;
                renderer.0.spawn_block(
                    &mut commands,
                    &mut meshes,
                    &mut materials_res,
                    &mut images,
                    &mut state,
                    &mut registry,
                    &payload,
                    x_offset,
                );
                received += 1;
            }
            Err(_) => break,
        }
    }
}

/// Flushes the record buffer to disk when the app exits.
pub fn flush_record_buffer(
    mut exit_events: EventReader<AppExit>,
    buffer: Option<Res<RecordBuffer>>,
) {
    if exit_events.read().next().is_some() {
        if let Some(buf) = buffer {
            buf.flush();
        }
    }
}

/// Despawns blocks outside the rolling time window.
/// All chains share the same temporal window so lanes stay aligned.
pub fn cleanup_old_blocks(
    mut commands: Commands,
    slabs: Query<(Entity, &BlockSlab)>,
    cubes: Query<(Entity, &crate::scene::TxCube)>,
    labels: Query<(Entity, &BlockLabel)>,
    mut registry: ResMut<BlockRegistry>,
    blob_link_registry: Option<ResMut<BlobLinkRegistry>>,
) {
    // Find the latest timestamp across all chains
    let latest_ts = slabs.iter().map(|(_, s)| s.timestamp).max().unwrap_or(0);
    if latest_ts == 0 {
        return;
    }
    let cutoff = latest_ts.saturating_sub(WINDOW_SECONDS);

    let mut removed: HashSet<(Chain, u64)> = HashSet::new();

    for (entity, slab) in &slabs {
        if slab.timestamp < cutoff {
            commands.entity(entity).despawn();
            removed.insert((slab.chain, slab.number));
        }
    }

    if removed.is_empty() {
        return;
    }

    for (entity, cube) in &cubes {
        if removed.contains(&(cube.chain, cube.block_number)) {
            commands.entity(entity).despawn_recursive();
        }
    }

    for (entity, label) in &labels {
        if removed.contains(&(label.chain, label.block_number)) {
            commands.entity(entity).despawn();
        }
    }

    registry
        .entries
        .retain(|e| !removed.contains(&(e.chain, e.number)));

    if let Some(mut links) = blob_link_registry {
        links.remove_blocks(&removed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setup_scene_inserts_resources_and_entities() {
        let mut app = App::new();
        app.add_systems(Startup, setup_scene);

        app.update();

        assert!(app.world().get_resource::<ExplorerState>().is_some());
        assert!(app.world().get_resource::<BlockRegistry>().is_some());

        let world = app.world_mut();
        let camera_count = world.query::<&Camera3d>().iter(world).count();
        let light_count = world.query::<&DirectionalLight>().iter(world).count();

        assert!(camera_count >= 1);
        assert!(light_count >= 1);
    }
}
