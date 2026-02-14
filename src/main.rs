//! Tessera — block space explorer. Runs the block_explorer app.

use bevy::prelude::*;
use block_explorer::{
    arc_plugin, config, fly_camera_plugin, heatmap_plugin, hud_plugin, ingest_blocks,
    init_block_channel, inspector_plugin, setup_scene, timeline_plugin,
};

fn main() {
    let _ = dotenvy::dotenv();
    let config = config::chain_config();
    let channel = init_block_channel(config);

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Tessera".into(),
                resolution: (1280., 720.).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.08)))
        .insert_resource(channel)
        .add_plugins(fly_camera_plugin)
        .add_plugins(hud_plugin)
        .add_plugins(inspector_plugin)
        .add_plugins(timeline_plugin)
        .add_plugins(arc_plugin)
        .add_plugins(heatmap_plugin)
        .add_systems(Startup, setup_scene)
        .add_systems(Update, ingest_blocks)
        .run();
}
