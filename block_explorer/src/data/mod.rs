mod channel;
pub mod evm;
mod model;
#[allow(dead_code)]
mod solana;

use alloy_chains::Chain;
use crossbeam_channel::Receiver;
use url::Url;

pub use channel::{init_block_channel, BlockChannel};
pub use model::{BlockPayload, TxPayload};

/// Configuration for spawning a chain fetcher.
pub struct FetcherConfig {
    pub chain: Chain,
    pub rpc_url: Url,
}

/// Interface for chain-specific block fetchers.
pub trait ChainFetcher: Send + 'static {
    fn spawn(config: FetcherConfig) -> Receiver<BlockPayload>;
}
