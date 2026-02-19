use std::path::Path;

use crossbeam_channel::Receiver;

use crate::data::evm::EvmFetcher;
use crate::data::model::BlockPayload;
use crate::data::{ChainFetcher, FetcherConfig};

/// Bevy resource holding the channel from the EVM fetcher thread.
/// Systems drain this in ingest_blocks.
#[derive(bevy::prelude::Resource)]
pub struct BlockChannel(pub Receiver<BlockPayload>);

/// Create a block channel and spawn the EVM fetcher on a dedicated thread.
pub fn init_block_channel(config: FetcherConfig) -> BlockChannel {
    init_multi_chain_channel(vec![config])
}

/// Spawn one fetcher per config and fan them into a single receiver.
/// Each source gets its own forwarding thread so payloads from all chains
/// arrive in a single channel that the ECS drains each frame.
pub fn init_multi_chain_channel(configs: Vec<FetcherConfig>) -> BlockChannel {
    assert!(!configs.is_empty(), "at least one chain config is required");

    if configs.len() == 1 {
        let rx = EvmFetcher::spawn(configs.into_iter().next().unwrap());
        return BlockChannel(rx);
    }

    let (fan_tx, fan_rx) = crossbeam_channel::bounded(64);

    for config in configs {
        let tx = fan_tx.clone();
        let rx = EvmFetcher::spawn(config);
        std::thread::spawn(move || {
            while let Ok(payload) = rx.recv() {
                if tx.send(payload).is_err() {
                    return;
                }
            }
        });
    }

    BlockChannel(fan_rx)
}

/// Bevy resource that records ingested payloads for later serialization to a fixture file.
#[derive(bevy::prelude::Resource)]
pub struct RecordBuffer {
    pub payloads: Vec<BlockPayload>,
    pub path: std::path::PathBuf,
}

impl RecordBuffer {
    pub fn new(path: std::path::PathBuf) -> Self {
        Self {
            payloads: Vec::new(),
            path,
        }
    }

    /// Serialize accumulated payloads to the target path as JSON.
    pub fn flush(&self) {
        let json = serde_json::to_string_pretty(&self.payloads)
            .expect("failed to serialize record buffer");
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(&self.path, json)
            .unwrap_or_else(|e| panic!("failed to write fixture to {}: {e}", self.path.display()));
        eprintln!(
            "tessera: recorded {} blocks to {}",
            self.payloads.len(),
            self.path.display()
        );
    }
}

/// Create a block channel that replays pre-recorded payloads from a JSON fixture file.
/// Payloads are sent with a 50ms delay between each to simulate realistic ingestion pacing.
pub fn init_fixture_channel(path: &Path) -> BlockChannel {
    let json = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read fixture {}: {e}", path.display()));
    let payloads: Vec<BlockPayload> = serde_json::from_str(&json)
        .unwrap_or_else(|e| panic!("failed to parse fixture {}: {e}", path.display()));

    let (tx, rx) = crossbeam_channel::bounded(64);

    std::thread::spawn(move || {
        for payload in payloads {
            if tx.send(payload).is_err() {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    });

    BlockChannel(rx)
}
