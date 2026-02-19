// Block and transaction payloads using Alloy primitive types for type safety.

use alloy::primitives::{Address, B256};
use alloy_chains::Chain;
use serde::{Deserialize, Serialize};

/// A single block's summary and its transactions.
#[derive(Clone, Debug, Serialize, Deserialize)]
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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpStackFees {
    pub l1_fee: u128,
    pub l1_gas_price: Option<u128>,
    pub l1_blob_base_fee: Option<u128>,
}

/// A single transaction's display-relevant fields.
#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_block() -> BlockPayload {
        BlockPayload {
            chain: Chain::mainnet(),
            number: 18_000_000,
            gas_used: 12_000_000,
            gas_limit: 30_000_000,
            timestamp: 1_700_000_000,
            tx_count: 2,
            base_fee_per_gas: Some(30_000_000_000),
            blob_gas_used: Some(131_072),
            l1_origin_number: None,
            transactions: vec![
                TxPayload {
                    hash: B256::ZERO,
                    tx_index: 0,
                    gas: 21_000,
                    gas_price: 30_000_000_000,
                    value_eth: 1.5,
                    from: Address::ZERO,
                    to: Some(Address::ZERO),
                    blob_count: 0,
                    max_fee_per_blob_gas: None,
                    op_stack_fees: None,
                },
                TxPayload {
                    hash: B256::ZERO,
                    tx_index: 1,
                    gas: 100_000,
                    gas_price: 50_000_000_000,
                    value_eth: 0.0,
                    from: Address::ZERO,
                    to: None,
                    blob_count: 3,
                    max_fee_per_blob_gas: Some(1_000_000_000),
                    op_stack_fees: Some(OpStackFees {
                        l1_fee: 5_000_000_000_000,
                        l1_gas_price: Some(20_000_000_000),
                        l1_blob_base_fee: Some(1_000_000),
                    }),
                },
            ],
        }
    }

    #[test]
    fn serde_round_trip() {
        let block = sample_block();
        let json = serde_json::to_string(&block).expect("serialize");
        let deserialized: BlockPayload = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(block.chain, deserialized.chain);
        assert_eq!(block.number, deserialized.number);
        assert_eq!(block.gas_used, deserialized.gas_used);
        assert_eq!(block.transactions.len(), deserialized.transactions.len());
        assert_eq!(
            block.transactions[1].op_stack_fees.as_ref().unwrap().l1_fee,
            deserialized.transactions[1]
                .op_stack_fees
                .as_ref()
                .unwrap()
                .l1_fee
        );
    }

    #[test]
    fn serde_round_trip_vec() {
        let blocks = vec![sample_block(), sample_block()];
        let json = serde_json::to_string_pretty(&blocks).expect("serialize vec");
        let deserialized: Vec<BlockPayload> = serde_json::from_str(&json).expect("deserialize vec");
        assert_eq!(blocks.len(), deserialized.len());
    }
}
