// Block and transaction payloads using Alloy primitive types for type safety.

use alloy::primitives::{Address, B256};

/// A single block's summary and its transactions.
#[derive(Clone, Debug)]
pub struct BlockPayload {
    pub number: u64,
    pub gas_used: u64,
    pub gas_limit: u64,
    pub timestamp: u64,
    pub tx_count: u32,
    pub base_fee_per_gas: Option<u64>,
    pub blob_gas_used: Option<u64>,
    pub transactions: Vec<TxPayload>,
}

/// A single transaction's display-relevant fields.
#[derive(Clone, Debug)]
pub struct TxPayload {
    pub hash: B256,
    pub tx_index: usize,
    pub gas: u64,
    pub gas_price: u128,
    pub value_eth: f64,
    pub from: Address,
    pub to: Option<Address>,
    pub blob_count: usize,
    pub max_fee_per_blob_gas: Option<u128>,
}
