//! EVM block fetcher: dedicated thread + alloy → BlockPayload.

use alloy::eips::BlockNumberOrTag;
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

/// EVM-compatible block fetcher using Alloy.
pub struct EvmFetcher;

impl ChainFetcher for EvmFetcher {
    fn spawn(config: FetcherConfig) -> Receiver<BlockPayload> {
        let (tx, rx) = crossbeam_channel::bounded(64);
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
            rt.block_on(fetcher_loop(config.chain, config.rpc_url, tx));
        });
        rx
    }
}

async fn fetcher_loop(chain: Chain, rpc_url: Url, tx: Sender<BlockPayload>) {
    let provider = ProviderBuilder::new().connect_http(rpc_url);

    let latest = match provider.get_block_number().await {
        Ok(n) => n,
        Err(err) => {
            eprintln!("tessera: failed to get latest block number: {err}");
            return;
        }
    };

    let start = latest.saturating_sub(BACKFILL_COUNT - 1);
    eprintln!("tessera: backfilling blocks {start}..={latest}");

    for n in start..=latest {
        if fetch_and_send(&provider, chain, n, &tx).await.is_err() {
            return;
        }
    }

    eprintln!("tessera: backfill complete, polling for new blocks");

    let mut last_seen = latest;
    loop {
        tokio::time::sleep(POLL_INTERVAL).await;

        let tip = match provider.get_block_number().await {
            Ok(n) => n,
            Err(err) => {
                eprintln!("tessera: poll error: {err}");
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

/// Fetch a single block and send its payload on the channel.
/// Returns `Err(())` if the channel is closed (receiver dropped).
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
        "tessera: block {} ({} txs, gas {}/{})",
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
    }
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
