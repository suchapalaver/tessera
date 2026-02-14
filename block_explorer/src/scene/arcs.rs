//! Value-flow arcs: bezier arcs between transaction endpoints using Bevy Gizmos.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::scene::BlockSlab;
use crate::scene::TxCube;
use crate::ui::inspector::SelectedEntity;

const MAX_ARCS: usize = 200;
const MIN_VALUE_ETH: f64 = 0.01;

/// Controls arc visibility. Toggled with `V`.
#[derive(Resource)]
pub struct ArcSettings {
    pub enabled: bool,
}

impl Default for ArcSettings {
    fn default() -> Self {
        Self { enabled: true }
    }
}

pub fn arc_plugin(app: &mut App) {
    app.init_resource::<ArcSettings>()
        .add_systems(Update, (toggle_arcs_system, draw_arcs_system));
}

fn toggle_arcs_system(keys: Res<ButtonInput<KeyCode>>, mut settings: ResMut<ArcSettings>) {
    if keys.just_pressed(KeyCode::KeyV) {
        settings.enabled = !settings.enabled;
    }
}

fn draw_arcs_system(
    mut gizmos: Gizmos,
    settings: Res<ArcSettings>,
    selected: Res<SelectedEntity>,
    slabs: Query<&BlockSlab>,
    tx_cubes: Query<&TxCube>,
) {
    if !settings.enabled {
        return;
    }

    // Determine which block to show arcs for
    let selected_block = selected
        .entity
        .and_then(|e| slabs.get(e).ok())
        .map(|slab| slab.number);

    // Build address → centroid position map for the selected block
    let mut address_positions: HashMap<&str, (Vec3, u32)> = HashMap::new();
    let mut arcs_data: Vec<(&TxCube,)> = Vec::new();

    for tx_cube in tx_cubes.iter() {
        // Filter to selected block if one is selected
        if let Some(block_num) = selected_block {
            if tx_cube.block_number != block_num {
                continue;
            }
        } else {
            // No block selected: skip arcs entirely
            return;
        }

        // Accumulate positions for address centroid calculation
        if let Some(ref addr) = tx_cube.from {
            let entry = address_positions
                .entry(addr.as_str())
                .or_insert((Vec3::ZERO, 0));
            entry.0 += tx_cube.world_position;
            entry.1 += 1;
        }
        if let Some(ref addr) = tx_cube.to {
            let entry = address_positions
                .entry(addr.as_str())
                .or_insert((Vec3::ZERO, 0));
            entry.0 += tx_cube.world_position;
            entry.1 += 1;
        }

        arcs_data.push((tx_cube,));
    }

    // Compute centroids
    let centroids: HashMap<&str, Vec3> = address_positions
        .into_iter()
        .map(|(addr, (sum, count))| (addr, sum / count as f32))
        .collect();

    // Draw arcs
    let mut arc_count = 0;
    for (tx_cube,) in &arcs_data {
        if arc_count >= MAX_ARCS {
            break;
        }
        if tx_cube.value_eth < MIN_VALUE_ETH {
            continue;
        }

        let (Some(ref from_addr), Some(ref to_addr)) = (&tx_cube.from, &tx_cube.to) else {
            continue;
        };

        let (Some(&from_pos), Some(&to_pos)) = (
            centroids.get(from_addr.as_str()),
            centroids.get(to_addr.as_str()),
        ) else {
            continue;
        };

        if from_pos.distance(to_pos) < 0.01 {
            continue;
        }

        // Arc height based on value
        let arc_height = 1.0 + (tx_cube.value_eth as f32).log10().max(0.0) * 0.5;

        // Color: blue-to-gold by value magnitude
        let value_t = ((tx_cube.value_eth as f32).log10().clamp(-2.0, 2.0) + 2.0) / 4.0;
        let color = Color::srgb(
            0.2 + 0.8 * value_t,
            0.4 + 0.5 * value_t,
            1.0 - 0.8 * value_t,
        );

        // Draw bezier arc
        let mid = (from_pos + to_pos) / 2.0 + Vec3::Y * arc_height;
        let control1 = from_pos.lerp(mid, 0.5) + Vec3::Y * arc_height * 0.5;
        let control2 = mid.lerp(to_pos, 0.5) + Vec3::Y * arc_height * 0.5;

        let segments = 16;
        let mut prev = from_pos;
        for s in 1..=segments {
            let t = s as f32 / segments as f32;
            let point = cubic_bezier(from_pos, control1, control2, to_pos, t);
            gizmos.line(prev, point, color);
            prev = point;
        }

        arc_count += 1;
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
