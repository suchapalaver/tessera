//! Block slabs: ingest_blocks system, ExplorerState, BlockSlab component.

use crate::data::BlockChannel;
use crate::render::RendererResource;
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
    renderer: Res<RendererResource>,
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
                renderer.0.spawn_block(
                    &mut commands,
                    &mut meshes,
                    &mut materials_res,
                    &mut images,
                    &mut state,
                    &mut registry,
                    &payload,
                );
                received += 1;
            }
            Err(_) => break,
        }
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
