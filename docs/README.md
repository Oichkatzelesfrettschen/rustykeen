# Documentation Index

This directory contains design documents, architecture notes, and implementation plans for rustykeen.

## Quick Start

| Topic | File | Description |
|-------|------|-------------|
| Architecture | `architecture.md` | Workspace layout, data flow, threading model |
| Build & Targets | `target_matrix.md` | Build targets, CPU tuning, cross-compilation |
| Master Plan | `plan.md` | Synthesized implementation roadmap |

## Core Documentation

### Architecture & Design
- `architecture.md` - Workspace layout and design overview
- `architecture_stack.md` - Metal/Brain/Scale synergy layers (distilled from crates_audit)
- `design.md` - High-level design decisions
- `engineering.md` - Engineering principles

### Cleanroom Process
- `cleanroom_policy.md` - Operational rules for cleanroom reverse-engineering
- `cleanroom_plan.md` - Technical porting plan (algorithms, data structures, steps)
- `upstream_sgt_puzzles_keen.md` - Distilled notes from upstream behavior study

### Dependencies & Features
- `crates_audit.md` - **Single source of truth** for crate selection and integration status
- `dependencies.md` - Current workspace dependencies with usage notes
- `dependency_matrix.md` - 2026 target stack (planned vs implemented)
- `crate_feature_plan.md` - Detailed feature-gating strategy
- `features.md` - Feature flag overview
- `feature_gating.md` - Feature gate rules

### Solver & Algorithms
- `dlx_mapping.md` - DLX (Algorithm X) column mapping for Latin constraints
- `exact_cover_matrix.md` - Exact cover matrix construction
- `mrv_dlx.md` - MRV heuristic with DLX
- `lazy_dlx_solvercontext.md` - Lazy DLX initialization
- `sat_cage_encoding.md` - SAT encoding for cage arithmetic (Varisat)
- `cnf_templates.md` - CNF clause templates for SAT encoding
- `latin_squares.md` - Latin square constraint background

### Android & Mobile
- `android_build.md` - Android build instructions
- `android_rust_state.md` - Rust on Android ecosystem state
- `cargo_mobile2.md` - cargo-mobile2 integration notes
- `uniffi_codegen.md` - UniFFI code generation

### Build & Optimization
- `riced_build.md` - Release profile and optimization settings
- `rust_build_system.md` - Rust build system overview
- `target_matrix.md` - Build targets and cross-compilation

### Serialization
- `rkyv_snapshot_v2.md` - Snapshot v2 design notes

### Testing & Verification
- `formal_verification.md` - Formal verification strategy (Kani, Z3)
- `corpus.md` - Test corpus strategy
- `difficulty.md` - Difficulty grading framework

### Observability
- `tracing_tracy.md` - Tracy profiler integration
- `telemetry_build_assets.md` - Telemetry asset embedding

### Tracking
- `work_done.md` - What is implemented today
- `lacunae_audit.md` - Known gaps and missing pieces
- `lacunae_deep_dive.md` - Detailed gap analysis
- `checklist.md` - Implementation checklist

## Subdirectories

### `adr/`
Architecture Decision Records (ADRs):
- `0001-ruleset-cage-semantics.md` - Cage arithmetic semantics
- `0002-puzzle-formats.md` - Puzzle format decisions
- `0003-determinism-and-rng.md` - Determinism and RNG choices
- `0004-crate-layout-and-naming.md` - Crate layout and naming

### `deps/`
Per-crate documentation with upstream links and integration notes.
See `deps/README.md` for index.

### `book/`
mdBook source for user-facing documentation.
Build with: `mdbook build docs/book`

## Deprecated Files

Files renamed to `.deprecated` are superseded by other documents:
- `crate_audit_list.md.deprecated` - Merged into `crates_audit.md`
- `master_architecture_v2026.md.deprecated` - Merged into `architecture.md`

## Code Samples

Example Rust snippets for reference:
- `global_allocator.rs` - Global allocator setup example
- `keen_engine.rs` - Engine API sketch
- `solve_dlx.rs` - DLX solver sketch
- `build.rs` - Build script example

## UDL Definition

- `uniffi.udl` - UniFFI interface definition for bindings
