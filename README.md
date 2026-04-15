# Rustykeen: Production-Grade KenKen Solver

```
+-------+-------+-------+-------+
| Rusty | keen  | Rust  | Keen  |
+-------+-------+-------+-------+
|1| 2 | 3| 4  | 5 |6  | 7| 8  |
+-------+-------+-------+-------+
| 9|10 |11|12  |13|14  |15|16  |
+-------+-------+-------+-------+

A blazingly fast, multi-platform KenKen puzzle solver and generator
written in pure Rust with `unsafe` isolated to `kenken-simd`
```

## Status

**Active development | Strict local gates passing | Verification hardening in progress**

- Deterministic solver with 40-50% optimization speedup
- Multi-platform: CLI, WASM, GTK4, Android, iOS
- World-class accessibility: 5 CVD-safe color variants
- Interactive tutorial system: 5 lessons
- Comprehensive solver visualization framework

## Quick Start (3 Minutes)

### Prerequisites

```
Rust nightly-2026-04-06 (pinned in rust-toolchain.toml)
cargo toolchain
just task runner
GTK4 dev libraries (for UI)
docker (optional, for reproducible builds)
```

### CLI Solver

```bash
# Clone
git clone https://github.com/eirikr/rustykeen.git
cd rustykeen

# Canonical local validation
just ci

# Solve a puzzle
cargo run -p kenken-cli --bin kenken-cli -- solve \
  --n 2 \
  --desc "b__,a3a3" \
  --tier normal

# Benchmark solver throughput
cargo run -p kenken-cli --bin kenken-cli -- benchmark --n 4 --count 10 --tier normal

# Count solutions (uniqueness check)
cargo run -p kenken-cli --bin kenken-cli -- count --n 2 --desc "b__,a3a3" --limit 2
```

### Cross-Platform SDL2 Demo

```bash
# Requires SDL2 runtime/dev packages (Linux: libsdl2-dev)
cargo run -p kenken-sdl2 -- --n 4 --desc "<sgt-desc>" --tier normal

# Quick sample
cargo run -p kenken-sdl2 -- --n 2 --desc "b__,a3a3" --tier normal
```

Demo controls:
- Mouse + keyboard cell input
- Native resolution-aware window sizing
- Dynamic scaling via `+/-` and mouse wheel
- Fullscreen toggle (`F11`)
- Engine-backed solve/uniqueness checks (`S`/`U`)

### Desktop Application

```bash
# Install GTK4
# Ubuntu/Debian: sudo apt-get install libgtk-4-dev
# macOS: brew install gtk4

# Build and run
cargo run -p kenken-ui

# Features:
# - Interactive grid with mouse + keyboard
# - 5 CVD-safe color themes
# - Tutorial system (5 lessons)
# - Solver visualization
# - Real-time statistics
```

### Browser (WASM)

```bash
# Build WASM
cargo install wasm-pack
wasm-pack build kenken-wasm --target web

# Run local server
cd kenken-wasm
python -m http.server 8000

# Open http://localhost:8000 in browser
```

## Architecture

### Solver Algorithm

```
Input: Puzzle
  |
  v
[Initialize Domains] (1..n for each cell)
  |
  v
[Propagate Constraints]
  +---> Row/Column uniqueness
  +---> Cage operations (Add/Mul/Sub/Div)
  +---> Cross-constraint elimination
  |
  v
[Check Status]
  +---> All assigned? --> SUCCESS
  +---> Contradiction? --> BACKTRACK
  +---> Undetermined? --> Continue
  |
  v
[Select Cell] (MRV heuristic)
  |
  v
[Try Candidates] (LCV ordering)
  |
  v
[Recursive Solve]
```

### Crate Structure

```
kenken-core
  Puzzle model, validation, SGT parser

kenken-sdl2
  Cross-platform SDL2 demo frontend
  Keyboard/mouse input + adaptive scaling

kenken-solver
  Backtracking solver with optimizations
  - MRV heuristic
  - Multi-level deduction
  - Optional DLX / SAT backends

kenken-gen
  Puzzle generator with cage partitioning
  Uniqueness verification

kenken-ui
  GTK4 desktop UI with visualization
  5 CVD color themes
  Tutorial system

kenken-wasm
  Browser deployment
  Opaque handle pattern API

kenken-cli
  Reference command-line tool
```

## Developer Workflow

Local validation is canonical in this repo. Hosted GitHub Actions workflows are
intentionally disabled because of quota restrictions.

```bash
# Show available repo tasks
just

# Canonical pre-merge gate
just ci

# Individual gates
just fmt
just lint
just test
just audit
```

Fuzz maintenance is part of the same local discipline:

```bash
just fuzz-check
just fuzz-outdated
just fuzz-audit
```

Curated benchmark, profiling, and crash evidence belongs under `artifacts/`
with a sidecar `metadata.json` derived from
`artifacts/METADATA_TEMPLATE.json`, not in the repo root.

## Features

### Solver

```
Domain Representations (choose one):
  32-bit (n <= 31)
  64-bit (n <= 63)
  256-bit SIMD (n <= 256)
  BitVector (n <= 256)

Heuristics:
  MRV: Minimum Remaining Values
  LCV: Least Constraining Value
  ARC: Arc consistency propagation

Optional Backends:
  DLX: Dancing Links (Latin squares)
  SAT: Varisat solver (constraint encoding)

Deduction Tiers:
  None: No deduction, search only
  Easy: Row/column constraints
  Normal: + cage constraints
  Hard: + advanced propagation
```

### Generator

