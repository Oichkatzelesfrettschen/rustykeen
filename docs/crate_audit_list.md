# Canonical crate list (audited)

This file is the **source of truth** for which third-party crates we intend to use for the “Keen” engine, grouped by architecture layer.

Notes:
- Some names in discussion are aliases; this doc records the *crates.io* crate names we will audit/fetch.
- This is a *target stack*; we will keep most crates **feature-gated** and adopt them incrementally.

## I. Blue Smoke Performance Core (solver & memory)
- `dlx_rs` (crates.io: `dlx-rs`)
- `bitvec` (crates.io: `bitvec`)
- `mimalloc` (crates.io: `mimalloc`)
- `bumpalo` (crates.io: `bumpalo`)
- `smallvec` (crates.io: `smallvec`)
- `wide` (crates.io: `wide`)
- `soa-derive` (crates.io: `soa_derive`)
- `likely_stable` (crates.io: `likely_stable`)
- `static_assertions` (crates.io: `static_assertions`)

## II. Hyper-Scale Logic (concurrency & math)
- `rayon` (crates.io: `rayon`)
- `parking_lot` (crates.io: `parking_lot`)
- `ringbuf` (crates.io: `ringbuf`)
- `dashmap` (crates.io: `dashmap`)
- `nohash` / `fxhash` alternatives:
  - (crates.io: `nohash-hasher`)
  - (crates.io: `fxhash`)
- `rand_pcg` (crates.io: `rand_pcg`)
- `fixed` (crates.io: `fixed`)
- `num-integer` (crates.io: `num-integer`)

## III. Zero-Overhead Architecture (I/O & bindings)
- `rkyv` (crates.io: `rkyv`)
- `crux` (crates.io: `crux_core` / `crux` ecosystem; audit required)
- `uniffi` (crates.io: `uniffi`)
- `rust-embed` (crates.io: `rust-embed`)
- `bytemuck` (crates.io: `bytemuck`)
- `anyhow` (crates.io: `anyhow`)
- `thiserror` (crates.io: `thiserror`)

## IV. Deep Vision Tooling (telemetry & verification)
- `tracing` (crates.io: `tracing`)
- `tracing-subscriber` (crates.io: `tracing-subscriber`)
- `tracing-tracy` (crates.io: `tracing-tracy`)
- `criterion` (crates.io: `criterion`)
- `ratatui` (crates.io: `ratatui`)
- Sparkline UI: use `ratatui`’s built-in `Sparkline` widget (no separate crate required).
- `varisat` (crates.io: `varisat`)
- `z3` (crates.io: `z3`)
- `kani` (crates.io: `kani`)
- `proptest` (crates.io: `proptest`)
- `bolero` (crates.io: `bolero`)
- `nom` (crates.io: `nom`)
