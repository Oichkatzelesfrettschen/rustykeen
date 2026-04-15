# Getting Started with Rustykeen

## Prerequisites

- Rust nightly-2026-04-06 (pinned via rust-toolchain.toml)
- cargo toolchain
- For UI: GTK4 development libraries (libgtk-4-dev on Ubuntu)
- For WASM: wasm-pack

## Installation

### Clone Repository

```bash
git clone https://github.com/eirikr/rustykeen.git
cd rustykeen
```

### Verify Build

```bash
cargo build
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

All tests should pass with zero warnings.

## Basic Usage

### Solve a Puzzle

Solve a 2x2 puzzle using SGT (Simon Tatham's Puzzles) format:

```bash
cargo run -p kenken-cli --bin kenken-cli -- solve --n 2 --desc "b__,a3a3" --tier normal
```

Parameters:
- `--n`: Grid size for SGT `--desc` input (1-16; parser-constrained)
- `--desc`: Puzzle description in SGT format
- `--tier`: Deduction tier (none, easy, normal, hard)

Output:
```
n=2
1 2
2 1
```

### Benchmark Throughput

Run a quick throughput benchmark (supports n=2..32):

```bash
cargo run -p kenken-cli --bin kenken-cli -- benchmark --n 4 --count 10 --tier normal
```

### Count Solutions

Verify puzzle uniqueness by counting solutions:

```bash
cargo run -p kenken-cli --bin kenken-cli -- count --n 2 --desc "b__,a3a3" --limit 2
```

Result "1" means unique puzzle, "2" means ambiguous.

## Understanding SGT Format

The SGT (Simon Tatham's Puzzles) format encodes:
1. Block structure: Which cell boundaries are removed
2. Clues: Operation and target for each cage

Example: `b__,a3a3` for 2x2 grid
- `b` = block type (removed edges)
- `__,a3a3` = clues for cages

See `kenken-core/src/format/sgt_desc.rs` for full format specification.

## Building WASM

For browser deployment:

```bash
# Install wasm-pack
cargo install wasm-pack

# Build for web target
wasm-pack build kenken-wasm --target web

# Open in browser (from kenken-wasm/)
python -m http.server 8000
# Navigate to http://localhost:8000/index.html
```

## Building UI

For desktop application (requires GTK4):

```bash
# On Ubuntu/Debian
sudo apt-get install libgtk-4-dev

# Build and run
cargo run -p kenken-ui
```

Features:
- Interactive puzzle grid
- CVD-safe color themes (5 variants)
- Tutorial system (5 lessons)
- Solver visualization
- Performance metrics

## Feature Flags

Enable optional features:

```bash
# All features
cargo build --all-features

# Specific features
cargo build -p kenken-solver --features ui-instrumentation
cargo build -p kenken-solver --features sat-varisat
cargo build -p kenken-solver --features solver-dlx
```

Common features:
- `ui-instrumentation`: Solver event callbacks for visualization
- `sat-varisat`: SAT solver backend via Varisat
- `solver-dlx`: Dancing Links for Latin constraints
- `solver-u64`: 64-bit domain support (grid size > 31)
- `simd-dispatch`: Runtime SIMD dispatch
- `alloc-bumpalo`: Arena allocator for performance

## Testing

Run full test suite:

```bash
# Canonical local gate
just ci

# Full local test suite
just test

# Integration tests
cargo test -p kenken-solver --test corpus_golden
cargo test -p kenken-solver --test corpus_difficulty

# Property tests
cargo test -p kenken-core --test prop_cage_semantics

# Formal verification
cargo kani --tests

# Fuzz testing
cargo fuzz run fuzz_solver
cargo fuzz run fuzz_sgt_desc_parser
```

## Next Steps

- Read [Architecture](./03-architecture.md) for system design
- Explore [Solver Algorithm](./04-solver-algorithm.md) for deduction strategies
- Try [CLI Usage](./06-cli-usage.md) for advanced command examples
- Build [WASM Browser App](./07-wasm-browser.md) for web deployment
- Launch [Desktop UI](./08-ui-tutorial.md) with tutorial system

## Troubleshooting

### Clippy Warnings

If you see clippy warnings, ensure you have the nightly toolchain:

```bash
rustup update nightly-2026-04-06
```

### WASM Build Fails

Ensure wasm-pack is installed:

```bash
cargo install wasm-pack --version 1.3
```

### GTK4 Not Found

Install development libraries:

```bash
# Ubuntu/Debian
sudo apt-get install libgtk-4-dev libglib2.0-dev

# macOS (brew)
brew install gtk4 glib

# Alpine Linux (musl)
apk add gtk4-dev glib-dev
```

### Performance Issues

For faster builds, use:

```bash
cargo build -p kenken-cli --release
```

For profiling:

```bash
cargo flamegraph -p kenken-cli -- solve --n 6 --desc "..."
```
