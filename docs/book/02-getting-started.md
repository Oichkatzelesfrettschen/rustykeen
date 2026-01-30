# Getting Started with Rustykeen

## Prerequisites

- Rust 1.75+ (nightly-2026-01-01 pinned via rust-toolchain.toml)
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
cargo test
cargo clippy
```

All tests should pass with zero warnings.

## Basic Usage

### Solve a Puzzle

Solve a 4x4 puzzle using SGT (Simon Tatham's Puzzles) format:

```bash
cargo run -p kenken-cli -- solve --n 4 --desc "b__,a3a3" --tier normal
```

Parameters:
- `--n`: Grid size (2-32)
- `--desc`: Puzzle description in SGT format
- `--tier`: Deduction tier (none, easy, normal, hard)

Output:
```
Success: true
Solution: [1, 2, 3, 4, 2, 1, 4, 3, 3, 4, 1, 2, 4, 3, 2, 1]
Assignments: 16
Nodes Visited: 42
Max Depth: 4
Backtracked: false
```

### Generate Puzzles

Generate 10 random 4x4 puzzles:

```bash
cargo run -p kenken-cli -- generate --n 4 --count 10 --tier normal
```

### Count Solutions

Verify puzzle uniqueness by counting solutions:

```bash
cargo run -p kenken-cli -- count --n 4 --desc "..." --limit 2
```

Result "1" means unique puzzle, "2" means ambiguous.

## Understanding SGT Format

The SGT (Simon Tatham's Puzzles) format encodes:
1. Block structure: Which cell boundaries are removed
2. Clues: Operation and target for each cage

Example: `b__,a3a3` for 2x2 grid
- `b` = block type (removed edges)
- `__,a3a3` = clues for cages

See `kenken-core/src/sgt.rs` for full format specification.

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
# Unit tests
cargo test --all-targets

# Integration tests
cargo test --test corpus_golden
cargo test --test corpus_difficulty

# Property tests
cargo test --features proptest

# Formal verification
cargo kani --tests

# Fuzz testing
cargo fuzz run solver_fuzz
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
rustup update nightly-2026-01-01
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
