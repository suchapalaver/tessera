use std::collections::HashMap;

use alloy::primitives::{address, Address};
use bevy::prelude::*;

use crate::data::{BlockPayload, TxPayload};
use crate::render::BlockRenderer;
use crate::scene::blocks::{BlockEntry, BlockSlab, HeatmapMaterial};
use crate::scene::{labels, materials, BlockLabel, TxCube};

#[derive(Clone, Debug)]
pub struct SlabSettings {
    pub base_width: f32,
    pub width_scale: f32,
    pub height: f32,
    pub depth: f32,
    pub z_spacing: f32,
}

#[derive(Clone, Debug)]
pub struct TxRenderSettings {
    pub grid_spacing: f32,
    pub cube_base: f32,
    pub min_height: f32,
    pub max_height: f32,
}

#[derive(Clone, Debug)]
pub struct ClusterLabelSettings {
    pub max_labels: usize,
    pub quad_height: f32,
}

#[derive(Clone, Debug)]
pub struct BlobRenderSettings {
    pub sphere_radius: f32,
    pub sphere_spacing: f32,
    pub base_batcher: Address,
}

#[derive(Clone, Debug)]
pub struct SlabsAndCubesSettings {
    pub slab: SlabSettings,
    pub tx: TxRenderSettings,
    pub clusters: ClusterLabelSettings,
    pub blobs: BlobRenderSettings,
}

impl Default for SlabsAndCubesSettings {
    fn default() -> Self {
        Self {
            slab: SlabSettings {
                base_width: 2.0,
                width_scale: 10.0,
                height: 1.0,
                depth: 2.0,
                z_spacing: 4.0,
            },
            tx: TxRenderSettings {
                grid_spacing: 0.25,
                cube_base: 0.2,
                min_height: 0.1,
                max_height: 0.6,
            },
            clusters: ClusterLabelSettings {
                max_labels: 1,
                quad_height: 0.4,
            },
            blobs: BlobRenderSettings {
                sphere_radius: 0.06,
                sphere_spacing: 0.14,
                base_batcher: address!("5050F69a9786F081509234F1a7F4684b5E5b76C9"),
            },
        }
    }
}

#[derive(Default)]
pub struct SlabsAndCubesRenderer {
    pub settings: SlabsAndCubesSettings,
}

