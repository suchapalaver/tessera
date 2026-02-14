//! Well-known Ethereum contract registry.

use alloy::primitives::{address, Address};

/// Well-known contract addresses and their human-readable names.
const KNOWN_CONTRACTS: &[(Address, &str)] = &[
    (address!("dac17f958d2ee523a2206206994597c13d831ec7"), "USDT"),
    (address!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"), "USDC"),
    (address!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"), "WETH"),
    (
        address!("7a250d5630b4cf539739df2c5dacb4c659f2488d"),
        "UniV2Router",
    ),
    (
        address!("e592427a0aece92de3edee1f18e0157c05861564"),
        "UniV3Router",
    ),
    (
        address!("68b3465833fb72a70ecdf485e0e4c7bd8665fc45"),
        "UniRouter2",
    ),
    (
        address!("3fc91a3afd70395cd496c647d5a6cc9d4b2b7fad"),
        "UniRouter",
    ),
    (
        address!("1111111254eeb25477b68fb85ed929f73a960582"),
        "1inch",
    ),
    (
        address!("881d40237659c251811cec9c364ef91dc08d300c"),
        "Metamask",
    ),
    (
        address!("7d1afa7b718fb893db30a3abc0cfc608aacfebb0"),
        "MATIC",
    ),
];

/// Returns a human-readable name for well-known contracts, or `None`.
pub fn known_contract_name(addr: &Address) -> Option<&'static str> {
    KNOWN_CONTRACTS
        .iter()
        .find(|(a, _)| a == addr)
        .map(|(_, name)| *name)
}
