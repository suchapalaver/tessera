//! Tessera — block space explorer. Runs the block_explorer app.

use block_explorer::prelude::*;

fn main() {
    let _ = dotenvy::dotenv();

    let mut builder = BlockExplorerBuilder::new().chain_configs();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--fixture" => {
                let path = args.next().expect("--fixture requires a path argument");
                builder = builder.fixture(path);
            }
            "--screenshot" => {
                let path = args.next().expect("--screenshot requires a path argument");
                builder = builder.screenshot(path);
            }
            "--record" => {
                let path = args.next().expect("--record requires a path argument");
                builder = builder.record(path);
            }
            other => {
                eprintln!("tessera: unknown argument: {other}");
                std::process::exit(1);
            }
        }
    }

    builder.build().run();
}
