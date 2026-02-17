// Block and transaction payloads using Alloy primitive types for type safety.

use alloy::primitives::{Address, B256};
use alloy_chains::Chain;

/// A single block's summary and its transactions.
#[derive(Clone, Debug)]
pub struct BlockPayload {
    pub chain: Chain,
    pub number: u64,
    pub gas_used: u64,
    pub gas_limit: u64,
    pub timestamp: u64,
    pub tx_count: u32,
    pub base_fee_per_gas: Option<u64>,
    pub blob_gas_used: Option<u64>,
    pub transactions: Vec<TxPayload>,
    /// L1 block number this L2 block was derived from (OP Stack only).
    pub l1_origin_number: Option<u64>,
}

/// OP Stack L1 fee data extracted from transaction receipts.
#[derive(Clone, Debug)]
pub struct OpStackFees {
    pub l1_fee: u128,
    pub l1_gas_price: Option<u128>,
    pub l1_blob_base_fee: Option<u128>,
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
    /// OP Stack L1 fee data (present only for OP Stack L2 transactions).
    pub op_stack_fees: Option<OpStackFees>,
}
