//! Block space explorer — 3D visualization of EVM blocks and transactions.
//!
//! Library root: data, SDK builder, and config modules.

mod camera;
pub mod config;
pub mod data;
pub mod render;
mod scene;
mod ui;

pub mod prelude;
pub mod sdk;

pub use data::evm::EvmFetcher;
pub use data::{BlockPayload, ChainFetcher, FetcherConfig, TxPayload};
