//! Block-number labels rendered as textured quads on all four vertical slab faces.

use std::f32::consts::{FRAC_PI_2, PI};

use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

const GLYPH_W: u32 = 5;
const GLYPH_H: u32 = 7;
const GLYPH_PAD: u32 = 1;
const FACE_MARGIN: f32 = 0.85;
const FACE_OFFSET: f32 = 0.02;

fn char_bitmap(c: char) -> Option<[u8; 35]> {
    #[rustfmt::skip]
    let bmp: [u8; 35] = match c {
        '#' => [
            0,1,0,1,0,
            1,1,1,1,1,
            0,1,0,1,0,
            0,1,0,1,0,
            0,1,0,1,0,
            1,1,1,1,1,
            0,1,0,1,0,
        ],
        '0' => [
            0,1,1,1,0,
            1,0,0,0,1,
            1,0,0,1,1,
            1,0,1,0,1,
            1,1,0,0,1,
            1,0,0,0,1,
            0,1,1,1,0,
        ],
        '1' => [
            0,0,1,0,0,
            0,1,1,0,0,
            0,0,1,0,0,
            0,0,1,0,0,
            0,0,1,0,0,
            0,0,1,0,0,
            0,1,1,1,0,
        ],
        '2' => [
            0,1,1,1,0,
            1,0,0,0,1,
            0,0,0,0,1,
            0,0,1,1,0,
            0,1,0,0,0,
            1,0,0,0,0,
            1,1,1,1,1,
        ],
        '3' => [
            0,1,1,1,0,
            1,0,0,0,1,
            0,0,0,0,1,
            0,0,1,1,0,
            0,0,0,0,1,
            1,0,0,0,1,
            0,1,1,1,0,
        ],
        '4' => [
            1,0,0,0,1,
            1,0,0,0,1,
            1,0,0,0,1,
            1,1,1,1,1,
            0,0,0,0,1,
            0,0,0,0,1,
            0,0,0,0,1,
        ],
        '5' => [
            1,1,1,1,1,
            1,0,0,0,0,
            1,1,1,1,0,
            0,0,0,0,1,
            0,0,0,0,1,
            1,0,0,0,1,
            0,1,1,1,0,
        ],
        '6' => [
            0,1,1,1,0,
            1,0,0,0,0,
            1,0,0,0,0,
            1,1,1,1,0,
            1,0,0,0,1,
            1,0,0,0,1,
            0,1,1,1,0,
        ],
        '7' => [
            1,1,1,1,1,
            0,0,0,0,1,
            0,0,0,1,0,
            0,0,1,0,0,
            0,0,1,0,0,
            0,0,1,0,0,
            0,0,1,0,0,
        ],
        '8' => [
            0,1,1,1,0,
            1,0,0,0,1,
            1,0,0,0,1,
            0,1,1,1,0,
            1,0,0,0,1,
            1,0,0,0,1,
            0,1,1,1,0,
        ],
        '9' => [
            0,1,1,1,0,
            1,0,0,0,1,
            1,0,0,0,1,
            0,1,1,1,1,
            0,0,0,0,1,
            0,0,0,1,0,
            0,1,1,0,0,
        ],
        _ => return None,
    };
    Some(bmp)
}

fn render_label_image(text: &str) -> Image {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len() as u32;
    let img_w = n * GLYPH_W + n.saturating_sub(1) * GLYPH_PAD;
    let img_h = GLYPH_H;
    let mut data = vec![0u8; (img_w * img_h * 4) as usize];

    for (i, &c) in chars.iter().enumerate() {
        let Some(bmp) = char_bitmap(c) else {
            continue;
        };
        let x_off = i as u32 * (GLYPH_W + GLYPH_PAD);
        for row in 0..GLYPH_H {
            for col in 0..GLYPH_W {
                if bmp[(row * GLYPH_W + col) as usize] == 1 {
                    let px = x_off + col;
                    let py = row;
                    let idx = ((py * img_w + px) * 4) as usize;
                    data[idx] = 150;
                    data[idx + 1] = 230;
                    data[idx + 2] = 190;
                    data[idx + 3] = 255;
                }
            }
        }
    }

    let mut image = Image::new(
        Extent3d {
            width: img_w,
            height: img_h,
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

/// Fits a quad to the given face dimensions while preserving the text aspect ratio.
fn fit_quad(face_w: f32, img_aspect: f32) -> (f32, f32) {
    let w = face_w * FACE_MARGIN;
    let h = w / img_aspect;
    if h > FACE_MARGIN {
        (FACE_MARGIN * img_aspect, FACE_MARGIN)
    } else {
        (w, h)
    }
}

pub fn spawn_block_labels(
    commands: &mut Commands,
    images: &mut ResMut<Assets<Image>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    meshes: &mut ResMut<Assets<Mesh>>,
    block_number: u64,
    slab_z: f32,
    slab_width: f32,
) {
    let text = format!("#{block_number}");
    let char_count = text.len() as u32;
    let img_w = char_count * GLYPH_W + char_count.saturating_sub(1) * GLYPH_PAD;
    let img_aspect = img_w as f32 / GLYPH_H as f32;

    let image = render_label_image(&text);
    let img_handle = images.add(image);
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(img_handle),
        unlit: true,
        alpha_mode: AlphaMode::Mask(0.5),
        ..default()
    });

    let hw = slab_width / 2.0;
    let hd = 1.0;

    let (fb_w, fb_h) = fit_quad(slab_width, img_aspect);
    let fb_mesh = meshes.add(Rectangle::new(fb_w, fb_h));

    let (sd_w, sd_h) = fit_quad(2.0, img_aspect);
    let sd_mesh = meshes.add(Rectangle::new(sd_w, sd_h));

    let pos = Vec3::new(0.0, 0.0, slab_z);

    // Front (+Z)
    commands.spawn((
        Mesh3d(fb_mesh.clone()),
        MeshMaterial3d(material.clone()),
        Transform::from_translation(pos + Vec3::new(0.0, 0.0, hd + FACE_OFFSET)),
    ));
    // Back (-Z)
    commands.spawn((
        Mesh3d(fb_mesh),
        MeshMaterial3d(material.clone()),
        Transform::from_translation(pos + Vec3::new(0.0, 0.0, -hd - FACE_OFFSET))
            .with_rotation(Quat::from_rotation_y(PI)),
    ));
    // Right (+X)
    commands.spawn((
        Mesh3d(sd_mesh.clone()),
        MeshMaterial3d(material.clone()),
        Transform::from_translation(pos + Vec3::new(hw + FACE_OFFSET, 0.0, 0.0))
            .with_rotation(Quat::from_rotation_y(FRAC_PI_2)),
    ));
    // Left (-X)
    commands.spawn((
        Mesh3d(sd_mesh),
        MeshMaterial3d(material),
        Transform::from_translation(pos + Vec3::new(-hw - FACE_OFFSET, 0.0, 0.0))
            .with_rotation(Quat::from_rotation_y(-FRAC_PI_2)),
    ));
}
