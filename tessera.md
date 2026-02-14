# Block Space Explorer — Implementation Plan

## Project Structure

**Tessera** is a Cargo workspace. The root package `tessera` is the binary; `block_explorer` is the library.

```
tessera/                           # workspace root
├── Cargo.toml                     # [workspace] members = ["block_explorer"], package name = "tessera"
├── src/
│   └── main.rs                   # Binary: loads .env, runs block_explorer app
├── tessera.md
├── .env                           # RPC_URL, optional keys (at root or in block_explorer)
└── block_explorer/
    ├── Cargo.toml
    ├── src/
    │   ├── lib.rs                # Library root; no main.rs
    │   ├── data/
    │   │   ├── mod.rs
    │   │   ├── model.rs          # BlockPayload, TxPayload, chain-agnostic types
    │   │   ├── evm.rs            # alloy fetcher → BlockPayload
    │   │   ├── solana.rs         # (future) solana-client → BlockPayload
    │   │   └── channel.rs       # BlockChannel resource, spawn helpers
    │   ├── scene/
    │   │   ├── mod.rs
    │   │   ├── blocks.rs        # ingest_blocks system, slab spawning
    │   │   ├── transactions.rs  # tx cube spawning, grid layout, coloring
    │   │   ├── arcs.rs          # (Phase 3) value-flow arc rendering
    │   │   └── materials.rs     # shared material/color utilities
    │   ├── camera/
    │   │   ├── mod.rs
    │   │   └── fly.rs           # FlyCamera component + system
    │   ├── ui/
    │   │   ├── mod.rs
    │   │   ├── hud.rs           # egui overlay: block stats, gas gauge
    │   │   ├── inspector.rs     # click-to-select entity detail panel
    │   │   └── timeline.rs     # (Phase 3) scrubber / playback controls
    │   └── config.rs           # CLI args, env parsing, constants
    └── assets/
        └── shaders/            # (Phase 4) custom wgsl shaders
```

From the repo root: `cargo run` runs the **tessera** binary; `cargo run -p block_explorer` is not used (no binary in block_explorer). `cargo test -p block_explorer` runs the library tests.

---

## Phase 1 — Foundation (get it compiling and rendering)

### 1.1 Scaffold the workspace

- [ ] `cargo init block_explorer`
- [ ] Set up `Cargo.toml` with pinned deps:
  - `bevy = "0.15"`, `alloy = "1.0"` (features: `provider-http`, `rpc-types`)
  - `tokio = "1"` (features: `rt`, `time`), `crossbeam-channel = "0.5"`
  - `dotenvy = "0.15"` for `.env` loading
- [ ] Create directory structure above (empty `mod.rs` files with public re-exports)
- [ ] Verify `cargo check` passes with stub modules

### 1.2 Data model (`src/data/model.rs`)

- [ ] Define `BlockPayload { number, gas_used, gas_limit, timestamp, tx_count, transactions }`
- [ ] Define `TxPayload { hash: Option<String>, gas, gas_price, value_eth, from: Option<String>, to: Option<String> }`
- [ ] Both types: `Clone`, `Debug`, `Send`, `Sync`
- [ ] Keep these types alloy-free — conversion happens in `evm.rs`

### 1.3 EVM fetcher (`src/data/evm.rs`)

- [ ] `pub fn spawn_evm_fetcher(rpc_url: String) -> Receiver<BlockPayload>`
- [ ] Internal tokio runtime on a dedicated `std::thread`
- [ ] Backfill last 20 blocks, then poll every 2s
- [ ] `fetch_block()` converts alloy types → `BlockPayload`
- [ ] Graceful error handling: log and continue, never panic the thread
- [ ] **Test checkpoint:** run fetcher standalone, print payloads to stdout

### 1.4 Channel bridge (`src/data/channel.rs`)

- [ ] `BlockChannel(Receiver<BlockPayload>)` as a Bevy `Resource`
- [ ] Helper: `pub fn init_block_channel(rpc: &str) -> BlockChannel`

### 1.5 Scene — block slabs (`src/scene/blocks.rs`)

