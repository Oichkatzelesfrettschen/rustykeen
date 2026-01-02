# rustykeen

A cleanroom Rust implementation of a **KenKen-style puzzle solver and generator** (based on Simon Tatham's "Keen" puzzle). Library-first design with deterministic, reproducible behavior suitable for embedding in Android/iOS/desktop/web applications.

## Features

- **Pure Rust** core puzzle model, solver, and generator
- **Deterministic** generation and solving (seeded RNG, stable ordering)
- **Performance-oriented** architecture (cache-local data, optional SIMD, optional parallelism)
- **FFI bindings** for Kotlin/Swift via UniFFI
- **Multiple solver backends**: backtracking with MRV/LCV, optional DLX, optional SAT (Varisat)

## Quick Start

```bash
# Build
cargo build --release

# Solve a puzzle (SGT format)
cargo run -p kenken-cli --release -- solve --n 4 --desc b__,a3a3 --tier normal

# Count solutions
cargo run -p kenken-cli --release -- count --n 4 --desc b__,a3a3 --limit 2

# Run tests
cargo test --all-targets
```

## Workspace Structure

| Crate | Purpose |
|-------|---------|
| `kenken-core` | Puzzle model, validation, cage semantics, format parsing |
| `kenken-solver` | Backtracking solver with MRV/LCV, optional DLX/SAT backends |
| `kenken-gen` | Puzzle generator with cage partitioning, uniqueness verification |
| `kenken-simd` | Runtime ISA dispatch (popcount, etc.) - controlled unsafe |
| `kenken-io` | Versioned serialization (rkyv snapshots) |
| `kenken-uniffi` | UniFFI bindings for Kotlin/Swift |
| `kenken-verify` | Formal verification helpers |
| `kenken-cli` | Reference CLI tool |

## Design Principles

- **Cleanroom**: No upstream code copied; behavior derived from specification
- **Determinism**: `ChaCha20Rng` for reproducible RNG across all platforms
- **Safety**: `unsafe_code = "forbid"` everywhere except `kenken-simd`
- **Quality**: `warnings = "deny"` workspace-wide, enforced in CI

## Feature Flags

**kenken-solver:**
- `solver-dlx` - DLX Latin square solver
- `sat-varisat` - SAT solver backend
- `simd-dispatch` - Runtime SIMD dispatch
- `tracing` - Instrumentation spans

**kenken-gen:**
- `gen-dlx` - DLX-based Latin solution generation
- `parallel-rayon` - Parallel batch solving

## Documentation

- [`docs/plan.md`](docs/plan.md) - Primary implementation plan
- [`docs/architecture.md`](docs/architecture.md) - Workspace layout and data flow
- [`docs/cleanroom_plan.md`](docs/cleanroom_plan.md) - Cleanroom porting approach
- [`CLAUDE.md`](CLAUDE.md) - Claude Code guidance

## Requirements

- Rust nightly (see `rust-toolchain.toml` for pinned version)
- Optional: Z3 for formal verification features

## License

GPL-2.0-only
