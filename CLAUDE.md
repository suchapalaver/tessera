# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Tessera is a 3D block space explorer that visualizes Ethereum blocks as slabs and transactions as cubes, built with Bevy (ECS game engine) and Alloy (Ethereum library). The implementation plan lives in `tessera.md`.

## Build Commands

```bash
# Build (library crate only — faster iteration)
cargo check --manifest-path block_explorer/Cargo.toml

# Run (requires RPC_URL in block_explorer/.env, defaults to localhost:8545)
cargo run --release

# Lint (run when finishing work)
cargo clippy --manifest-path block_explorer/Cargo.toml --all-targets --all-features -- -D warnings

# Test
cargo test --manifest-path block_explorer/Cargo.toml
cargo test --manifest-path block_explorer/Cargo.toml test_name  # single test
```

## Architecture

**Workspace layout:** The root package `tessera` (`src/main.rs`) is the binary — it loads `.env`, initializes the Bevy app, and wires together the `block_explorer` library crate. All domain logic lives in `block_explorer/`.

**Data pipeline:** A dedicated `std::thread` runs a tokio runtime that fetches blocks via Alloy, converts them to chain-agnostic `BlockPayload`/`TxPayload` structs (in `data/evm.rs`), and sends them over a bounded crossbeam channel. The Bevy ECS drains that channel each frame (max 5 blocks/frame) to spawn 3D entities. No async code runs inside Bevy systems.

**Alloy types are confined to `data/evm.rs`** — the rest of the codebase uses `BlockPayload`/`TxPayload` from `data/model.rs`.

**Visual encoding:** Slab width = `2.0 + 10.0 * gas_fullness`. Transaction cubes are colored by gas price on a blue-to-red gradient. Slabs advance along the negative Z axis, spaced 4.0 apart.

## Configuration

`RPC_URL` is read from env (loaded via dotenvy from `block_explorer/.env`). Falls back to `http://127.0.0.1:8545` (local Anvil).

## Tech Stack

- **Bevy 0.15** — ECS game engine
- **Alloy 1.0** — Ethereum RPC (`provider-http`, `rpc-types`)
- **Tokio 1** — Async runtime (fetcher thread only)
- **Crossbeam-channel 0.5** — Async-to-ECS bridge
- **Dotenvy 0.15** — `.env` loading
- **testcontainers-modules 0.14** (dev) — Anvil in Docker for integration tests
