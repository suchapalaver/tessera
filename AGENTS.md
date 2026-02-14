# Repository Guidelines

## Project Structure & Module Organization
- `src/main.rs` is the binary entry point that wires Bevy plugins from the `block_explorer` crate.
- `block_explorer/src/` is the core library, grouped by feature areas: `camera/`, `data/`, `scene/`, and `ui/`, with shared modules like `config.rs` and `lib.rs`.
- Build artifacts live under `target/` and `block_explorer/target/` (do not edit).
- Runtime config lives in `block_explorer/.env` (see `RPC_URL` below).

## Build, Test, and Development Commands
- `cargo run --release` runs the optimized build (recommended for smooth rendering).
- `cargo run` runs a debug build for faster iteration.
- `cargo build` builds the workspace without running.
- `cargo test` runs unit, smoke, and integration tests (if present).
- `cargo test -- --nocapture` shows test logging, useful for debugging.

## Coding Style & Naming Conventions
- Use standard Rust formatting (`rustfmt` defaults). Indentation is 4 spaces.
- Naming follows Rust conventions: `snake_case` for functions/variables/modules, `CamelCase` for types/traits, `SCREAMING_SNAKE_CASE` for constants.
- Keep Bevy systems small and composable; register new systems alongside related plugins in `src/main.rs` or `block_explorer/src/lib.rs`.

## Testing Guidelines (Agent Feedback Loop Priority)
- Goal: tests must provide a fast feedback loop for agents. Target under ~30s on a typical dev machine.
- Current state: no in-repo tests exist yet.
- Unit tests: add `#[cfg(test)] mod tests` in the owning module. Start with config parsing and deterministic helpers in `block_explorer/src/config.rs` and `block_explorer/src/data/`.
- Smoke test: add a minimal startup test that constructs the Bevy app without panicking, ideally in headless/minimal mode.
- Integration tests: place under `block_explorer/tests/` to exercise RPC ingestion and cross-system behavior. Use `testcontainers-modules` with Anvil for repeatable chain state.
- Naming: `*_test.rs` for integration tests, `test_*` for unit tests.
- Requirements: integration tests must be self-contained and must not depend on external RPC providers or secrets.

## Commit & Pull Request Guidelines
- Commit messages follow Conventional Commits (`feat: ...`, `fix: ...`, `docs: ...`, `refactor: ...`).
- PRs should include a clear summary, any user-visible UI/controls changes, and links to issues if applicable.
- If changes affect rendering or interaction, include a short demo note (e.g., “ran on local Anvil”).

## Configuration & Security Notes
- Required config: `RPC_URL` in `block_explorer/.env`. Example:
  ```bash
  echo 'RPC_URL=http://127.0.0.1:8545' > block_explorer/.env
  ```
- Do not commit API keys or private RPC URLs.
