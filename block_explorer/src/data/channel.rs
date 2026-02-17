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
