//! Tessera — block space explorer. Runs the block_explorer app.

use bevy::prelude::*;
use block_explorer::{config, fly_camera_plugin, ingest_blocks, init_block_channel, setup_scene};

fn main() {
    let _ = dotenvy::dotenv();
    let rpc_url = config::rpc_url();
    let channel = init_block_channel(&rpc_url);

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
        .add_systems(Startup, setup_scene)
        .add_systems(Update, ingest_blocks)
        .run();
}
