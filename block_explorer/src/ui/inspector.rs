//! Entity inspector: click a block slab or transaction cube to see details.
//!
//! Uses manual ray-AABB intersection instead of Bevy's mesh picking to avoid
//! input absorption conflicts with bevy_egui.

use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use bevy_egui::{egui, EguiContexts};

use crate::scene::{BlockSlab, TxCube};

/// Tracks which entity is selected and its original material for highlight restore.
#[derive(Resource, Default)]
pub struct SelectedEntity {
    pub entity: Option<Entity>,
    original_material: Option<Handle<StandardMaterial>>,
}

pub fn inspector_plugin(app: &mut App) {
    app.init_resource::<SelectedEntity>().add_systems(
        Update,
        (
            click_raycast_system,
            inspector_panel_system,
            dismiss_selection_system,
        ),
    );
}

#[allow(clippy::too_many_arguments)]
fn click_raycast_system(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut contexts: EguiContexts,
    slabs: Query<(Entity, &GlobalTransform, &Aabb), With<BlockSlab>>,
    tx_cubes: Query<(Entity, &GlobalTransform, &Aabb), With<TxCube>>,
    material_query: Query<&MeshMaterial3d<StandardMaterial>>,
    mut selected: ResMut<SelectedEntity>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    if contexts.ctx_mut().is_pointer_over_area() {
        return;
    }

    let window = windows.single();
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let (camera, cam_transform) = cameras.single();
    let Ok(ray) = camera.viewport_to_world(cam_transform, cursor_pos) else {
        return;
    };

    let ray_origin = ray.origin;
    let ray_dir: Vec3 = *ray.direction;

    let mut best_hit: Option<(Entity, f32)> = None;

    // Check slabs
    for (entity, transform, aabb) in &slabs {
        if let Some(dist) = ray_aabb_test(ray_origin, ray_dir, transform, aabb) {
            if best_hit.is_none_or(|(_, d)| dist < d) {
                best_hit = Some((entity, dist));
            }
        }
    }

    // Check tx cubes — prefer cubes over slabs at similar distance (cubes sit on top)
    for (entity, transform, aabb) in &tx_cubes {
        if let Some(dist) = ray_aabb_test(ray_origin, ray_dir, transform, aabb) {
            if best_hit.is_none_or(|(_, d)| dist <= d) {
                best_hit = Some((entity, dist));
            }
        }
    }

    let Some((hit_entity, _)) = best_hit else {
        return;
    };

    let Ok(current_material) = material_query.get(hit_entity) else {
        return;
    };
    select_entity(
        hit_entity,
        &mut commands,
        current_material,
        &mut selected,
        &mut materials,
    );
}

fn ray_aabb_test(
    ray_origin: Vec3,
    ray_dir: Vec3,
    transform: &GlobalTransform,
    aabb: &Aabb,
) -> Option<f32> {
    let translation = transform.translation();
    let center: Vec3 = aabb.center.into();
    let half: Vec3 = aabb.half_extents.into();
    let aabb_min = translation + center - half;
    let aabb_max = translation + center + half;
    ray_aabb_intersect(ray_origin, ray_dir, aabb_min, aabb_max)
}

fn ray_aabb_intersect(origin: Vec3, dir: Vec3, aabb_min: Vec3, aabb_max: Vec3) -> Option<f32> {
    let inv_dir = 1.0 / dir;
    let t1 = (aabb_min - origin) * inv_dir;
    let t2 = (aabb_max - origin) * inv_dir;
    let t_min = t1.min(t2);
    let t_max = t1.max(t2);
    let t_enter = t_min.x.max(t_min.y).max(t_min.z);
    let t_exit = t_max.x.min(t_max.y).min(t_max.z);
    if t_enter <= t_exit && t_exit > 0.0 {
        Some(t_enter.max(0.0))
    } else {
        None
    }
}

fn select_entity(
    entity: Entity,
    commands: &mut Commands,
    current_material: &MeshMaterial3d<StandardMaterial>,
    selected: &mut SelectedEntity,
    materials: &mut Assets<StandardMaterial>,
) {
    let is_reselect = selected.entity == Some(entity);

    restore_material(commands, selected);

    if is_reselect {
        return;
    }

    selected.entity = Some(entity);
    selected.original_material = Some(current_material.0.clone());

    if let Some(mat_data) = materials.get(&current_material.0) {
        let mut highlight = mat_data.clone();
        highlight.emissive = LinearRgba::rgb(0.2, 0.8, 0.6);
        let handle = materials.add(highlight);
        commands.entity(entity).insert(MeshMaterial3d(handle));
    }
}

fn restore_material(commands: &mut Commands, selected: &mut SelectedEntity) {
    if let (Some(entity), Some(original)) =
        (selected.entity.take(), selected.original_material.take())
    {
        commands.entity(entity).insert(MeshMaterial3d(original));
    }
}

fn dismiss_selection_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut selected: ResMut<SelectedEntity>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        restore_material(&mut commands, &mut selected);
    }
}

