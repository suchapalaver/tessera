//! HUD overlay: block stats, gas info, FPS counter.

use std::collections::VecDeque;

use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};

use alloy_chains::Chain;

use crate::data::BlockPayload;

const GAS_PRICE_WINDOW: usize = 10;

/// Live HUD state updated each time a block is ingested.
#[derive(Resource)]
pub struct HudState {
    pub chain: Option<Chain>,
    pub latest_block_number: u64,
    pub latest_gas_used: u64,
    pub latest_gas_limit: u64,
    pub latest_tx_count: u32,
    pub latest_timestamp: u64,
    pub blocks_rendered: u64,
    pub avg_gas_price_gwei: f64,
    pub base_fee_per_gas: Option<u64>,
    pub blob_gas_used: Option<u64>,
    gas_price_buffer: VecDeque<f64>,
}

impl Default for HudState {
    fn default() -> Self {
        Self {
            chain: None,
            latest_block_number: 0,
            latest_gas_used: 0,
            latest_gas_limit: 0,
            latest_tx_count: 0,
            latest_timestamp: 0,
            blocks_rendered: 0,
            avg_gas_price_gwei: 0.0,
            base_fee_per_gas: None,
            blob_gas_used: None,
            gas_price_buffer: VecDeque::new(),
        }
    }
}

impl HudState {
    pub fn update_from_block_entry(&mut self, entry: &crate::scene::BlockEntry) {
        self.chain = Some(entry.chain);
        self.latest_block_number = entry.number;
        self.latest_gas_used = entry.gas_used;
        self.latest_gas_limit = entry.gas_limit;
        self.latest_tx_count = entry.tx_count;
        self.latest_timestamp = entry.timestamp;
        self.base_fee_per_gas = entry.base_fee_per_gas;
        self.blob_gas_used = entry.blob_gas_used;
    }

    pub fn update_from_payload(&mut self, payload: &BlockPayload) {
        self.chain = Some(payload.chain);
        self.latest_block_number = payload.number;
        self.latest_gas_used = payload.gas_used;
        self.latest_gas_limit = payload.gas_limit;
        self.latest_tx_count = payload.tx_count;
        self.latest_timestamp = payload.timestamp;
        self.base_fee_per_gas = payload.base_fee_per_gas;
        self.blob_gas_used = payload.blob_gas_used;
        self.blocks_rendered += 1;

        if !payload.transactions.is_empty() {
            let avg_wei: f64 = payload
                .transactions
                .iter()
                .map(|tx| tx.gas_price as f64)
                .sum::<f64>()
                / payload.transactions.len() as f64;
            let avg_gwei = avg_wei / 1e9;
            self.gas_price_buffer.push_back(avg_gwei);
            if self.gas_price_buffer.len() > GAS_PRICE_WINDOW {
                self.gas_price_buffer.pop_front();
            }
            self.avg_gas_price_gwei =
                self.gas_price_buffer.iter().sum::<f64>() / self.gas_price_buffer.len() as f64;
        }
    }
}

