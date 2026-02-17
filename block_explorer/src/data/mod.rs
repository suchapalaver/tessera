mod channel;
pub mod evm;
mod model;
#[allow(dead_code)]
mod solana;

use alloy_chains::{Chain, NamedChain};
use crossbeam_channel::Receiver;
use url::Url;

pub use channel::{init_block_channel, init_multi_chain_channel, BlockChannel};
pub use model::{BlockPayload, OpStackFees, TxPayload};

/// Returns true if the chain is an OP Stack L2 (Base, Optimism).
pub fn is_op_stack(chain: &Chain) -> bool {
    matches!(chain.named(), Some(NamedChain::Base | NamedChain::Optimism))
}

/// Configuration for spawning a chain fetcher.
pub struct FetcherConfig {
    pub chain: Chain,
    pub rpc_url: Url,
}

/// Interface for chain-specific block fetchers.
pub trait ChainFetcher: Send + 'static {
    fn spawn(config: FetcherConfig) -> Receiver<BlockPayload>;
}