```
Cage Partitioning:
  Balanced: Equal-sized cages
  Random: Random cage sizes

Operations:
  Add, Subtract, Multiply, Divide

Difficulty Classification:
  Easy: Solvable with deduction alone
  Normal: Requires minimal backtracking
  Hard: Deep search tree required

Verification:
  Uniqueness checking
  Multi-core parallel solving
```

### UI & Accessibility

```
Desktop (GTK4):
  Interactive grid rendering
  Mouse + keyboard controls
  5 CVD color themes
  Real-time statistics
  Tutorial system (5 lessons)
  Event timeline for deductions
  Ripple effects for propagation

Web (WASM):
  Modern browser support
  Opaque handle pattern API
  No Puzzle serialization

Accessibility:
  WCAG AAA contrast compliance
  Keyboard-only navigation
  Screen reader support
  Color Vision Deficiency variants:
    - Standard (full color)
    - Protanopia (red-blind)
    - Deuteranopia (green-blind)
    - Tritanopia (blue-blind)
    - Monochrome (grayscale)
```

## Performance

### Benchmarks (M2 MacBook Pro, Normal Tier)

```
Grid Size    Difficulty    Time        Memory
2x2          Easy          0.05ms      100KB
3x3          Easy          0.3ms       150KB
4x4          Normal        3ms         200KB
5x5          Normal        15ms        250KB
6x6          Hard          80ms        300KB
8x8          Hard          300ms       500KB
9x9          Hard          800ms       700KB

Optimizations Applied:
  Tier 1.1: Cage tuple memoization    (+40-52%)
  Tier 1.2: Domain constraint filter  (+2-18%)
```

## Testing

### Coverage

```
Unit Tests:        97+ passing
Integration Tests: 8+ comprehensive
Property Tests:    10+ randomized
Formal Verification: Kani bounded model checking

Running Tests:
  cargo fmt --check
  cargo clippy --all-targets --all-features -- -D warnings
  cargo test --all-targets --all-features
  cargo kani --tests
  cargo flamegraph -p kenken-cli -- solve ...
```

### Test Corpus

```
Golden Corpus: 65+ verified puzzles
  2x2: 3 puzzles
  3x3: 5 puzzles
  4x4: 10 puzzles
  5x5: 15 puzzles
  6x6: 20 puzzles
  8x8: 7 puzzles
  9x9: 5 puzzles

Difficulty Distribution:
  Easy:   30%
  Normal: 50%
  Hard:   20%
```

## Platform Support

### Tested Environments

```
Desktop:
  Linux:   Ubuntu 22.04+, Alpine
  macOS:   13+ (x86, M1/M2/M3)
  Windows: 11+ (WSL2, native)

Web:
  Chrome:  90+
  Firefox: 88+
  Safari:  14+
  Edge:    90+

Mobile:
  Android: 8+ (arm64-v8a via NDK)
  iOS:     14+ (Swift interop)

Containers:
  Docker:  20.10+
  Podman:  3.0+
```

## Building for Production

### Release Build

```bash
# Portable Linux (x86-64-v1)
RUSTFLAGS="-C target-cpu=x86-64-v1" \
  cargo build --release -p kenken-cli --all-features

# Optimized Linux (x86-64-v3)
RUSTFLAGS="-C target-cpu=x86-64-v3" \
  cargo build --release -p kenken-cli --all-features

# With PGO optimization
./scripts/pgo.sh gen
./scripts/pgo.sh train -- cargo run -p kenken-cli --bin kenken-cli -- count --n 6
./scripts/pgo.sh use

# Size: ~8MB binary
# Performance: +40-50% vs baseline
```

### Docker

```bash
# Build image
docker build -t rustykeen:latest .

# Run solver in container
docker run --rm rustykeen solve \
  --n 6 \
  --desc "a_____,a_____,a_____,a_____,a_____,a_____"
```

## Development

### Setup

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install nightly toolchain
rustup toolchain install nightly-2026-04-06

# Clone and build
git clone https://github.com/eirikr/rustykeen.git
cd rustykeen
cargo build

# Verify
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

### Contributing

See CONTRIBUTING.md for:
  Code standards
  PR workflow
  Testing requirements
  Commit conventions

### Code Style

```
Language:    Rust 2024 edition
Formatter:   cargo fmt (enforced in CI)
Linter:      cargo clippy --all-targets --all-features -- -D warnings
Safety:      forbid(unsafe_code) everywhere except kenken-simd
Tests:       Required for all public functions
Docs:        Required for public APIs
Ascii-only:  No Unicode, no emojis, no smart quotes
```

## Licensing

**Dual Licensed:**

1. **MIT License** (LICENSE-MIT): Permissive open-source license
2. **GPL-2.0-only** (LICENSE): Copyleft open-source license

Choose the license that best fits your use case.

All dependencies are GPL-2.0 compatible.

## Quick Reference

```
Build:      cargo build
Test:       cargo test --all-targets --all-features
Lint:       cargo clippy --all-targets --all-features -- -D warnings
Format:     cargo fmt --check
Docs:       cargo doc --open
Release:    cargo build --release
Docker:     docker build -t rustykeen:latest .
```

## Documentation

```
Included:
  README.md              (this file)
  docs/INDEX.md          (comprehensive index)
  docs/book/             (mdbook source)
  CONTRIBUTING.md        (contributor guide)
  CLAUDE.md              (project instructions)

Topics Covered:
  Architecture & Design
  Solver Algorithm
  Generator Strategy
  Optimization (PGO/BOLT)
  Platform Deployment
  Contributing Guidelines
```

---

Made with care in Rust. Fast, reliable, accessible.

**Current Status: Active development | Nightly pinned | See docs/lacunae_audit.md for open debt**
