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

/// Base brand blue for Base chain arcs.
const BASE_COLOR: Color = Color::srgb(0.0, 0.322, 1.0);
/// Optimism red for OP Mainnet arcs.
const OPTIMISM_COLOR: Color = Color::srgb(1.0, 0.016, 0.125);

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

    // Also build lookup from block number → entry for timestamp comparison
    let mut block_timestamps: HashMap<(Chain, u64), u64> = HashMap::new();
    for entry in &registry.entries {
        block_timestamps.insert((entry.chain, entry.number), entry.timestamp);
    }

    for link in &link_registry.links {
        // Find L1 slab position (mainnet)
        let Some(&l1_pos) = slab_positions.get(&(Chain::mainnet(), link.l1_block_number)) else {
            continue;
        };

        // Find L2 slab position
        let Some(&l2_pos) = slab_positions.get(&(link.l2_chain, link.l2_block_number)) else {
            continue;
        };

        // Arc color based on L2 chain
        let color = chain_arc_color(link.l2_chain);

        // Arc height proportional to the time gap between L1 and L2 blocks
        let l1_ts = block_timestamps
            .get(&(Chain::mainnet(), link.l1_block_number))
            .copied()
            .unwrap_or(0);
        let l2_ts = block_timestamps
            .get(&(link.l2_chain, link.l2_block_number))
            .copied()
            .unwrap_or(0);
        let time_gap = l2_ts.saturating_sub(l1_ts) as f32;
        let arc_height = 2.0 + (time_gap / 12.0).min(8.0);

        // Draw cubic bezier arc between L1 and L2 slabs
        let mid = (l1_pos + l2_pos) / 2.0 + Vec3::Y * arc_height;
        let control1 = l1_pos.lerp(mid, 0.5) + Vec3::Y * arc_height * 0.3;
        let control2 = mid.lerp(l2_pos, 0.5) + Vec3::Y * arc_height * 0.3;

        let segments = 20;
        let mut prev = l1_pos;
        for s in 1..=segments {
            let t = s as f32 / segments as f32;
            let point = cubic_bezier(l1_pos, control1, control2, l2_pos, t);
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