- [ ] `ExplorerState` resource: `blocks_rendered: u64`, `z_cursor: f32`
- [ ] `ingest_blocks` system: drain channel (max 5/frame), spawn slab entities
- [ ] Slab width = `2.0 + 10.0 * fullness` — directly encodes gas usage
- [ ] Slab color darkens/lightens with fullness
- [ ] `BlockSlab { number }` component on each entity

### 1.6 Scene — transaction cubes (`src/scene/transactions.rs`)

- [ ] `grid_positions(count, width, depth)` → `Vec<(f32, f32)>` layout helper
- [ ] `gas_price_color(gwei: f64) -> Color` gradient: blue → cyan → yellow → red
- [ ] Cube height ∝ normalized gas, side ∝ normalized gas
- [ ] Emissive glow for value > 1 ETH
- [ ] `TxCube` marker component

### 1.7 Camera (`src/camera/fly.rs`)

- [ ] `FlyCamera { speed, sensitivity, pitch, yaw }` component
- [ ] Right-click to grab cursor + mouse-look
- [ ] WASD movement, Q/E for vertical, Shift for sprint
- [ ] Clamp pitch to avoid gimbal flip

### 1.8 Main assembly (`src/main.rs`)

- [ ] Load `.env`, read `RPC_URL`
- [ ] Init channel, insert resources
- [ ] Register systems: `setup_scene` (Startup), `ingest_blocks` + `fly_camera` (Update)
- [ ] Window config: title, resolution, dark clear color
- [ ] Directional light + ambient light
- [ ] **Milestone: `cargo run --release` shows live blocks you can fly through**

---

## Phase 2 — UI Overlay & Interaction

**New dep:** `bevy_egui = "0.34"` (match your Bevy version)

### 2.1 HUD (`src/ui/hud.rs`)

- [ ] Top-left panel: latest block number, gas used/limit, tx count
- [ ] Gas fullness gauge (horizontal bar, color-coded)
- [ ] Running average gas price (last 10 blocks)
- [ ] Blocks rendered counter, FPS display
- [ ] Style: semi-transparent dark background, monospace font

### 2.2 Entity inspector (`src/ui/inspector.rs`)

