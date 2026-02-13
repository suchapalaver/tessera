use crossbeam_channel::Receiver;

use crate::data::model::BlockPayload;

/// Bevy resource holding the channel from the EVM fetcher thread.
/// Systems drain this in ingest_blocks.
#[derive(bevy::prelude::Resource)]
pub struct BlockChannel(pub Receiver<BlockPayload>);

/// Create a block channel and spawn the EVM fetcher on a dedicated thread.
/// Returns the resource to insert into the app.
pub fn init_block_channel(rpc_url: &str) -> BlockChannel {
    let rx = crate::data::evm::spawn_evm_fetcher(rpc_url.to_string());
    BlockChannel(rx)
}
