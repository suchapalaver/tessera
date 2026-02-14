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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn lock_env() -> std::sync::MutexGuard<'static, ()> {
        ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    struct EnvGuard {
        snapshot: Vec<(&'static str, Option<String>)>,
    }

    impl EnvGuard {
        fn capture(keys: &[&'static str]) -> Self {
            let snapshot = keys
                .iter()
                .map(|&key| (key, std::env::var(key).ok()))
                .collect();
            Self { snapshot }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (key, value) in &self.snapshot {
                match value {
                    Some(val) => std::env::set_var(key, val),
                    None => std::env::remove_var(key),
                }
            }
        }
    }

    const ENV_KEYS: [&str; 5] = [
        "MAINNET_RPC_URL",
        "BASE_RPC_URL",
        "OPTIMISM_RPC_URL",
        "ARBITRUM_RPC_URL",
        "RPC_URL",
    ];

    #[test]
    fn chain_specific_env_takes_priority() {
        let _lock = lock_env();
        let _guard = EnvGuard::capture(&ENV_KEYS);

        std::env::set_var("MAINNET_RPC_URL", "http://127.0.0.1:8545");
        std::env::set_var("RPC_URL", "http://127.0.0.1:9999");

        let config = chain_config();

        assert_eq!(config.chain, Chain::mainnet());
        assert_eq!(config.rpc_url.as_str(), "http://127.0.0.1:8545/");
    }

    #[test]
    fn rpc_url_is_used_when_no_chain_envs_present() {
        let _lock = lock_env();
        let _guard = EnvGuard::capture(&ENV_KEYS);

        std::env::set_var("RPC_URL", "http://127.0.0.1:8545");

        let config = chain_config();

        assert_eq!(config.chain, Chain::mainnet());
        assert_eq!(config.rpc_url.as_str(), "http://127.0.0.1:8545/");
    }

    #[test]
    fn invalid_chain_env_falls_back_to_rpc_url() {
        let _lock = lock_env();
        let _guard = EnvGuard::capture(&ENV_KEYS);

        std::env::set_var("MAINNET_RPC_URL", "not-a-url");
        std::env::set_var("RPC_URL", "http://127.0.0.1:8545");

        let config = chain_config();

        assert_eq!(config.chain, Chain::mainnet());
        assert_eq!(config.rpc_url.as_str(), "http://127.0.0.1:8545/");
    }
}