fn inspector_panel_system(
    mut contexts: EguiContexts,
    selected: Res<SelectedEntity>,
    slabs: Query<&BlockSlab>,
    tx_cubes: Query<&TxCube>,
) {
    let Some(entity) = selected.entity else {
        return;
    };

    if let Ok(slab) = slabs.get(entity) {
        show_block_panel(&mut contexts, slab);
    } else if let Ok(tx) = tx_cubes.get(entity) {
        show_tx_panel(&mut contexts, tx);
    }
}

fn show_block_panel(contexts: &mut EguiContexts, slab: &BlockSlab) {
    let fullness = if slab.gas_limit > 0 {
        slab.gas_used as f32 / slab.gas_limit as f32
    } else {
        0.0
    };

    egui::SidePanel::right("inspector")
        .default_width(260.0)
        .frame(inspector_frame())
        .show(contexts.ctx_mut(), |ui| {
            apply_inspector_style(ui);

            ui.label(
                egui::RichText::new(format!("Block #{}", slab.number))
                    .size(18.0)
                    .color(egui::Color32::from_rgb(100, 220, 180)),
            );
            ui.add_space(8.0);

            ui.label(format!("Gas Used     {}", format_number(slab.gas_used)));
            ui.label(format!("Gas Limit    {}", format_number(slab.gas_limit)));
            ui.label(format!(
                "Fullness     {fullness:.1}%",
                fullness = fullness * 100.0
            ));
            ui.add_space(4.0);

            ui.label(format!("Transactions {}", slab.tx_count));
            ui.label(format!("Timestamp    {}", slab.timestamp));
            ui.add_space(12.0);

            dismiss_hint(ui);
        });
}

fn show_tx_panel(contexts: &mut EguiContexts, tx: &TxCube) {
    egui::SidePanel::right("inspector")
        .default_width(280.0)
        .frame(inspector_frame())
        .show(contexts.ctx_mut(), |ui| {
            apply_inspector_style(ui);

            ui.label(
                egui::RichText::new(format!("Tx #{}", tx.tx_index))
                    .size(18.0)
                    .color(egui::Color32::from_rgb(100, 220, 180)),
            );
            ui.add_space(4.0);

            ui.label(
                egui::RichText::new(format!("Block #{}", tx.block_number))
                    .size(12.0)
                    .color(egui::Color32::from_rgb(140, 160, 180)),
            );
            ui.add_space(8.0);

            if let Some(ref hash) = tx.hash {
                ui.label(format!("Hash  {}", abbreviate(hash, 10, 6)));
            }
            ui.add_space(4.0);

            if let Some(ref from) = tx.from {
                ui.label(format!("From  {}", abbreviate(from, 8, 6)));
            }
            if let Some(ref to) = tx.to {
                let display = if let Some(name) = known_contract_label(to) {
                    name.to_string()
                } else {
                    abbreviate(to, 8, 6)
                };
                ui.label(format!("To    {display}"));
            } else {
                ui.label(
                    egui::RichText::new("To    Contract Creation")
                        .color(egui::Color32::from_rgb(200, 180, 100)),
                );
            }
            ui.add_space(8.0);

            ui.label(format!("Value   {:.6} ETH", tx.value_eth));
            ui.label(format!(
                "Gas     {} ({:.2} gwei)",
                format_number(tx.gas),
                tx.gas_price as f64 / 1e9
            ));
            ui.add_space(4.0);

            if tx.blob_count > 0 {
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(format!("Blobs  {}", tx.blob_count))
                        .color(egui::Color32::from_rgb(160, 100, 220)),
                );
                if let Some(fee) = tx.max_fee_per_blob_gas {
                    ui.label(format!("Blob fee  {:.2} gwei", fee as f64 / 1e9));
                }
            }

            ui.add_space(12.0);
            dismiss_hint(ui);
        });
}

fn abbreviate(s: &str, prefix_len: usize, suffix_len: usize) -> String {
    if s.len() <= prefix_len + suffix_len + 2 {
        return s.to_string();
    }
    format!("{}..{}", &s[..prefix_len], &s[s.len() - suffix_len..])
}

fn inspector_frame() -> egui::Frame {
    egui::Frame::default()
        .fill(egui::Color32::from_rgba_premultiplied(15, 15, 25, 220))
        .inner_margin(egui::Margin::same(14))
}

fn apply_inspector_style(ui: &mut egui::Ui) {
    ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
    ui.visuals_mut().override_text_color = Some(egui::Color32::from_rgb(200, 220, 240));
}

fn dismiss_hint(ui: &mut egui::Ui) {
    ui.label(
        egui::RichText::new("Esc to dismiss")
            .size(11.0)
            .color(egui::Color32::from_rgb(120, 120, 140)),
    );
}

fn known_contract_label(address: &str) -> Option<&'static str> {
    let addr = address.to_lowercase();
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

fn format_number(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.2}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}
