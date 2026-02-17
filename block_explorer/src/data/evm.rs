//! EVM block fetcher: dedicated thread + alloy → BlockPayload.

use alloy::eips::BlockNumberOrTag;
use alloy::primitives::{address, Address};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::BlockTransactions;
use alloy_chains::Chain;
use crossbeam_channel::{Receiver, Sender};
use std::thread;
use std::time::Duration;
use url::Url;

use crate::data::model::{BlockPayload, TxPayload};
use crate::data::{ChainFetcher, FetcherConfig};

const BACKFILL_COUNT: u64 = 20;
const POLL_INTERVAL: Duration = Duration::from_secs(2);

/// L1Block predeploy contract on OP Stack L2s.
const L1_BLOCK_PREDEPLOY: Address = address!("4200000000000000000000000000000000000015");

/// EVM-compatible block fetcher using Alloy.
pub struct EvmFetcher;

impl ChainFetcher for EvmFetcher {
    fn spawn(config: FetcherConfig) -> Receiver<BlockPayload> {
        let (tx, rx) = crossbeam_channel::bounded(64);
        let is_op = crate::data::is_op_stack(&config.chain);
        thread::spawn(move || {
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(err) => {
                    eprintln!("tessera: failed to build tokio runtime: {err}");
                    return;
                }
            };
            if is_op {
                rt.block_on(op_stack_fetcher_loop(config.chain, config.rpc_url, tx));
            } else {
                rt.block_on(fetcher_loop(config.chain, config.rpc_url, tx));
            }
        });
        rx
    }
}

// ---------------------------------------------------------------------------
// Standard (L1) fetcher
// ---------------------------------------------------------------------------

async fn fetcher_loop(chain: Chain, rpc_url: Url, tx: Sender<BlockPayload>) {
    let provider = ProviderBuilder::new().connect_http(rpc_url);

    let latest = match provider.get_block_number().await {
        Ok(n) => n,
        Err(err) => {
            eprintln!("tessera [{chain}]: failed to get latest block number: {err}");
            return;
        }
    };

    let start = latest.saturating_sub(BACKFILL_COUNT - 1);
    eprintln!("tessera [{chain}]: backfilling blocks {start}..={latest}");

    for n in start..=latest {
        if fetch_and_send(&provider, chain, n, &tx).await.is_err() {
            return;
        }
    }

    eprintln!("tessera [{chain}]: backfill complete, polling for new blocks");

    let mut last_seen = latest;
    loop {
        tokio::time::sleep(POLL_INTERVAL).await;

        let tip = match provider.get_block_number().await {
            Ok(n) => n,
            Err(err) => {
                eprintln!("tessera [{chain}]: poll error: {err}");
                continue;
            }
        };

        for n in (last_seen + 1)..=tip {
            if fetch_and_send(&provider, chain, n, &tx).await.is_err() {
                return;
            }
        }
        last_seen = tip;
    }
}

async fn fetch_and_send(
    provider: &impl Provider,
    chain: Chain,
    number: u64,
    tx: &Sender<BlockPayload>,
) -> Result<(), ()> {
    let block = match provider
        .get_block_by_number(BlockNumberOrTag::Number(number))
        .full()
        .await
    {
        Ok(Some(block)) => block,
        Ok(None) => {
            eprintln!("tessera: block {number} not found");
            return Ok(());
        }
        Err(err) => {
            eprintln!("tessera: failed to fetch block {number}: {err}");
            return Ok(());
        }
    };

    let payload = block_to_payload(chain, &block);
    eprintln!(
        "tessera [{chain}]: block {} ({} txs, gas {}/{})",
        payload.number, payload.tx_count, payload.gas_used, payload.gas_limit
    );
    tx.send(payload).map_err(|_| ())
}

fn block_to_payload(chain: Chain, block: &alloy::rpc::types::Block) -> BlockPayload {
    let header = &block.header;

    let transactions: Vec<TxPayload> = match &block.transactions {
        BlockTransactions::Full(txs) => txs
            .iter()
            .enumerate()
            .map(|(i, tx)| tx_to_payload(i, tx))
            .collect(),
        _ => Vec::new(),
    };

    BlockPayload {
        chain,
        number: header.number,
        gas_used: header.gas_used,
        gas_limit: header.gas_limit,
        timestamp: header.timestamp,
        tx_count: transactions.len() as u32,
        base_fee_per_gas: header.base_fee_per_gas,
        blob_gas_used: header.blob_gas_used,
        transactions,
        l1_origin_number: None,
    }
}

fn tx_to_payload(index: usize, tx: &alloy::rpc::types::Transaction) -> TxPayload {
    use alloy::consensus::Transaction as TxConsensus;
    use alloy::network::TransactionResponse;

    let blob_count = TxConsensus::blob_versioned_hashes(tx).map_or(0, |h| h.len());

    TxPayload {
        hash: tx.tx_hash(),
        tx_index: index,
        gas: tx.gas_limit(),
        gas_price: TxConsensus::gas_price(tx).unwrap_or(0),
        value_eth: wei_to_eth(tx.value()),
        from: TransactionResponse::from(tx),
        to: tx.to(),
        blob_count,
        max_fee_per_blob_gas: TxConsensus::max_fee_per_blob_gas(tx),
        op_stack_fees: None,
    }
}

// ---------------------------------------------------------------------------
// OP Stack (L2) fetcher
// ---------------------------------------------------------------------------

