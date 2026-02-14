use crossbeam_channel::Receiver;

use crate::data::evm::EvmFetcher;
use crate::data::model::BlockPayload;
use crate::data::{ChainFetcher, FetcherConfig};

/// Bevy resource holding the channel from the EVM fetcher thread.
/// Systems drain this in ingest_blocks.
#[derive(bevy::prelude::Resource)]
pub struct BlockChannel(pub Receiver<BlockPayload>);

/// Create a block channel and spawn the EVM fetcher on a dedicated thread.
/// Returns the resource to insert into the app.
pub fn init_block_channel(config: FetcherConfig) -> BlockChannel {
    let rx = EvmFetcher::spawn(config);
    BlockChannel(rx)
}
