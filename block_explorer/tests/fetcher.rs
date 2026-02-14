use std::time::Duration;

use alloy::providers::{Provider, ProviderBuilder};
use alloy_chains::Chain;
use block_explorer::{ChainFetcher, EvmFetcher, FetcherConfig};
use testcontainers_modules::{anvil::AnvilNode, testcontainers::runners::AsyncRunner};
use url::Url;

const ANVIL_PORT: u16 = 8545;
const RECV_TIMEOUT: Duration = Duration::from_secs(10);

async fn anvil_rpc_url(
    node: &testcontainers_modules::testcontainers::ContainerAsync<AnvilNode>,
) -> Url {
    let port = node.get_host_port_ipv4(ANVIL_PORT).await.unwrap();
    format!("http://localhost:{port}").parse().unwrap()
}

#[tokio::test]
async fn fetcher_receives_backfilled_blocks() {
    let node = AnvilNode::latest().start().await.unwrap();
    let rpc_url = anvil_rpc_url(&node).await;

    let config = FetcherConfig {
        chain: Chain::mainnet(),
        rpc_url,
    };
    let rx = EvmFetcher::spawn(config);

    let payload = rx
        .recv_timeout(RECV_TIMEOUT)
        .expect("should receive genesis block");

    assert_eq!(payload.number, 0, "first block should be genesis");
    assert_eq!(payload.chain, Chain::mainnet());
    assert_eq!(payload.tx_count, 0, "genesis block has no transactions");
}

#[tokio::test]
async fn fetcher_includes_transactions() {
    let node = AnvilNode::latest().start().await.unwrap();
    let rpc_url = anvil_rpc_url(&node).await;

    let provider = ProviderBuilder::new().connect_http(rpc_url.clone());

    // Anvil pre-funds 10 accounts; send value transfers between them.
    let accounts = provider.get_accounts().await.unwrap();
    assert!(accounts.len() >= 2, "Anvil should have pre-funded accounts");

    let from = accounts[0];
    let to = accounts[1];

    for _ in 0..2 {
        let tx = alloy::rpc::types::TransactionRequest::default()
            .from(from)
            .to(to)
            .value(alloy::primitives::U256::from(1_000_000_000_000_000_000u128)); // 1 ETH
        provider
            .send_transaction(tx)
            .await
            .expect("send_transaction should succeed")
            .watch()
            .await
            .expect("transaction should be mined");
    }

    let config = FetcherConfig {
        chain: Chain::mainnet(),
        rpc_url,
    };
    let rx = EvmFetcher::spawn(config);

    // Drain until we find a block with transactions.
    let mut found = None;
    for _ in 0..30 {
        match rx.recv_timeout(RECV_TIMEOUT) {
            Ok(block) if block.tx_count > 0 => {
                found = Some(block);
                break;
            }
            Ok(_) => continue,
            Err(_) => break,
        }
    }

    let block = found.expect("should receive a block with transactions");
    assert!(block.tx_count >= 1);

    let tx = &block.transactions[0];
    assert!(tx.gas > 0, "transaction gas should be positive");
    assert_eq!(tx.from, from);
    assert!(
        tx.value_eth > 0.0,
        "value transfer should have positive ETH value"
    );
}