impl BlockRenderer for SlabsAndCubesRenderer {
    fn spawn_block(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        images: &mut ResMut<Assets<Image>>,
        state: &mut ResMut<crate::scene::blocks::ExplorerState>,
        registry: &mut ResMut<crate::scene::blocks::BlockRegistry>,
        payload: &BlockPayload,
        x_offset: f32,
    ) {
        let slab_settings = &self.settings.slab;
        let tx_settings = &self.settings.tx;
        let cluster_settings = &self.settings.clusters;
        let blob_settings = &self.settings.blobs;

        let fullness = if payload.gas_limit > 0 {
            payload.gas_used as f32 / payload.gas_limit as f32
        } else {
            0.0
        };

        let width = slab_settings.base_width + slab_settings.width_scale * fullness;
        let original_material = materials::block_slab_material_with_fullness(materials, fullness);
        let heatmap_image = materials::generate_heatmap_image(&payload.transactions, payload.chain);
        let heatmap_img_handle = images.add(heatmap_image);
        let heatmap_material = materials.add(StandardMaterial {
            base_color_texture: Some(heatmap_img_handle),
            unlit: true,
            ..default()
        });

        let lane = state.lane_for(payload.chain);
        lane.z_cursor -= slab_settings.z_spacing;
        lane.blocks_rendered += 1;
        let z_cursor = lane.z_cursor;

        registry.entries.push(BlockEntry {
            chain: payload.chain,
            number: payload.number,
            z_position: z_cursor,
            x_offset,
            timestamp: payload.timestamp,
            gas_fullness: fullness,
            gas_used: payload.gas_used,
            gas_limit: payload.gas_limit,
            tx_count: payload.tx_count,
            base_fee_per_gas: payload.base_fee_per_gas,
            blob_gas_used: payload.blob_gas_used,
        });

        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(
                width,
                slab_settings.height,
                slab_settings.depth,
            ))),
            MeshMaterial3d(original_material.clone()),
            Transform::from_xyz(x_offset, 0.0, z_cursor),
            Visibility::Visible,
            HeatmapMaterial {
                original: original_material,
                heatmap: heatmap_material,
            },
            BlockSlab {
                chain: payload.chain,
                number: payload.number,
                gas_used: payload.gas_used,
                gas_limit: payload.gas_limit,
                timestamp: payload.timestamp,
                tx_count: payload.tx_count,
                l1_origin_number: payload.l1_origin_number,
            },
        ));

        labels::spawn_block_labels(
            commands,
            images,
            materials,
            meshes,
            payload.chain,
            payload.number,
            z_cursor,
            width,
            x_offset,
        );

        spawn_tx_cubes(
            commands,
            payload,
            z_cursor,
            meshes,
            materials,
            images,
            slab_settings.height,
            slab_settings.depth,
            width,
            tx_settings,
            cluster_settings,
            blob_settings,
            x_offset,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_tx_cubes(
    commands: &mut Commands,
    payload: &BlockPayload,
    z: f32,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials_res: &mut ResMut<Assets<StandardMaterial>>,
    images: &mut ResMut<Assets<Image>>,
    slab_height: f32,
    slab_depth: f32,
    slab_width: f32,
    settings: &TxRenderSettings,
    cluster_settings: &ClusterLabelSettings,
    blob_settings: &BlobRenderSettings,
    x_offset: f32,
) {
    if payload.transactions.is_empty() {
        return;
    }

    let ordered_txs = cluster_transactions(&payload.transactions);
    let positions = grid_positions(
        ordered_txs.len(),
        slab_width,
        slab_depth,
        settings.grid_spacing,
        settings.cube_base,
    );

    for (i, tx) in ordered_txs.iter().enumerate() {
        if i >= positions.len() {
            break;
        }
        let pos = positions[i];
        let height = tx_height(tx, settings);
        let y = slab_height / 2.0 + height / 2.0;
        let material = materials::tx_cube_material(
            materials_res,
            tx,
            payload.transactions.len(),
            payload.chain,
        );

        let world_pos = Vec3::new(x_offset + pos.0, y, z + pos.1);
        let mut entity_commands = commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(settings.cube_base, height, settings.cube_base))),
            MeshMaterial3d(material),
            Transform::from_xyz(x_offset + pos.0, y, z + pos.1),
            Visibility::Visible,
            TxCube {
                chain: payload.chain,
                hash: format!("{}", tx.hash),
                tx_index: tx.tx_index,
                gas: tx.gas,
                gas_price: tx.gas_price,
                value_eth: tx.value_eth,
                from: tx.from,
                to: tx.to,
                block_number: payload.number,
                world_position: world_pos,
                blob_count: tx.blob_count,
                max_fee_per_blob_gas: tx.max_fee_per_blob_gas,
            },
        ));

        if tx.blob_count > 0 {
            spawn_blob_spheres(
                &mut entity_commands,
                tx.blob_count,
                tx.from,
                height,
                meshes,
                materials_res,
                blob_settings,
            );
        }
    }

    let tag = BlockLabel {
        chain: payload.chain,
        block_number: payload.number,
    };
    spawn_cluster_labels(
        commands,
        &ordered_txs,
        &positions,
        z,
        meshes,
        materials_res,
        images,
        slab_height,
        cluster_settings,
        x_offset,
        &tag,
    );
}

/// Groups transactions by `to` address, sorts groups largest-first, and returns
/// a flat list in cluster order.
fn cluster_transactions(txs: &[TxPayload]) -> Vec<&TxPayload> {
    let mut groups: HashMap<Option<Address>, Vec<&TxPayload>> = HashMap::new();
    for tx in txs {
        groups.entry(tx.to).or_default().push(tx);
    }

    let mut sorted_groups: Vec<(Option<Address>, Vec<&TxPayload>)> = groups.into_iter().collect();
    sorted_groups.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    sorted_groups.into_iter().flat_map(|(_, txs)| txs).collect()
}

#[allow(clippy::too_many_arguments)]
fn spawn_cluster_labels(
    commands: &mut Commands,
    ordered_txs: &[&TxPayload],
    positions: &[(f32, f32)],
    z: f32,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials_res: &mut ResMut<Assets<StandardMaterial>>,
    images: &mut ResMut<Assets<Image>>,
    slab_height: f32,
    settings: &ClusterLabelSettings,
    x_offset: f32,
    tag: &BlockLabel,
) {
    if ordered_txs.is_empty() || positions.is_empty() {
        return;
    }

    // Identify cluster boundaries and labels
    let mut clusters: Vec<(String, usize, usize)> = Vec::new(); // (label, start_idx, end_idx)
    let mut current_to = ordered_txs[0].to;
    let mut start = 0;

    for (i, tx) in ordered_txs.iter().enumerate() {
        if tx.to != current_to {
            if let Some(addr) = current_to {
                let label = cluster_label(&addr);
                clusters.push((label, start, i));
            }
            current_to = tx.to;
            start = i;
        }
    }
    if let Some(addr) = current_to {
        let label = cluster_label(&addr);
        clusters.push((label, start, ordered_txs.len()));
    }

    // Sort by cluster size and take top N
    clusters.sort_by(|a, b| (b.2 - b.1).cmp(&(a.2 - a.1)));
    let labels_to_spawn = clusters.iter().take(settings.max_labels);

    for (label, start_idx, end_idx) in labels_to_spawn {
        if *start_idx >= positions.len() || *end_idx == 0 {
            continue;
        }
        let clamped_end = (*end_idx).min(positions.len());
        if *start_idx >= clamped_end {
            continue;
        }

        // Compute centroid of this cluster's positions
        let cluster_positions = &positions[*start_idx..clamped_end];
        let centroid_x: f32 =
            cluster_positions.iter().map(|p| p.0).sum::<f32>() / cluster_positions.len() as f32;
        let centroid_z: f32 =
            cluster_positions.iter().map(|p| p.1).sum::<f32>() / cluster_positions.len() as f32;

        spawn_cluster_label_quad(
            commands,
            meshes,
            materials_res,
            images,
            label,
            Vec3::new(x_offset + centroid_x, slab_height + 1.4, z + centroid_z),
            settings.quad_height,
            tag,
        );
    }
}

