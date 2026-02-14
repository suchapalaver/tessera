//! CLI args, env parsing, and constants.

use alloy_chains::{Chain, NamedChain};
use url::Url;

use crate::data::FetcherConfig;

const CHAIN_ENV_VARS: &[(NamedChain, &str)] = &[
    (NamedChain::Mainnet, "MAINNET_RPC_URL"),
    (NamedChain::Base, "BASE_RPC_URL"),
    (NamedChain::Optimism, "OPTIMISM_RPC_URL"),
    (NamedChain::Arbitrum, "ARBITRUM_RPC_URL"),
];

const DEFAULT_RPC: &str = "http://127.0.0.1:8545";

/// Returns the chain and RPC URL based on which env var is set.
/// Checks chain-specific vars first, falls back to RPC_URL → mainnet.
pub fn chain_config() -> FetcherConfig {
    for (named, env_var) in CHAIN_ENV_VARS {
        if let Ok(raw) = std::env::var(env_var) {
            if let Ok(url) = raw.parse::<Url>() {
                return FetcherConfig {
                    chain: Chain::from_named(*named),
                    rpc_url: url,
                };
            }
            eprintln!("tessera: invalid URL in {env_var}: {raw:?}");
        }
    }
    let raw = std::env::var("RPC_URL").unwrap_or_else(|_| DEFAULT_RPC.to_string());
    let url = raw.parse::<Url>().unwrap_or_else(|err| {
        panic!("tessera: invalid RPC_URL {raw:?}: {err}");
    });
    FetcherConfig {
        chain: Chain::mainnet(),
        rpc_url: url,
    }
}
