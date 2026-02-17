//! Shared material and color helpers for slabs and tx cubes.

use crate::data::TxPayload;
use bevy::prelude::*;

pub fn block_slab_material_with_fullness(
    materials: &mut ResMut<Assets<StandardMaterial>>,
    fullness: f32,
) -> Handle<StandardMaterial> {
    let g = 0.2 + 0.5 * fullness;
    materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, g, 0.3),
        ..default()
    })
}

pub fn tx_cube_material(
    materials: &mut ResMut<Assets<StandardMaterial>>,
    tx: &TxPayload,
    tx_count: usize,
) -> Handle<StandardMaterial> {
    let gwei = tx.gas_price as f64 / 1e9;
    let color = gas_price_color(gwei);

    // Position-based brightness: first tx = full, last tx = 40%
    let brightness = if tx_count > 1 {
        1.0 - 0.6 * (tx.tx_index as f32 / (tx_count - 1) as f32)
    } else {
        1.0
    };
    let lin = color.to_linear();
    let modulated = Color::linear_rgb(
        lin.red * brightness,
        lin.green * brightness,
        lin.blue * brightness,
    );

    let emissive = if tx.value_eth > 1.0 {
        let m = modulated.to_linear();
        Color::linear_rgb(m.red * 5.0, m.green * 5.0, m.blue * 5.0)
    } else {
        Color::BLACK
    };

    materials.add(StandardMaterial {
        base_color: modulated,
        emissive: emissive.into(),
        ..default()
    })
}

/// Generates a heatmap image from transaction gas prices.
/// Each pixel column represents one transaction, colored by gas price.
pub(crate) fn generate_heatmap_image(txs: &[TxPayload]) -> Image {
    use bevy::image::ImageSampler;
    use bevy::render::render_asset::RenderAssetUsages;
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

    let width = txs.len().max(1) as u32;
    let height: u32 = 16;
    let mut data = vec![0u8; (width * height * 4) as usize];

    for (i, tx) in txs.iter().enumerate() {
        let gwei = tx.gas_price as f64 / 1e9;
        let color = gas_price_color(gwei);
        let lin = color.to_linear();
        let r = (lin.red * 255.0) as u8;
        let g = (lin.green * 255.0) as u8;
        let b = (lin.blue * 255.0) as u8;

        for row in 0..height {
            let idx = ((row * width + i as u32) * 4) as usize;
            data[idx] = r;
            data[idx + 1] = g;
            data[idx + 2] = b;
            data[idx + 3] = 255;
        }
    }

    let mut image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::nearest();
    image
}

/// Blue → Cyan → Yellow → Red gradient mapped to 0–200 gwei.
fn gas_price_color(gwei: f64) -> Color {
    let t = (gwei / 200.0).clamp(0.0, 1.0) as f32;

    if t < 0.33 {
        let s = t / 0.33;
        Color::srgb(0.0, s, 1.0 - s * 0.5)
    } else if t < 0.66 {
        let s = (t - 0.33) / 0.33;
        Color::srgb(s, 1.0, 0.5 * (1.0 - s))
    } else {
        let s = (t - 0.66) / 0.34;
        Color::srgb(1.0, 1.0 - s, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{Address, B256};

    fn tx_with_gas(gwei: u64, tx_index: usize) -> TxPayload {
        TxPayload {
            hash: B256::ZERO,
            tx_index,
            gas: 21_000,
            gas_price: (gwei as u128) * 1_000_000_000u128,
            value_eth: 0.0,
            from: Address::ZERO,
            to: None,
            blob_count: 0,
            max_fee_per_blob_gas: None,
            op_stack_fees: None,
        }
    }

    #[test]
    fn heatmap_image_has_expected_size_and_colors() {
        let txs = vec![tx_with_gas(0, 0), tx_with_gas(200, 1)];
        let image = generate_heatmap_image(&txs);

        let width = image.texture_descriptor.size.width as usize;
        let height = image.texture_descriptor.size.height as usize;

        assert_eq!(width, 2);
        assert_eq!(height, 16);
        assert_eq!(image.data.len(), width * height * 4);

        let first = &image.data[0..4];
        let second = &image.data[4..8];

        assert_eq!(first, &[0, 0, 255, 255]);
        assert_eq!(second, &[255, 0, 0, 255]);
    }
}
