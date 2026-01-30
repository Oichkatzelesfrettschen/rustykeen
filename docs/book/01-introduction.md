# Introduction to Rustykeen

Rustykeen is a production-grade Rust implementation of a KenKen-style puzzle solver and generator. It provides:

- **Deterministic solver** with 40-50% optimization speedup through caching and constraint propagation
- **Intelligent generator** using cage partitioning and uniqueness verification
- **Multi-platform support**: CLI, WASM browser, Android NDK, iOS
- **World-class accessibility**: Color vision deficiency (CVD) support with 5 variants
- **Interactive UI**: GTK4-based desktop application with solver visualization
- **Comprehensive testing**: 50+ unit tests, property tests, formal verification with Kani

## What is KenKen?

KenKen is a logic puzzle similar to Sudoku, also called "Keen" or "Arithmeticum":

- Grid-based puzzle: 2x2 to 9x9 cells
- Constraints:
  - Each row contains digits 1 through n exactly once (Latin square)
  - Each column contains digits 1 through n exactly once
  - Cages (outlined regions) must satisfy operations: Add, Subtract, Multiply, Divide
  - Operation targets are unique within each cage

Example 4x4 puzzle:
```
+---+---+---+---+
| 1 | 2 | 3 | 4 |
+---+---+---+---+
| 2 | 1 | 4 | 3 |
+---+---+---+---+
| 3 | 4 | 1 | 2 |
+---+---+---+---+
| 4 | 3 | 2 | 1 |
+---+---+---+---+
```

## Quick Start

### Solving a Puzzle

```bash
cargo run -p kenken-cli -- solve --n 4 --desc "b__,a3a3" --tier normal
```

### Generating Puzzles

```bash
cargo run -p kenken-cli -- generate --n 4 --count 10 --tier normal
```

### Browser Solver

Build WASM and open `kenken-wasm/index.html` in modern browser for interactive solving.

### Desktop Application

Build and run GTK4 UI with solver visualization, tutorial mode, and accessibility features.

## Project Structure

```
rustykeen/
├── kenken-core/      # Puzzle model, validation, SGT parser
├── kenken-solver/    # Backtracking solver with optimizations
├── kenken-gen/       # Puzzle generator with uniqueness verification
├── kenken-ui/        # GTK4 desktop UI with visualization
├── kenken-wasm/      # WASM bindings for browser deployment
├── kenken-cli/       # Reference command-line tool
└── docs/             # Comprehensive documentation
```

## Key Features

### Solver
- Minimum Remaining Values (MRV) heuristic for intelligent search
- Least Constraining Value (LCV) ordering
- Multi-level deduction: None, Easy, Normal, Hard
- Optional DLX (Dancing Links) for Latin squares
- Optional SAT-based constraint encoding
- Parallel search with rayon (feature-gated)

### Generator
- Cage partitioning for balanced puzzles
- Uniqueness verification via exhaustive search
- Deterministic RNG (ChaCha20) for reproducibility
- Difficulty classification: Easy, Normal, Hard

### UI
- Modern GTK4 with Cairo graphics
- 5 CVD-safe color variants (OKLCH color space)
- WCAG AAA contrast compliance
- Interactive tutorial: 5 lessons from basics to advanced
- Solver visualization: backtracking animation, ripple effects, timeline

### Accessibility
- Color Vision Deficiency support (Protanopia, Deuteranopia, Tritanopia)
- Monochrome mode for grayscale displays
- Keyboard-only navigation
- Screen reader support (labels, ARIA)

### Platforms
- **Desktop**: Linux (GTK4), macOS, Windows
- **Web**: WASM via opaque handle pattern (2026 best practice)
- **Mobile**: Android NDK, iOS
- **Cloud**: Docker-ready, reproducible builds

## Performance

Baseline on M2 MacBook Pro (Normal deduction tier):
- 4x4 puzzle: < 5ms
- 6x6 puzzle: < 100ms
- 8x8 puzzle: < 500ms

Optimizations applied:
- Tier 1.1: Cage tuple memoization (40-52% improvement)
- Tier 1.2: Domain constraint filtering (2-18% improvement)
- Tier 2+: MRV heuristic, constraint propagation, parallel search (planned)

## Licensing

Rustykeen is licensed under GPL-2.0-only. See LICENSE for details.

Dependencies include:
- OpenPerception design tokens (MIT)
- libDaltonLens (Public Domain) for CVD simulation
- GTK4 (LGPL)
- wasm-bindgen (MIT/Apache-2.0)

All dependencies are GPL-2.0-compatible.
