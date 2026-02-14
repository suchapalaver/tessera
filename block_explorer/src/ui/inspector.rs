//! Entity inspector: click a block slab to see details in a side panel.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::scene::BlockSlab;

/// Tracks which entity is selected and its original material for highlight restore.
#[derive(Resource, Default)]
pub struct SelectedEntity {
    pub entity: Option<Entity>,
    original_material: Option<Handle<StandardMaterial>>,
}

pub fn inspector_plugin(app: &mut App) {
    app.init_resource::<SelectedEntity>()
        .insert_resource(MeshPickingSettings {
            require_markers: true,
            ..default()
        })
        .add_observer(on_slab_click)
        .add_systems(Update, (inspector_panel_system, dismiss_selection_system));
}

fn on_slab_click(
    trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    slabs: Query<&MeshMaterial3d<StandardMaterial>, With<BlockSlab>>,
    mut selected: ResMut<SelectedEntity>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let entity = trigger.entity();

    let Ok(current_material) = slabs.get(entity) else {
        return;
    };

    let is_reselect = selected.entity == Some(entity);

    restore_material(&mut commands, &mut selected);

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
) {
    let Some(entity) = selected.entity else {
        return;
    };
    let Ok(slab) = slabs.get(entity) else {
        return;
    };

    let fullness = if slab.gas_limit > 0 {
        slab.gas_used as f32 / slab.gas_limit as f32
    } else {
        0.0
    };

    egui::SidePanel::right("inspector")
        .default_width(260.0)
        .frame(
            egui::Frame::default()
                .fill(egui::Color32::from_rgba_premultiplied(15, 15, 25, 220))
                .inner_margin(egui::Margin::same(14)),
        )
        .show(contexts.ctx_mut(), |ui| {
            ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
            ui.visuals_mut().override_text_color = Some(egui::Color32::from_rgb(200, 220, 240));

            ui.label(
                egui::RichText::new(format!("Block #{}", slab.number))
                    .size(18.0)
                    .color(egui::Color32::from_rgb(100, 220, 180)),
            );
            ui.add_space(8.0);

            ui.label(format!("Gas Used     {}", format_number(slab.gas_used)));
            ui.label(format!("Gas Limit    {}", format_number(slab.gas_limit)));
            ui.label(format!("Fullness     {:.1}%", fullness * 100.0));
            ui.add_space(4.0);

            ui.label(format!("Transactions {}", slab.tx_count));
            ui.label(format!("Timestamp    {}", slab.timestamp));
            ui.add_space(12.0);

            ui.label(
                egui::RichText::new("Esc to dismiss")
                    .size(11.0)
                    .color(egui::Color32::from_rgb(120, 120, 140)),
            );
        });
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
