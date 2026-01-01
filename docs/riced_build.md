# Riced Build Configuration (2026)

## `Cargo.toml` (profiles)
```toml
[profile.release]
codegen-units = 1
opt-level = 3
panic = "abort"
strip = "symbols"
debug = false
lto = "thin"
incremental = false

[profile.bench]
inherits = "release"
debug = true

[profile.dev]
opt-level = 1
debug = 0
incremental = true
```

Notes:
- `opt-level = 3` is the equivalent of `-O3`.
- We intentionally do **not** enable `fast-math` by default; difficulty math (when added) should remain deterministic across targets.

## Warnings as errors
The workspace sets `warnings = "deny"` via Cargo lints, which is the Rust-side equivalent of `-Werror`.
CI also runs `cargo clippy ... -D warnings`.

## PGO (profile guided optimization)
Use `scripts/pgo.sh`:
- `./scripts/pgo.sh gen`
- `./scripts/pgo.sh train -- <training command>`
- `./scripts/pgo.sh use`

Optional CPU tuning (local only):
- `PGO_RUSTFLAGS_EXTRA="-C target-cpu=native" ./scripts/pgo.sh gen`

## BOLT (post-link optimization)
Use `scripts/bolt.sh`:
- `./scripts/bolt.sh record -- <training command>`
- `./scripts/bolt.sh optimize ./target/release/<binary>`
