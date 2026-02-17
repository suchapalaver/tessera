//! Tessera — block space explorer. Runs the block_explorer app.

use block_explorer::prelude::*;

fn main() {
    let _ = dotenvy::dotenv();
    BlockExplorerBuilder::new().chain_configs().build().run();
}
