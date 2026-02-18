//! Cross-lane blob link arcs between L2 blocks and their L1 origin blocks.

use std::collections::HashMap;

use alloy_chains::Chain;
use bevy::prelude::*;

use crate::scene::blocks::BlockRegistry;
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

    /// Remove links referencing any of the given (chain, block_number) pairs.
    pub fn remove_blocks(&mut self, removed: &std::collections::HashSet<(Chain, u64)>) {
        self.links.retain(|link| {
            !removed.contains(&(Chain::mainnet(), link.l1_block_number))
                && !removed.contains(&(link.l2_chain, link.l2_block_number))
        });
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
const BASE_COLOR: Color = Color::srgba(0.0, 0.322, 1.0, 0.25);
/// Optimism red for OP Mainnet arcs (low alpha).
const OPTIMISM_COLOR: Color = Color::srgba(1.0, 0.016, 0.125, 0.25);

/// Groups links by (l2_chain, l1_block_number) so we draw one arc per
/// L1-origin group instead of one per L2 block. Each arc connects the
/// L1 slab to the centroid of its child L2 slabs.
fn draw_blob_links_system(
    mut gizmos: Gizmos,
    settings: Res<BlobLinkSettings>,
    link_registry: Res<BlobLinkRegistry>,
    registry: Res<BlockRegistry>,
    slabs: Query<(&BlockSlab, &GlobalTransform)>,
) {
    if !settings.enabled || link_registry.links.is_empty() {
        return;
    }

    // Build lookup from (chain, block_number) → world position
    let mut slab_positions: HashMap<(Chain, u64), Vec3> = HashMap::new();
    for (slab, transform) in &slabs {
        slab_positions.insert((slab.chain, slab.number), transform.translation());
    }

    let mut block_timestamps: HashMap<(Chain, u64), u64> = HashMap::new();
    for entry in &registry.entries {
        block_timestamps.insert((entry.chain, entry.number), entry.timestamp);
    }

    // Group links by (l2_chain, l1_block_number) → list of L2 block numbers
    let mut groups: HashMap<(Chain, u64), Vec<u64>> = HashMap::new();
    for link in &link_registry.links {
        groups
            .entry((link.l2_chain, link.l1_block_number))
            .or_default()
            .push(link.l2_block_number);
    }

    for ((l2_chain, l1_block_number), l2_blocks) in &groups {
        let Some(&l1_pos) = slab_positions.get(&(Chain::mainnet(), *l1_block_number)) else {
            continue;
        };

        // Compute centroid of all L2 slabs in this group
        let mut centroid = Vec3::ZERO;
        let mut count = 0u32;
        for &l2_num in l2_blocks {
            if let Some(&pos) = slab_positions.get(&(*l2_chain, l2_num)) {
                centroid += pos;
                count += 1;
            }
        }
        if count == 0 {
            continue;
        }
        let l2_centroid = centroid / count as f32;

        let color = chain_arc_color(*l2_chain);

        // Arc height from average time gap
        let l1_ts = block_timestamps
            .get(&(Chain::mainnet(), *l1_block_number))
            .copied()
            .unwrap_or(0);
        let avg_l2_ts: u64 = l2_blocks
            .iter()
            .filter_map(|n| block_timestamps.get(&(*l2_chain, *n)).copied())
            .sum::<u64>()
            / count as u64;
        let time_gap = avg_l2_ts.saturating_sub(l1_ts) as f32;
        let arc_height = 2.0 + (time_gap / 12.0).min(8.0);

        // Draw one bezier arc from L1 slab to L2 group centroid
        let mid = (l1_pos + l2_centroid) / 2.0 + Vec3::Y * arc_height;
        let control1 = l1_pos.lerp(mid, 0.5) + Vec3::Y * arc_height * 0.3;
        let control2 = mid.lerp(l2_centroid, 0.5) + Vec3::Y * arc_height * 0.3;

        // More segments for larger groups (subtle detail reward)
        let segments = 16 + count.min(8) as usize;
        let mut prev = l1_pos;
        for s in 1..=segments {
            let t = s as f32 / segments as f32;
            let point = cubic_bezier(l1_pos, control1, control2, l2_centroid, t);
            gizmos.line(prev, point, color);
            prev = point;
        }
    }
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
