//! CLI args, env parsing, and constants.

/// RPC URL from env (RPC_URL) or default local Anvil.
pub fn rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "http://127.0.0.1:8545".to_string())
}