async fn op_stack_fetcher_loop(chain: Chain, rpc_url: Url, tx: Sender<BlockPayload>) {
    use op_alloy::network::Optimism;

    // Use default() (no fillers) since we only read blocks, not send transactions.
    // ProviderBuilder::new() adds recommended fillers that are incompatible with
    // the OP Stack transaction request type.
    use alloy::providers::Identity;
    let builder: ProviderBuilder<Identity, Identity> = ProviderBuilder::default();
    let provider = builder.network::<Optimism>().connect_http(rpc_url);

    let latest = match provider.get_block_number().await {
        Ok(n) => n,
        Err(err) => {
            eprintln!("tessera [{chain}]: failed to get latest block number: {err}");
            return;
        }
    };

    let start = latest.saturating_sub(BACKFILL_COUNT - 1);
    eprintln!("tessera [{chain}]: backfilling blocks {start}..={latest}");

    for n in start..=latest {
        if op_fetch_and_send(&provider, chain, n, &tx).await.is_err() {
            return;
        }
    }

    eprintln!("tessera [{chain}]: backfill complete, polling for new blocks");

    let mut last_seen = latest;
    loop {
        tokio::time::sleep(POLL_INTERVAL).await;

        let tip = match provider.get_block_number().await {
            Ok(n) => n,
            Err(err) => {
                eprintln!("tessera [{chain}]: poll error: {err}");
                continue;
            }
        };

        for n in (last_seen + 1)..=tip {
            if op_fetch_and_send(&provider, chain, n, &tx).await.is_err() {
                return;
            }
        }
        last_seen = tip;
    }
}

async fn op_fetch_and_send(
    provider: &impl Provider<op_alloy::network::Optimism>,
    chain: Chain,
    number: u64,
    tx: &Sender<BlockPayload>,
) -> Result<(), ()> {
    use alloy::consensus::Transaction as TxConsensus;
    use alloy::network::TransactionResponse;

    let block = match provider
        .get_block_by_number(BlockNumberOrTag::Number(number))
        .full()
        .await
    {
        Ok(Some(block)) => block,
        Ok(None) => {
            eprintln!("tessera: block {number} not found");
            return Ok(());
        }
        Err(err) => {
            eprintln!("tessera: failed to fetch block {number}: {err}");
            return Ok(());
        }
    };

    let header = &block.header;
    let l1_origin = extract_l1_origin(block.transactions.as_transactions());

    let transactions: Vec<TxPayload> = match &block.transactions {
        BlockTransactions::Full(txs) => txs
            .iter()
            .enumerate()
            .map(|(i, op_tx)| {
                let blob_count = TxConsensus::blob_versioned_hashes(op_tx).map_or(0, |h| h.len());
                TxPayload {
                    hash: op_tx.tx_hash(),
                    tx_index: i,
                    gas: op_tx.gas_limit(),
                    gas_price: TxConsensus::gas_price(op_tx).unwrap_or(0),
                    value_eth: wei_to_eth(op_tx.value()),
                    from: TransactionResponse::from(op_tx),
                    to: op_tx.to(),
                    blob_count,
                    max_fee_per_blob_gas: TxConsensus::max_fee_per_blob_gas(op_tx),
                    op_stack_fees: None,
                }
            })
            .collect(),
        _ => Vec::new(),
    };

    let payload = BlockPayload {
        chain,
        number: header.number,
        gas_used: header.gas_used,
        gas_limit: header.gas_limit,
        timestamp: header.timestamp,
        tx_count: transactions.len() as u32,
        base_fee_per_gas: header.base_fee_per_gas,
        blob_gas_used: header.blob_gas_used,
        transactions,
        l1_origin_number: l1_origin,
    };

    eprintln!(
        "tessera [{chain}]: block {} ({} txs, gas {}/{}, L1 origin: {:?})",
        payload.number, payload.tx_count, payload.gas_used, payload.gas_limit, l1_origin
    );
    tx.send(payload).map_err(|_| ())
}

/// Extracts the L1 block number from the first deposit transaction's calldata.
///
/// Every OP Stack L2 block starts with an L1 Attributes deposit transaction
/// targeting the L1Block predeploy. The L1 block number is at bytes 28-35
/// in both Ecotone (packed) and pre-Ecotone (ABI-encoded) calldata formats.
fn extract_l1_origin<T: alloy::consensus::Transaction>(txs: Option<&[T]>) -> Option<u64> {
    let first = txs?.first()?;

    if first.to() != Some(L1_BLOCK_PREDEPLOY) {
        return None;
    }

    let input = first.input();
    if input.len() < 36 {
        return None;
    }

    let bytes: [u8; 8] = input[28..36].try_into().ok()?;
    Some(u64::from_be_bytes(bytes))
}

fn wei_to_eth(wei: alloy::primitives::U256) -> f64 {
    let wei_u128: u128 = wei.try_into().unwrap_or(u128::MAX);
    wei_u128 as f64 / 1e18
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::U256;

    #[test]
    fn wei_to_eth_converts_1_eth() {
        let wei = U256::from(1_000_000_000_000_000_000u128);
        let eth = wei_to_eth(wei);
        assert!((eth - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn wei_to_eth_handles_zero() {
        let eth = wei_to_eth(U256::ZERO);
        assert_eq!(eth, 0.0);
    }
}
