# Repository Roadmap — Tessera + SDK

This roadmap combines the original implementation plan with the new SDK‑first refactor work. Items marked **Done** are already in the codebase as of this update.

## Current Status Summary
- **SDK surface** with `BlockExplorerBuilder` and `prelude` is in place. **Done**
- **Renderer abstraction** (`BlockRenderer`) and default `SlabsAndCubesRenderer` extracted. **Done**
- **Agent feedback loop tests** (fast unit/smoke) are in place. **Done**
- **Integration tests** use Anvil and are gated behind `--features integration`. **Done**

---

## Phase A — SDK Foundation (Complete)

- [x] SDK boundary with `block_explorer::sdk::BlockExplorerBuilder`.
- [x] Minimal `block_explorer::prelude` (builder + core types).
- [x] App entrypoint uses builder (`src/main.rs`).
- [x] Remove internal ECS resources from public API surface.
- [x] Document SDK usage in `README.md`.

**Milestone:** Consumers can embed Tessera as a Bevy app via a stable builder.

---

## Phase B — Renderer Abstraction (Complete)

- [x] `BlockRenderer` trait with `spawn_block` hook.
- [x] `render` module with `SlabsAndCubesRenderer` default implementation.
- [x] Hardcoded visual formulas moved into config structs:
  - `SlabSettings`, `TxRenderSettings`, `ClusterLabelSettings`, `BlobRenderSettings`.
- [x] Ingest path delegates rendering to renderer resource.

**Milestone:** Visual encoding is pluggable without forking `scene/`.

---

## Phase C — Chain-Agnostic Data Model (Next)

**Goal:** evolve `BlockPayload`/`TxPayload` beyond EVM‑specific fields.

- [ ] Introduce `TransactionEnvelope` with chain‑agnostic fields + optional EVM addenda.
- [ ] Keep EVM mapping in `data/evm.rs` only.
- [ ] Update renderers to consume the envelope instead of EVM types.
- [ ] Add Solana placeholder types and a thin adapter skeleton.

**Milestone:** new chains can map into a neutral transaction model without touching rendering.

---

## Phase D — Extensibility & Events (Next)

**Goal:** allow SDK consumers to extend behavior without forking.

- [ ] Add `BlockSpawned`, `TxSpawned`, `TxClicked` events.
- [ ] Provide builder hooks for registering event listeners or custom systems.
- [ ] Document extension points with a simple example.

**Milestone:** third‑party extensions can attach to the render pipeline and UI interactions.

---

## Phase E — Visual Storytelling (Aligned with Original Plan)

- [ ] Value‑flow arcs (gas‑weighted, value‑colored) gated by selection or top‑N.
- [ ] Timeline scrubber with playback speed control.
- [ ] MEV & ordering visualization (cluster + ordering gradient).
- [ ] Gas heatmap mode (shader or texture).
- [ ] Contract clustering and labels.

**Milestone:** the scene explains *why* a block looks the way it does.

---

## Phase F — Multi‑Chain & Performance

- [ ] Chain‑aware fee decomposition for OP Stack (L1 data fee, blob fee).
- [ ] Solana fetcher and adapter.
- [ ] Multi‑lane layout (parallel lanes or layered planes).
- [ ] Instanced rendering for tx cubes.
- [ ] Optional persistence (`redb`) for fast startup and replay.

**Milestone:** multi‑chain, performant, and replayable explorer.

---

## Phase G — Polish & Ship

- [ ] Bloom, fog, SSAO, smooth camera transitions.
- [ ] Accessibility (color‑blind palettes, keyboard‑only nav).
- [ ] Cross‑platform packaging and a small web demo (optional WASM).
- [ ] README screenshots and setup walkthrough.

**Milestone:** distributable, polished visual SDK.

---

## Testing & Agent Feedback Loop (Priority)

- **Fast loop:** `cargo test` (unit + smoke; target < 30s).
- **Integration:** `cargo test --features integration` (Anvil, Docker required).
- **Guideline:** integration tests must not depend on external RPC or secrets.

---

## Quick Reference — Key Decisions

- **Async bridge:** `crossbeam_channel` for ECS safety.
- **RPC:** polling instead of websockets for portability.
- **EVM client:** `alloy` confined to `data/evm.rs`.
- **Rendering:** now pluggable via `BlockRenderer`.
- **SDK surface:** minimal public API, internal ECS resources hidden.