fn cluster_label(addr: &Address) -> String {
    if let Some(name) = crate::scene::contracts::known_contract_name(addr) {
        return name.to_string();
    }
    // Show abbreviated address (checksummed)
    let s = format!("{addr}");
    format!("{}..{}", &s[..6], &s[s.len() - 4..])
}

#[allow(clippy::too_many_arguments)]
fn spawn_cluster_label_quad(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials_res: &mut ResMut<Assets<StandardMaterial>>,
    images: &mut ResMut<Assets<Image>>,
    text: &str,
    position: Vec3,
    quad_height: f32,
    tag: &BlockLabel,
) {
    let image = crate::scene::labels::render_label_image(text);
    let img_w = image.width();
    let img_h = image.height();
    if img_w == 0 || img_h == 0 {
        return;
    }

    let aspect = img_w as f32 / img_h as f32;
    let quad_w = quad_height * aspect;

    let img_handle = images.add(image);
    let material = materials_res.add(StandardMaterial {
        base_color_texture: Some(img_handle),
        unlit: true,
        alpha_mode: AlphaMode::Mask(0.5),
        ..default()
    });

    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(quad_w, quad_height))),
        MeshMaterial3d(material),
        Transform::from_translation(position).looking_at(position - Vec3::Z, Vec3::Y),
        tag.clone(),
    ));
}

fn grid_positions(
    count: usize,
    slab_width: f32,
    slab_depth: f32,
    grid_spacing: f32,
    cube_base: f32,
) -> Vec<(f32, f32)> {
    let cols = ((slab_width - cube_base) / grid_spacing).floor().max(1.0) as usize;
    let max_rows = ((slab_depth - cube_base) / grid_spacing).floor().max(1.0) as usize;
    let half_w = (cols as f32 * grid_spacing) / 2.0;
    let half_d = (max_rows as f32 * grid_spacing) / 2.0;

    let mut positions = Vec::with_capacity(count);
    for i in 0..count {
        let col = i % cols;
        let row = i / cols;
        if row >= max_rows {
            break;
        }
        let x = -half_w + grid_spacing / 2.0 + col as f32 * grid_spacing;
        let dz = -half_d + grid_spacing / 2.0 + row as f32 * grid_spacing;
        positions.push((x, dz));
    }
    positions
}

fn tx_height(tx: &TxPayload, settings: &TxRenderSettings) -> f32 {
    let t = (tx.gas as f32 / 500_000.0).clamp(0.0, 1.0);
    settings.min_height + (settings.max_height - settings.min_height) * t
}

fn spawn_blob_spheres(
    parent: &mut bevy::prelude::EntityCommands,
    blob_count: usize,
    from: Address,
    cube_height: f32,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials_res: &mut ResMut<Assets<StandardMaterial>>,
    settings: &BlobRenderSettings,
) {
    let sphere_mesh = meshes.add(Sphere::new(settings.sphere_radius));

    let (base_color, emissive) = if from == settings.base_batcher {
        // Base brand blue (#0052FF)
        (
            Color::srgba(0.0, 0.322, 1.0, 0.7),
            LinearRgba::rgb(0.0, 0.2, 0.8),
        )
    } else {
        // Default purple
        (
            Color::srgba(0.6, 0.3, 0.9, 0.7),
            LinearRgba::rgb(0.4, 0.15, 0.7),
        )
    };

    let blob_material = materials_res.add(StandardMaterial {
        base_color,
        emissive,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let total_width = (blob_count as f32 - 1.0) * settings.sphere_spacing;
    let start_x = -total_width / 2.0;
    let y_offset = cube_height / 2.0 + settings.sphere_radius + 0.02;

    parent.with_children(|builder| {
        for i in 0..blob_count {
            let x = start_x + i as f32 * settings.sphere_spacing;
            builder.spawn((
                Mesh3d(sphere_mesh.clone()),
                MeshMaterial3d(blob_material.clone()),
                Transform::from_xyz(x, y_offset, 0.0),
            ));
        }
    });
}
