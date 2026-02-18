//! Cross-lane blob link arcs between L2 blocks and their L1 origin blocks.
//!
//! Each OP Stack L2 block references an L1 "origin" block via a deposit tx.
//! Arcs bridge from L2 blocks to the nearest visible mainnet slab, showing
//! the derivation relationship. Multiple L2 blocks sharing the same L1
//! origin are drawn as a single grouped arc to reduce visual noise.

use std::collections::HashMap;

use alloy_chains::Chain;
use bevy::prelude::*;

use crate::scene::BlockSlab;

/// A link between an L2 block and the L1 block it was derived from.
struct BlobLink {
    l1_block_number: u64,
    l2_chain: Chain,
    l2_block_number: u64,
}

/// Registry of L1↔L2 blob links, populated during block ingestion.
#[derive(Resource, Default)]
pub struct BlobLinkRegistry {
    links: Vec<BlobLink>,
}

impl BlobLinkRegistry {
    pub fn register(&mut self, l1_block_number: u64, l2_chain: Chain, l2_block_number: u64) {
        self.links.push(BlobLink {
            l1_block_number,
            l2_chain,
            l2_block_number,
        });
    }

    /// Remove links whose L2 block has been despawned.
    pub fn remove_blocks(&mut self, removed: &std::collections::HashSet<(Chain, u64)>) {
        self.links
            .retain(|link| !removed.contains(&(link.l2_chain, link.l2_block_number)));
    }
}

/// Controls blob link arc visibility. Toggled with `B`.
#[derive(Resource)]
pub struct BlobLinkSettings {
    pub enabled: bool,
}

impl Default for BlobLinkSettings {
    fn default() -> Self {
        Self { enabled: true }
    }
}

pub fn blob_link_plugin(app: &mut App) {
    app.init_resource::<BlobLinkRegistry>()
        .init_resource::<BlobLinkSettings>()
        .add_systems(Update, (toggle_blob_links_system, draw_blob_links_system));
}

fn toggle_blob_links_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<BlobLinkSettings>,
) {
    if keys.just_pressed(KeyCode::KeyB) {
        settings.enabled = !settings.enabled;
    }
}

/// Base brand blue for Base chain arcs (low alpha — recedes behind blocks).
const BASE_COLOR: Color = Color::srgba(0.0, 0.322, 1.0, 0.35);
/// Optimism red for OP Mainnet arcs (low alpha).
const OPTIMISM_COLOR: Color = Color::srgba(1.0, 0.016, 0.125, 0.35);

/// Draws arcs from groups of L2 blocks to the nearest visible mainnet slab.
///
/// L2 blocks sharing the same L1 origin are grouped into a single arc
/// from their centroid. The arc targets the closest mainnet slab by Z depth,
/// so arcs always connect to a visible block rather than empty space.
fn draw_blob_links_system(
    mut gizmos: Gizmos,
    settings: Res<BlobLinkSettings>,
    link_registry: Res<BlobLinkRegistry>,
    slabs: Query<(&BlockSlab, &GlobalTransform)>,
) {
    if !settings.enabled || link_registry.links.is_empty() {
        return;
    }

    // Build lookup of currently visible slabs and collect mainnet positions.
    let mut slab_positions: HashMap<(Chain, u64), Vec3> = HashMap::new();
    let mut mainnet_positions: Vec<Vec3> = Vec::new();
    for (slab, transform) in &slabs {
        let pos = transform.translation();
        slab_positions.insert((slab.chain, slab.number), pos);
        if slab.chain == Chain::mainnet() {
            mainnet_positions.push(pos);
        }
    }

    if mainnet_positions.is_empty() {
        return;
    }

    // Group L2 blocks by (l2_chain, l1_origin) → collect their positions.
    let mut groups: HashMap<(Chain, u64), Vec<Vec3>> = HashMap::new();
    for link in &link_registry.links {
        let Some(&l2_pos) = slab_positions.get(&(link.l2_chain, link.l2_block_number)) else {
            continue;
        };
        groups
            .entry((link.l2_chain, link.l1_block_number))
            .or_default()
            .push(l2_pos);
    }

    for ((l2_chain, _l1_origin), positions) in &groups {
        let count = positions.len() as f32;
        let centroid = positions.iter().copied().sum::<Vec3>() / count;

        // Find the nearest visible mainnet slab by Z distance.
        let target = nearest_by_z(&mainnet_positions, centroid.z);
        let color = chain_arc_color(*l2_chain);

        let arc_height = 1.5;
        let mid = (centroid + target) / 2.0 + Vec3::Y * arc_height;
        let control1 = centroid.lerp(mid, 0.5) + Vec3::Y * arc_height * 0.3;
        let control2 = mid.lerp(target, 0.5) + Vec3::Y * arc_height * 0.3;

        let segments = 16;
        let mut prev = centroid;
        for s in 1..=segments {
            let t = s as f32 / segments as f32;
            let point = cubic_bezier(centroid, control1, control2, target, t);
            gizmos.line(prev, point, color);
            prev = point;
        }
    }
}

/// Returns the position from `positions` closest to the given Z value.
fn nearest_by_z(positions: &[Vec3], z: f32) -> Vec3 {
    *positions
        .iter()
        .min_by(|a, b| {
            let da = (a.z - z).abs();
            let db = (b.z - z).abs();
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
        .expect("positions must not be empty")
}

fn chain_arc_color(chain: Chain) -> Color {
    use alloy_chains::NamedChain;
    match chain.named() {
        Some(NamedChain::Base) => BASE_COLOR,
        Some(NamedChain::Optimism) => OPTIMISM_COLOR,
        _ => Color::srgb(0.5, 0.5, 0.8),
    }
}

fn cubic_bezier(p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3, t: f32) -> Vec3 {
    let u = 1.0 - t;
    let tt = t * t;
    let uu = u * u;
    let uuu = uu * u;
    let ttt = tt * t;
    uuu * p0 + 3.0 * uu * t * p1 + 3.0 * u * tt * p2 + ttt * p3
}