- [ ] Raycast on left-click (use `bevy_mod_raycast` or Bevy's built-in picking)
- [ ] If hit `BlockSlab`: show block number, gas stats, timestamp, tx count
- [ ] If hit `TxCube`: show gas, gas price, value, from/to addresses
- [ ] Right panel: detail card with data, dismiss on Escape or click-away
- [ ] Highlight selected entity (swap to emissive material or outline)

### 2.3 Block labels

- [ ] Floating text above each slab: block number
- [ ] Options: `bevy_mod_billboard` for 3D text, or screen-space egui labels
- [ ] LOD: only render labels for blocks within camera frustum + distance threshold

### 2.4 Config & CLI (`src/config.rs`)

- [ ] `clap` or simple env-based config: `RPC_URL`, `BACKFILL_COUNT`, `POLL_INTERVAL_MS`
- [ ] Optional: chain selector flag (`--chain ethereum|base|arbitrum`)
- [ ] **Milestone: interactive explorer with readable data on hover/click**

---

## Phase 3 — Richer Visual Storytelling

### 3.1 Value-flow arcs (`src/scene/arcs.rs`)

- [ ] For each tx with `from` and `to`, draw a cubic bezier arc above the slab
- [ ] Arc color = value magnitude, thickness = gas used
- [ ] Requires: expand `TxPayload` to carry `from`/`to` (add in 1.2, populate in 1.3)
- [ ] Optimization: only render arcs for selected block or top-N by value
- [ ] Use Bevy's `Gizmos` for prototyping, then custom mesh for production

### 3.2 Timeline scrubber (`src/ui/timeline.rs`)

- [ ] Horizontal bar at screen bottom: time axis
- [ ] Click to jump camera to that block's Z position
- [ ] Playback mode: animate camera flying through blocks at configurable speed
- [ ] Pause/resume, speed controls

### 3.3 MEV & ordering visualization

- [ ] Color-code tx cubes by position in block (first = bright, last = dim)
- [ ] Sandwich detection heuristic: highlight tx triplets with same pair address
- [ ] Optional: fetch builder/relay info via Flashbots API, tag builder on slab

### 3.4 Gas heatmap mode

- [ ] Toggle view: replace individual cubes with a continuous heatmap texture on the slab
- [ ] X-axis = tx index, color = gas price → shows ordering/priority patterns
- [ ] Custom wgsl fragment shader in `assets/shaders/`

### 3.5 Contract clustering

- [ ] Group txs by `to` address (contract calls)
- [ ] Cluster cubes spatially: same contract → same region of the slab
- [ ] Label top contracts (Uniswap, USDT, etc.) via known address registry
- [ ] **Milestone: the scene tells stories about MEV, contention, and flow**

---

## Phase 4 — Multi-Chain & Performance

### 4.1 Normalized data trait

- [ ] Define trait in `model.rs`:

  ```rust
  pub trait ChainFetcher: Send + 'static {
      fn spawn(config: ChainConfig) -> Receiver<BlockPayload>;
  }
  ```

- [ ] Refactor `evm.rs` to implement this trait
- [ ] `BlockPayload` gains `chain: ChainId` field (enum: Ethereum, Base, Arbitrum, Solana...)

### 4.2 OP Stack L2 gas fee decomposition

When fetching from OP Stack chains (Base, Optimism, etc.), use `op-alloy-network` with `Provider<Optimism>` instead of `Provider<Ethereum>` to access L2-specific receipt fields. This exposes three distinct gas cost components:

- **L2 execution cost** — gas used on L2 × L2 gas price
- **L1 data fee** — fee for posting tx data to L1
- **Blob gas cost** — EIP-4844 blob storage cost

Reference: `../../semiotic-ai/likwid/` L2 adapter pattern — branches on `chain.has_l1_fees()` to select the provider network type, then extracts fee components from OP Stack receipts.

- [ ] Add `op-alloy-network` dependency
- [ ] Branch `EvmFetcher` provider construction based on chain type
- [ ] Extend `TxPayload` with optional L1 data fee and blob gas fields
- [ ] Surface L2 fee breakdown in inspector UI

### 4.3 Solana fetcher (`src/data/solana.rs`)

- [ ] Use `solana-client` + `solana-transaction-status` crates
- [ ] Map slot → `BlockPayload`, instructions → `TxPayload`
- [ ] Handle Solana's faster cadence (400ms slots): aggregate into visual "super-slots"
  or use a time-normalized Z-axis instead of per-block spacing

### 4.4 Multi-chain scene layout

- [ ] Option A: parallel lanes (Ethereum = left lane, Solana = right lane, shared time axis)
- [ ] Option B: layered planes (each chain is a horizontal sheet at different Y heights)
- [ ] Color-code slabs by chain, unify the time axis to wall-clock
- [ ] Cross-chain bridge txs: detect (heuristic or tagged) and draw arcs between lanes

### 4.5 Instanced rendering

- [ ] Replace individual `TxCube` mesh spawns with `InstancedMesh` batches
- [ ] One mesh + one material per gas-price bucket, thousands of instances
- [ ] Target: 100+ blocks on screen at 60fps
- [ ] Bevy's `Mesh` with custom vertex data, or use `bevy_hanabi` for particle-like tx clouds

### 4.6 Data persistence

- [ ] Optional: write `BlockPayload` history to `redb` (embedded, zero-config)
- [ ] Enables: launch app, immediately see last 1000 blocks without re-fetching
- [ ] Replay mode: scrub through stored history offline
- [ ] **Milestone: multi-chain spatial explorer with smooth performance at scale**

---

## Phase 5 — Polish & Ship

### 5.1 Visual polish

- [ ] Bloom post-processing (Bevy built-in) for emissive tx glow
- [ ] Fog for depth cue on distant blocks
- [ ] Ambient occlusion (SSAO) for grounding cubes on slabs
- [ ] Smooth camera transitions (lerp to clicked block)

### 5.2 Accessibility & UX

- [ ] Keyboard-only navigation (tab through blocks, enter to inspect)
- [ ] Color-blind safe palette option (swap gas gradient to viridis or cividis)
- [ ] Configurable text size in HUD

### 5.3 Distribution

- [ ] `cargo build --release` produces single binary
- [ ] Test on Linux, macOS, Windows (Bevy supports all three)
- [ ] Optional: WASM build (`--target wasm32-unknown-unknown`) for web demo
  - Note: alloy's HTTP provider works in WASM, but WebSocket may need `wasm-bindgen`
  - Bevy WASM requires WebGPU-capable browser or WebGL2 fallback
- [ ] README with screenshots, controls, RPC setup instructions

---

## Dependency Graph (what blocks what)

```
1.1 ──▶ 1.2 ──▶ 1.3 ──▶ 1.4 ──▶ 1.8 (runnable!)
                  │                 ▲
                  ▼                 │
                 1.5 ──▶ 1.6 ──────┘
                                    │
                 1.7 ───────────────┘
                                    │
         2.1 ───────────────────────┘ (needs running app)
         2.2 ──▶ 2.3
         2.4 (independent)
                                    
         3.1 requires 2.2 (selection) + expanded TxPayload
         3.2 requires 2.1 (HUD exists)
         3.3 requires 3.1
         3.4 independent (shader work)
         3.5 requires 1.6 (tx cubes exist)
         
         4.1 ──▶ 4.2 (OP Stack gas fees)
         4.1 ──▶ 4.3 ──▶ 4.4
         4.5 independent (perf work, do when needed)
         4.6 independent
```

---

## Testing & examples — Anvil + testcontainers

Use **Anvil** in Docker via **testcontainers-rs** for deterministic integration tests and for running examples without a live RPC. Reference implementation: **`../../semiotic-ai/likwid/`** (likwid).

### Pattern (from likwid)

- **Crate:** `testcontainers-modules = "0.14"` with feature `anvil` (pulls in `testcontainers`; use async runner).
- **Image:** `testcontainers_modules::anvil::AnvilNode::latest()` — community module wraps `ghcr.io/foundry-rs/foundry`, runs `anvil --host 0.0.0.0`.
- **Port:** RPC on **8545**. Get host-mapped port: `node.get_host_port_ipv4(8545).await?`.
- **RPC URL:** `format!("http://localhost:{}", port)` (or use `node.get_host().await?` for host when not localhost).
- **Lifecycle:** `AnvilNode::latest().start().await` returns `ContainerAsync<AnvilNode>`; container is removed when the value is dropped (RAII).
- **Optional (fork mode):** `.with_fork_url(url)`, `.with_chain_id(id)` for mainnet/testnet forks; block_explorer can start with a plain local Anvil (no fork) for fetcher tests.

### block_explorer usage

| Use case | Approach |
|----------|----------|
| **Fetcher integration test** | Start Anvil, `init_block_channel(rpc_url)` (or `spawn_evm_fetcher(rpc_url)`), drain channel for N blocks / payloads, assert block numbers and tx counts. Keeps container for test duration. |
| **Example: “explorer against local chain”** | Same: start Anvil, pass RPC URL into app (e.g. env or CLI), run Bevy app; optionally seed txs with `anvil_*` (alloy’s AnvilApi) for richer visuals. |
| **CI** | Require Docker; run `cargo test` (integration tests start/stop Anvil automatically). |

### Dependencies (block_explorer)

```toml
[dev-dependencies]
testcontainers-modules = { version = "0.14", features = ["anvil"] }
```

Use `testcontainers_modules::testcontainers::runners::AsyncRunner` (trait) so that `AnvilNode::latest().start().await` works. No need for alloy’s `anvil-node` feature unless tests use AnvilApi (impersonation, etc.).

### Checklist

- [ ] Add `testcontainers-modules` (anvil) to block_explorer `[dev-dependencies]`
- [ ] Integration test: start Anvil, get RPC URL, verify connectivity (e.g. `get_block_number` or drain fetcher)
- [ ] Optional: example that runs the explorer against Anvil (with or without seeded txs)

---

## Quick Reference — Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Async bridge | crossbeam channel | No async in ECS, bounded backpressure |
| RPC approach | Polling, not WS subscriptions | Simpler, more portable, debuggable |
| EVM library | alloy | Rust-native, typed, actively maintained |
| Chain-agnostic model | Flat structs, not trait objects | Avoid dyn dispatch in hot path |
| Slab sizing | Width ∝ gas fullness | Instantly readable at a glance |
| Tx color | Gas price gradient | Answers "how expensive was this block?" |
| Multi-chain layout | Parallel lanes, shared time axis | Preserves temporal correlation |
| Persistence | redb (embedded) | Zero config, Rust-native, no server |
