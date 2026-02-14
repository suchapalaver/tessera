# Repository Guidelines

## Project Structure & Module Organization
- `src/main.rs` is the binary entry point that wires Bevy plugins from the `block_explorer` crate.
- `block_explorer/src/` contains the core library code, grouped by feature area: `camera/`, `data/`, `scene/`, and `ui/`, plus shared modules like `config.rs` and `lib.rs`.
- `block_explorer/target/` and `target/` are build artifacts and should not be edited directly.
- Configuration lives in `block_explorer/.env` (see `RPC_URL` below).

## Build, Test, and Development Commands
- `cargo run --release` builds and runs the app with optimizations (recommended for smooth rendering).
- `cargo run` builds and runs a debug build for faster iteration.
- `cargo build` builds the workspace without running.
- `cargo test` runs tests (currently there are no in-repo tests; this will still compile the workspace).

## Coding Style & Naming Conventions
- Use standard Rust formatting (`rustfmt` defaults). Indentation is 4 spaces.
- Naming follows Rust conventions: `snake_case` for functions/variables/modules, `CamelCase` for types/traits, `SCREAMING_SNAKE_CASE` for constants.
- Keep Bevy systems small and composable; register new systems in `src/main.rs` or `block_explorer/src/lib.rs` alongside related plugins.

## Testing Guidelines
- No dedicated tests exist in the repository yet. If adding tests, place unit tests in module files with `#[cfg(test)] mod tests` and use `cargo test` to run them.
- For integration-style checks against an EVM node, prefer containerized fixtures (the dev dependency `testcontainers-modules` is already included).

## Commit & Pull Request Guidelines
- Commit messages follow Conventional Commits (`feat: ...`, `fix: ...`, `docs: ...`, `refactor: ...`). Keep the scope focused and imperative.
- PRs should include a clear summary, note any user-visible changes to the UI/controls, and link relevant issues if applicable.
- If changes affect rendering or interaction, include a short demo note (e.g., “ran on local Anvil” or “tested against mainnet RPC”).

## Configuration & Security Notes
- Required config: `RPC_URL` in `block_explorer/.env`. Example:
  ```bash
  echo 'RPC_URL=http://127.0.0.1:8545' > block_explorer/.env
  ```
- Do not commit API keys or RPC URLs with private credentials.
