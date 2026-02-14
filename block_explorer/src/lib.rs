//! Block space explorer — 3D visualization of EVM blocks and transactions.
//!
//! Library root: data, scene, camera, UI, and config modules.

pub mod camera;
pub mod config;
pub mod data;
pub mod scene;
pub mod ui;

pub use camera::fly_camera_plugin;
pub use data::{init_block_channel, BlockChannel, BlockPayload, TxPayload};
pub use scene::{ingest_blocks, setup_scene, BlockSlab, ExplorerState, TxCube};
pub use ui::{hud_plugin, inspector_plugin, timeline_plugin};
