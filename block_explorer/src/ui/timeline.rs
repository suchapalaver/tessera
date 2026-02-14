//! Timeline scrubber: bottom panel with block rectangles, playback controls.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::camera::CameraTarget;
use crate::scene::BlockRegistry;

/// Playback state for the timeline scrubber.
#[derive(Resource)]
pub struct TimelineState {
    pub playing: bool,
    pub speed: f32,
    pub current_index: usize,
    playback_timer: f32,
}

impl Default for TimelineState {
    fn default() -> Self {
        Self {
            playing: false,
            speed: 1.0,
            current_index: 0,
            playback_timer: 0.0,
        }
    }
}

pub fn timeline_plugin(app: &mut App) {
    app.init_resource::<TimelineState>()
        .add_systems(Update, (timeline_ui_system, playback_system));
}

fn timeline_ui_system(
    mut contexts: EguiContexts,
    registry: Res<BlockRegistry>,
    mut state: ResMut<TimelineState>,
    mut camera_target: ResMut<CameraTarget>,
) {
    if registry.entries.is_empty() {
        return;
    }

    egui::TopBottomPanel::bottom("timeline")
        .frame(
            egui::Frame::default()
                .fill(egui::Color32::from_rgba_premultiplied(15, 15, 25, 210))
                .inner_margin(egui::Margin::same(8))
                .corner_radius(egui::CornerRadius::same(0)),
        )
        .show(contexts.ctx_mut(), |ui| {
            ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
            ui.visuals_mut().override_text_color = Some(egui::Color32::from_rgb(200, 220, 240));

            ui.horizontal(|ui| {
                // Play/Pause button
                let label = if state.playing { "Pause" } else { "Play" };
                if ui.button(label).clicked() {
                    state.playing = !state.playing;
                }

                // Speed selector
                egui::ComboBox::from_id_salt("speed")
                    .selected_text(format!("{:.1}x", state.speed))
                    .width(50.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut state.speed, 0.5, "0.5x");
                        ui.selectable_value(&mut state.speed, 1.0, "1.0x");
                        ui.selectable_value(&mut state.speed, 2.0, "2.0x");
                        ui.selectable_value(&mut state.speed, 4.0, "4.0x");
                    });

                ui.separator();

                // Block number indicator
                if let Some(entry) = registry.entries.get(state.current_index) {
                    ui.label(
                        egui::RichText::new(format!("#{}", entry.number))
                            .color(egui::Color32::from_rgb(100, 220, 180)),
                    );
                }

                ui.separator();

                // Scrollable row of block rectangles
                egui::ScrollArea::horizontal()
                    .id_salt("timeline_blocks")
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 2.0;
                            for (i, entry) in registry.entries.iter().enumerate() {
                                let is_current = i == state.current_index;
                                let rect_width = 4.0 + 20.0 * entry.gas_fullness;
                                let rect_height = 20.0;

                                let (response, painter) = ui.allocate_painter(
                                    egui::vec2(rect_width, rect_height),
                                    egui::Sense::click(),
                                );

                                let color = if is_current {
                                    egui::Color32::from_rgb(100, 220, 180)
                                } else {
                                    fullness_color(entry.gas_fullness)
                                };

                                painter.rect_filled(response.rect, 2.0, color);

                                if response.clicked() {
                                    state.current_index = i;
                                    state.playing = false;
                                    jump_to_block(entry.z_position, &mut camera_target);
                                }

                                response.on_hover_text(format!(
                                    "#{} ({:.0}% full)",
                                    entry.number,
                                    entry.gas_fullness * 100.0,
                                ));
                            }
                        });
                    });
            });
        });
}

fn playback_system(
    time: Res<Time>,
    registry: Res<BlockRegistry>,
    mut state: ResMut<TimelineState>,
    mut camera_target: ResMut<CameraTarget>,
) {
    if !state.playing || registry.entries.is_empty() {
        return;
    }

    state.playback_timer += time.delta_secs() * state.speed;

    if state.playback_timer >= 1.0 {
        state.playback_timer = 0.0;

        if state.current_index + 1 < registry.entries.len() {
            state.current_index += 1;
            let z = registry.entries[state.current_index].z_position;
            jump_to_block(z, &mut camera_target);
        } else {
            state.playing = false;
        }
    }
}

fn jump_to_block(z_position: f32, camera_target: &mut CameraTarget) {
    camera_target.target = Some(Vec3::new(0.0, 5.0, z_position + 10.0));
    camera_target.look_at = Some(Vec3::new(0.0, 0.0, z_position));
}

fn fullness_color(fullness: f32) -> egui::Color32 {
    let r = (0.2 * 255.0) as u8;
    let g = ((0.2 + 0.5 * fullness) * 255.0) as u8;
    let b = (0.3 * 255.0) as u8;
    egui::Color32::from_rgb(r, g, b)
}
