# Tessera

A 3D block space explorer that visualizes Ethereum blocks as slabs and transactions as cubes.

## Features

- **Live block streaming** — connects to any EVM-compatible RPC endpoint, backfills recent blocks, then follows the chain tip
- **3D visualization** — each block is a slab whose width encodes gas fullness; transactions sit on top as cubes colored by gas price (blue = cheap, red = expensive)
- **High-value glow** — transactions transferring more than 1 ETH emit a glow
- **HUD overlay** — live block number, gas usage bar, average gas price, transaction count, and FPS
- **Block inspector** — click any block slab to open a detail panel with gas stats, transaction count, and timestamp
- **Fly camera** — navigate the scene freely with keyboard and trackpad

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- An EVM RPC URL (e.g. from [Alchemy](https://www.alchemy.com/), [Infura](https://www.infura.io/), or a local [Anvil](https://book.getfoundry.sh/anvil/) node)

### Setup

1. Clone the repository:

   ```bash
   git clone https://github.com/suchapalaver/tessera.git
   cd tessera
   ```

2. Create a `.env` file inside `block_explorer/` with your RPC URL:

   ```bash
   echo 'RPC_URL=https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY' > block_explorer/.env
   ```

3. Run in release mode:

   ```bash
   cargo run --release
   ```

If no `RPC_URL` is set, it defaults to `http://127.0.0.1:8545` (local Anvil).

### Library Usage

Use the SDK builder when embedding Tessera in another Bevy app:

```rust
use block_explorer::prelude::*;

let _ = dotenvy::dotenv();
BlockExplorerBuilder::new().chain_config().build().run();
```

## Controls

| Key | Action |
|-----|--------|
| W / A / S / D | Move forward / left / backward / right |
| Q / E | Move up / down |
| Arrow keys | Look around |
| Trackpad scroll | Look around |
| Shift (hold) | Sprint (3x speed) |
| Space / Home | Reset camera to start position |
| Click (on slab) | Inspect block details |
| Escape | Dismiss inspector panel |

## Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `RPC_URL` | EVM JSON-RPC endpoint (defaults to mainnet) | `http://127.0.0.1:8545` |
| `MAINNET_RPC_URL` | Ethereum mainnet endpoint | — |
| `BASE_RPC_URL` | Base L2 endpoint | — |
| `OPTIMISM_RPC_URL` | Optimism endpoint | — |
| `ARBITRUM_RPC_URL` | Arbitrum endpoint | — |

Set via environment variable or in `block_explorer/.env`. Chain-specific vars override `RPC_URL` and auto-select the chain.

**Provider examples:**

```bash
# Alchemy (mainnet)
RPC_URL=https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY

# Base (auto-detected)
BASE_RPC_URL=https://base-mainnet.g.alchemy.com/v2/YOUR_KEY

# Local Anvil
RPC_URL=http://127.0.0.1:8545
```

## Tech Stack

- [Bevy](https://bevyengine.org/) 0.15 — ECS game engine and renderer
- [Alloy](https://github.com/alloy-rs/alloy) 1.0 — Ethereum RPC client
- [bevy_egui](https://github.com/vladbat00/bevy_egui) — immediate-mode UI for HUD and inspector
- [Tokio](https://tokio.rs/) — async runtime (fetcher thread only)
- [crossbeam-channel](https://github.com/crossbeam-rs/crossbeam) — async-to-ECS bridge
