//! Tessera — block space explorer. Runs the block_explorer app.

use block_explorer::prelude::*;

fn main() {
    let _ = dotenvy::dotenv();
    let config = chain_config();
    BlockExplorerBuilder::new().config(config).build().run();
}
