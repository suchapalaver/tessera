// Chain-agnostic block and transaction payloads.
// Alloy-specific types stay in evm.rs; conversion happens there.

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
    pub hash: Option<String>,
    pub tx_index: usize,
    pub gas: u64,
    pub gas_price: u64,
    pub value_eth: f64,
    pub from: Option<String>,
    pub to: Option<String>,
    pub blob_count: usize,
    pub max_fee_per_blob_gas: Option<u64>,
}

// Both are used across thread boundaries (fetcher → channel → ECS).
impl BlockPayload {}
impl TxPayload {}
