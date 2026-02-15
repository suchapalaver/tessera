# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Tessera is a 3D block space explorer that visualizes Ethereum blocks as slabs and transactions as cubes, built with Bevy (ECS game engine) and Alloy (Ethereum library). The implementation plan lives in `tessera.md`.

## Build Commands

```bash
# Build (library crate only — faster iteration)
cargo check --manifest-path block_explorer/Cargo.toml

# Run (requires RPC URL in block_explorer/.env, defaults to localhost:8545)
cargo run --release

# Lint (run when finishing work)
cargo clippy --manifest-path block_explorer/Cargo.toml --all-targets --all-features -- -D warnings

# Test (unit tests)
cargo test --manifest-path block_explorer/Cargo.toml
cargo test --manifest-path block_explorer/Cargo.toml test_name  # single test

# Integration tests (requires Docker for Anvil container)
cargo test --manifest-path block_explorer/Cargo.toml --features integration
```

## Architecture

**Workspace layout:** The root package `tessera` (`src/main.rs`) is the binary — it loads `.env`, initializes the Bevy app via `BlockExplorerBuilder`, and wires together the `block_explorer` library crate. All domain logic lives in `block_explorer/`.

**Data pipeline:** A dedicated `std::thread` runs a tokio runtime that fetches blocks via Alloy, converts them to chain-agnostic `BlockPayload`/`TxPayload` structs (in `data/evm.rs`), and sends them over a bounded crossbeam channel. The Bevy ECS drains that channel each frame (max 5 blocks/frame) to spawn 3D entities. No async code runs inside Bevy systems.

**Alloy types are confined to `data/evm.rs`** — the rest of the codebase uses `BlockPayload`/`TxPayload` from `data/model.rs`.

**SDK builder (`sdk.rs`):** `BlockExplorerBuilder` is the main entry point. It supports a pluggable `BlockRenderer` trait (defined in `render/mod.rs`) for custom visualization — the default is `SlabsAndCubesRenderer`. Individual features (fly camera, HUD, inspector, timeline, arcs, heatmap) can be toggled off.

**Key modules in `block_explorer/src/`:**
- `data/` — fetcher trait (`ChainFetcher`), EVM implementation, model types, crossbeam channel bridge
- `scene/` — ECS systems: block ingestion (`ingest_blocks`), scene setup, heatmap toggle, arc rendering, labels
- `render/` — `BlockRenderer` trait and `SlabsAndCubesRenderer` default implementation
- `ui/` — HUD overlay, block inspector panel, timeline (all via `bevy_egui`)
- `camera/` — fly camera controls

**Visual encoding:** Slab width = `2.0 + 10.0 * gas_fullness`. Transaction cubes are colored by gas price on a blue-to-red gradient. Slabs advance along the negative Z axis, spaced 4.0 apart.

## Configuration

Env vars are loaded via dotenvy from `block_explorer/.env`. Chain-specific vars take priority over `RPC_URL`:

| Variable | Chain |
|----------|-------|
| `MAINNET_RPC_URL` | Ethereum mainnet |
| `BASE_RPC_URL` | Base |
| `OPTIMISM_RPC_URL` | Optimism |
| `ARBITRUM_RPC_URL` | Arbitrum |
| `RPC_URL` | Mainnet (fallback) |

Falls back to `http://127.0.0.1:8545` (local Anvil) if nothing is set.

## Tech Stack

- **Bevy 0.15** — ECS game engine
- **Alloy 1.0** — Ethereum RPC (`provider-http`, `rpc-types`, `consensus`, `network`)
- **bevy_egui 0.33** — immediate-mode UI for HUD and inspector
- **Tokio 1** — Async runtime (fetcher thread only)
- **Crossbeam-channel 0.5** — Async-to-ECS bridge
- **Dotenvy 0.15** — `.env` loading
- **testcontainers-modules 0.14** (dev) — Anvil in Docker for integration tests
