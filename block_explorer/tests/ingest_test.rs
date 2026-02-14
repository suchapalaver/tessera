#[cfg(not(feature = "integration"))]
#[test]
fn integration_tests_disabled() {
    // Enable with: cargo test --features integration
    assert!(true);
}

#[cfg(feature = "integration")]
mod integration {
    use std::time::Duration;

    use alloy_chains::Chain;
    use testcontainers_modules::anvil::{AnvilNode, ANVIL_PORT};
    use testcontainers_modules::testcontainers::runners::AsyncRunner;
    use url::Url;

    use block_explorer::{ChainFetcher, EvmFetcher, FetcherConfig};

    #[tokio::test]
    async fn fetcher_ingests_genesis_block_from_anvil() {
        let node = AnvilNode::default().start().await.unwrap();
        let port = node.get_host_port_ipv4(ANVIL_PORT).await.unwrap();
        let rpc_url = Url::parse(&format!("http://localhost:{port}")).unwrap();

        let config = FetcherConfig {
            chain: Chain::mainnet(),
            rpc_url,
        };

        let rx = EvmFetcher::spawn(config);
        let payload = rx
            .recv_timeout(Duration::from_secs(10))
            .expect("expected a block payload from anvil");

        assert_eq!(payload.number, 0);
        assert_eq!(payload.tx_count, payload.transactions.len() as u32);
    }
}
