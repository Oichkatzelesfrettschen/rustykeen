# Developer workflow

## Toolchain
- This repo uses Rust nightly pinned by `rust-toolchain.toml`.

## Quality gates
- Format: `cargo fmt --check`
- Lint: `cargo clippy --all-targets --all-features -D warnings`
- Tests: `cargo test --all-targets`

## Upstream study artifacts
- Keep upstream code snapshots under `third_party/` (git-ignored).
- Distill upstream behavior into docs/tests, not copied code.

