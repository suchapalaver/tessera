//! Transaction cubes: grid layout with contract clustering, gas-price color.

use std::collections::HashMap;

use alloy::primitives::Address;

use crate::data::{BlockPayload, TxPayload};
use bevy::prelude::*;

use super::materials;

#[derive(Component)]
pub struct TxCube {
    pub hash: String,
    pub tx_index: usize,
    pub gas: u64,
    pub gas_price: u128,
    pub value_eth: f64,
    pub from: Address,
    pub to: Option<Address>,
    pub block_number: u64,
    pub world_position: Vec3,
    pub blob_count: usize,
    pub max_fee_per_blob_gas: Option<u128>,
}

const GRID_SPACING: f32 = 0.25;
const CUBE_BASE: f32 = 0.2;
const MIN_HEIGHT: f32 = 0.1;
const MAX_HEIGHT: f32 = 0.6;
const SLAB_HEIGHT: f32 = 1.0;
const SLAB_DEPTH: f32 = 2.0;
const MAX_CLUSTER_LABELS: usize = 3;

pub fn spawn_tx_cubes(
    commands: &mut Commands,
    payload: &BlockPayload,
    z: f32,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials_res: &mut ResMut<Assets<StandardMaterial>>,
    images: &mut ResMut<Assets<Image>>,
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

    // Group transactions by `to` address for clustering
    let ordered_txs = cluster_transactions(&payload.transactions);
    let positions = grid_positions(ordered_txs.len(), slab_width, SLAB_DEPTH);

    for (i, tx) in ordered_txs.iter().enumerate() {
        if i >= positions.len() {
            break;
        }
        let pos = positions[i];
        let height = tx_height(tx);
        let y = SLAB_HEIGHT / 2.0 + height / 2.0;
        let material = materials::tx_cube_material(materials_res, tx, payload.transactions.len());

        let world_pos = Vec3::new(pos.0, y, z + pos.1);
        let mut entity_commands = commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(CUBE_BASE, height, CUBE_BASE))),
            MeshMaterial3d(material),
            Transform::from_xyz(pos.0, y, z + pos.1),
            Visibility::Visible,
            TxCube {
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

        // Spawn blob spheres as children of blob-carrying transactions
        if tx.blob_count > 0 {
            spawn_blob_spheres(
                &mut entity_commands,
                tx.blob_count,
                height,
                meshes,
                materials_res,
            );
        }
    }

    // Spawn cluster labels for top groups
    spawn_cluster_labels(
        commands,
        &ordered_txs,
        &positions,
        z,
        meshes,
        materials_res,
        images,
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

fn spawn_cluster_labels(
    commands: &mut Commands,
    ordered_txs: &[&TxPayload],
    positions: &[(f32, f32)],
    z: f32,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials_res: &mut ResMut<Assets<StandardMaterial>>,
    images: &mut ResMut<Assets<Image>>,
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
    let labels_to_spawn = clusters.iter().take(MAX_CLUSTER_LABELS);

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
            Vec3::new(centroid_x, SLAB_HEIGHT + 0.6, z + centroid_z),
        );
    }
}

fn cluster_label(addr: &Address) -> String {
    if let Some(name) = super::contracts::known_contract_name(addr) {
        return name.to_string();
    }
    // Show abbreviated address (checksummed)
    let s = format!("{addr}");
    format!("{}..{}", &s[..6], &s[s.len() - 4..])
}

fn spawn_cluster_label_quad(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials_res: &mut ResMut<Assets<StandardMaterial>>,
    images: &mut ResMut<Assets<Image>>,
    text: &str,
    position: Vec3,
) {
    let image = super::labels::render_label_image(text);
    let img_w = image.width();
    let img_h = image.height();
    if img_w == 0 || img_h == 0 {
        return;
    }

    let aspect = img_w as f32 / img_h as f32;
    let quad_h = 0.3;
    let quad_w = quad_h * aspect;

    let img_handle = images.add(image);
    let material = materials_res.add(StandardMaterial {
        base_color_texture: Some(img_handle),
        unlit: true,
        alpha_mode: AlphaMode::Mask(0.5),
        ..default()
    });

    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(quad_w, quad_h))),
        MeshMaterial3d(material),
        Transform::from_translation(position).looking_at(position + Vec3::Z, Vec3::Y),
    ));
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

const BLOB_SPHERE_RADIUS: f32 = 0.06;
const BLOB_SPHERE_SPACING: f32 = 0.14;

fn spawn_blob_spheres(
    parent: &mut bevy::prelude::EntityCommands,
    blob_count: usize,
    cube_height: f32,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials_res: &mut ResMut<Assets<StandardMaterial>>,
) {
    let sphere_mesh = meshes.add(Sphere::new(BLOB_SPHERE_RADIUS));
    let blob_material = materials_res.add(StandardMaterial {
        base_color: Color::srgba(0.6, 0.3, 0.9, 0.7),
        emissive: LinearRgba::rgb(0.4, 0.15, 0.7),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let total_width = (blob_count as f32 - 1.0) * BLOB_SPHERE_SPACING;
    let start_x = -total_width / 2.0;
    let y_offset = cube_height / 2.0 + BLOB_SPHERE_RADIUS + 0.02;

    parent.with_children(|builder| {
        for i in 0..blob_count {
            let x = start_x + i as f32 * BLOB_SPHERE_SPACING;
            builder.spawn((
                Mesh3d(sphere_mesh.clone()),
                MeshMaterial3d(blob_material.clone()),
                Transform::from_xyz(x, y_offset, 0.0),
            ));
        }
    });
}
