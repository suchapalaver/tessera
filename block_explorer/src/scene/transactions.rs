//! Transaction cube component stored on spawned entities.

use alloy::primitives::Address;
use bevy::prelude::*;

#[derive(Component)]
pub struct TxCube {
    pub hash: String,
    pub tx_index: usize,
    pub gas: u64,
    pub gas_price: u128,
    pub value_eth: f64,
    pub from: Address,
    pub to: Option<Address>,
    pub block_number: u64,
    pub world_position: Vec3,
    pub blob_count: usize,
    pub max_fee_per_blob_gas: Option<u128>,
}