pub fn hud_plugin(app: &mut App) {
    app.add_plugins(EguiPlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .init_resource::<HudState>()
        .add_systems(Update, hud_overlay_system);
}

fn hud_overlay_system(
    mut contexts: EguiContexts,
    hud: Res<HudState>,
    diagnostics: Res<DiagnosticsStore>,
    heatmap_state: Res<crate::scene::HeatmapState>,
    arc_settings: Res<crate::scene::arcs::ArcSettings>,
    blob_link_settings: Option<Res<crate::scene::blob_links::BlobLinkSettings>>,
) {
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    let fullness = if hud.latest_gas_limit > 0 {
        hud.latest_gas_used as f32 / hud.latest_gas_limit as f32
    } else {
        0.0
    };

    egui::Window::new("Block Explorer")
        .anchor(egui::Align2::LEFT_TOP, [10.0, 10.0])
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .frame(
            egui::Frame::default()
                .fill(egui::Color32::from_rgba_premultiplied(15, 15, 25, 210))
                .inner_margin(egui::Margin::same(12))
                .corner_radius(egui::CornerRadius::same(6)),
        )
        .show(contexts.ctx_mut(), |ui| {
            ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
            ui.visuals_mut().override_text_color = Some(egui::Color32::from_rgb(200, 220, 240));

            let chain_label = hud
                .chain
                .and_then(|c| c.named())
                .map(|n| n.to_string())
                .unwrap_or_default();
            let header = if chain_label.is_empty() {
                format!("Block #{}", hud.latest_block_number)
            } else {
                format!("{chain_label} #{}", hud.latest_block_number)
            };
            ui.label(
                egui::RichText::new(header)
                    .size(16.0)
                    .color(egui::Color32::from_rgb(100, 220, 180)),
            );
            ui.add_space(4.0);

            ui.label(format!(
                "Gas  {}/{}",
                format_gas(hud.latest_gas_used),
                format_gas(hud.latest_gas_limit)
            ));
            ui.add(
                egui::ProgressBar::new(fullness)
                    .text(format!("{:.1}%", fullness * 100.0))
                    .fill(egui::Color32::from_rgb(80, 180, 140)),
            );
            ui.add_space(4.0);

            ui.label(format!("Txns {}", hud.latest_tx_count));
            ui.label(format!("Avg gas price  {:.2} gwei", hud.avg_gas_price_gwei));
            if let Some(base_fee) = hud.base_fee_per_gas {
                ui.label(format!("Base fee  {:.2} gwei", base_fee as f64 / 1e9));
            }
            if let Some(blob_gas) = hud.blob_gas_used {
                if blob_gas > 0 {
                    ui.label(format!("Blob gas used  {}", format_gas(blob_gas)));
                }
            }
            ui.label(format!("Time {}", format_timestamp(hud.latest_timestamp)));
            ui.add_space(4.0);

            ui.separator();
            ui.label(format!("Blocks rendered  {}", hud.blocks_rendered));
            ui.label(format!("FPS  {fps:.0}"));

            ui.add_space(4.0);
            ui.separator();
            let heatmap_label = if heatmap_state.enabled {
                "[H] Heatmap ON"
            } else {
                "[H] Heatmap OFF"
            };
            ui.label(
                egui::RichText::new(heatmap_label)
                    .size(11.0)
                    .color(egui::Color32::from_rgb(120, 160, 140)),
            );
            let arc_label = if arc_settings.enabled {
                "[V] Arcs ON"
            } else {
                "[V] Arcs OFF"
            };
            ui.label(
                egui::RichText::new(arc_label)
                    .size(11.0)
                    .color(egui::Color32::from_rgb(120, 160, 140)),
            );
            let blob_label = match blob_link_settings.as_ref() {
                Some(s) if s.enabled => "[B] Blob links ON",
                Some(_) => "[B] Blob links OFF",
                None => "[B] Blob links",
            };
            ui.label(
                egui::RichText::new(blob_label)
                    .size(11.0)
                    .color(egui::Color32::from_rgb(120, 160, 140)),
            );
        });
}

fn format_gas(gas: u64) -> String {
    if gas >= 1_000_000 {
        format!("{:.1}M", gas as f64 / 1_000_000.0)
    } else if gas >= 1_000 {
        format!("{:.1}K", gas as f64 / 1_000.0)
    } else {
        gas.to_string()
    }
}

fn format_timestamp(ts: u64) -> String {
    let secs = ts % 60;
    let mins = (ts / 60) % 60;
    let hours = (ts / 3600) % 24;
    format!("{hours:02}:{mins:02}:{secs:02} UTC")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_gas_formats_units() {
        assert_eq!(format_gas(999), "999");
        assert_eq!(format_gas(1_500), "1.5K");
        assert_eq!(format_gas(2_000_000), "2.0M");
    }

    #[test]
    fn format_timestamp_formats_hh_mm_ss() {
        assert_eq!(format_timestamp(0), "00:00:00 UTC");
        assert_eq!(format_timestamp(3661), "01:01:01 UTC");
    }
}
