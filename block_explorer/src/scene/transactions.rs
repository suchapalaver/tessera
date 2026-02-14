//! Transaction cubes: grid layout with contract clustering, gas-price color.

use std::collections::HashMap;

use crate::data::{BlockPayload, TxPayload};
use bevy::prelude::*;

use super::materials;

#[derive(Component)]
pub struct TxCube {
    pub hash: Option<String>,
    pub tx_index: usize,
    pub gas: u64,
    pub gas_price: u64,
    pub value_eth: f64,
    pub from: Option<String>,
    pub to: Option<String>,
    pub block_number: u64,
    pub world_position: Vec3,
}

const GRID_SPACING: f32 = 0.25;
const CUBE_BASE: f32 = 0.2;
const MIN_HEIGHT: f32 = 0.1;
const MAX_HEIGHT: f32 = 0.6;
const SLAB_HEIGHT: f32 = 1.0;
const SLAB_DEPTH: f32 = 2.0;
const MAX_CLUSTER_LABELS: usize = 3;

fn known_contract_name(address: &str) -> Option<&'static str> {
    let addr = address.to_lowercase();
    // Well-known Ethereum contracts
    match addr.as_str() {
        "0xdac17f958d2ee523a2206206994597c13d831ec7" => Some("USDT"),
        "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48" => Some("USDC"),
        "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2" => Some("WETH"),
        "0x7a250d5630b4cf539739df2c5dacb4c659f2488d" => Some("UniV2Router"),
        "0xe592427a0aece92de3edee1f18e0157c05861564" => Some("UniV3Router"),
        "0x68b3465833fb72a70ecdf485e0e4c7bd8665fc45" => Some("UniRouter2"),
        "0x3fc91a3afd70395cd496c647d5a6cc9d4b2b7fad" => Some("UniRouter"),
        "0x1111111254eeb25477b68fb85ed929f73a960582" => Some("1inch"),
        "0x881d40237659c251811cec9c364ef91dc08d300c" => Some("Metamask"),
        "0x7d1afa7b718fb893db30a3abc0cfc608aacfebb0" => Some("MATIC"),
        _ => None,
    }
}

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

        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(CUBE_BASE, height, CUBE_BASE))),
            MeshMaterial3d(material),
            Transform::from_xyz(pos.0, y, z + pos.1),
            Visibility::Visible,
            TxCube {
                hash: tx.hash.clone(),
                tx_index: tx.tx_index,
                gas: tx.gas,
                gas_price: tx.gas_price,
                value_eth: tx.value_eth,
                from: tx.from.clone(),
                to: tx.to.clone(),
                block_number: payload.number,
                world_position: Vec3::new(pos.0, y, z + pos.1),
            },
        ));
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
    let mut groups: HashMap<Option<&str>, Vec<&TxPayload>> = HashMap::new();
    for tx in txs {
        let key = tx.to.as_deref();
        groups.entry(key).or_default().push(tx);
    }

    let mut sorted_groups: Vec<(Option<&str>, Vec<&TxPayload>)> = groups.into_iter().collect();
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
    let mut current_to = ordered_txs[0].to.as_deref();
    let mut start = 0;

    for (i, tx) in ordered_txs.iter().enumerate() {
        let this_to = tx.to.as_deref();
        if this_to != current_to {
            if let Some(addr) = current_to {
                let label = cluster_label(addr);
                clusters.push((label, start, i));
            }
            current_to = this_to;
            start = i;
        }
    }
    if let Some(addr) = current_to {
        let label = cluster_label(addr);
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

fn cluster_label(address: &str) -> String {
    if let Some(name) = known_contract_name(address) {
        return name.to_string();
    }
    // Show abbreviated address
    if address.len() >= 10 {
        format!("{}..{}", &address[..6], &address[address.len() - 4..])
    } else {
        address.to_string()
    }
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
