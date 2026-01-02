# API Stability Policy

Last updated: 2026-01-01

## Version Status

**Current version**: 0.0.0 (pre-release, unstable)

The project is not yet at v1.0. All APIs are subject to change without notice.
This document outlines the stability policy that will apply once v1.0 is released.

## Semantic Versioning

rustykeen follows [Semantic Versioning 2.0.0](https://semver.org/):

- **MAJOR**: Breaking changes to public API
- **MINOR**: New features, backwards-compatible
- **PATCH**: Bug fixes, backwards-compatible

## Public API Definition

### Stable (post-v1.0)

The following items constitute the public API and are subject to semver:

#### kenken-core

- `Puzzle`, `Cage`, `CellId`, `Coord` structs and their public fields
- `cell_id()`, `coord()`, `cell_index()` functions
- `CoreError` variants
- `Ruleset` and rule configuration
- `Op` enum variants (Add, Mul, Sub, Div, Eq)
- `format::sgt_desc::parse_keen_desc()` function
- `BitDomain` type (behind `core-bitvec` feature)

#### kenken-solver

- `solve_one()`, `solve_one_with_deductions()`, `solve_one_with_stats()` functions
- `count_solutions_up_to()`, `count_solutions_up_to_with_deductions()` functions
- `classify_tier_required()`, `classify_difficulty_from_tier()` functions
- `Solution`, `SolveStats`, `TierRequiredResult` structs
- `DeductionTier`, `DifficultyTier` enums
- `SolveError` variants

#### kenken-gen

- `generate()`, `generate_with_stats()` functions
- `minimize_puzzle()` function
- `GenerateConfig`, `MinimizeConfig` structs
- `GeneratedPuzzle`, `GeneratedPuzzleWithStats`, `MinimizeResult` structs
- `GenError` variants

#### kenken-io

- `save_puzzle()`, `load_puzzle()` functions
- Snapshot format v2 (backwards-compatible reads)

#### kenken-uniffi

- All UniFFI-exported types and functions
- Kotlin/Swift interface stability

### Unstable (never covered by semver)

The following are explicitly **not** part of the public API:

- Internal solver state (`State`, `place()`, `unplace()`)
- Propagation internals (`propagate()`, `apply_cage_deduction()`)
- DLX/SAT backend implementations
- Benchmark harnesses
- Test utilities and corpus data
- Items marked `#[doc(hidden)]`
- Items behind `#[cfg(test)]` or `#[cfg(kani)]`

### Feature Flags

Feature flags are part of the public API surface:

| Feature | Crate | Stability |
|---------|-------|-----------|
| `core-bitvec` | kenken-core | Stable |
| `solver-dlx` | kenken-solver | Stable |
| `sat-varisat` | kenken-solver | Stable |
| `simd-dispatch` | kenken-solver | Stable |
| `tracing` | kenken-solver | Stable |
| `alloc-bumpalo` | kenken-solver | Unstable |
| `perf-likely` | kenken-solver | Unstable |
| `gen-dlx` | kenken-gen | Stable |
| `parallel-rayon` | kenken-gen | Stable |
| `verify-sat` | kenken-gen | Stable |

Unstable features may change behavior between minor versions.

## Breaking Change Policy

### Pre-v1.0

- Breaking changes can occur in any release
- CHANGELOG documents all changes
- Users should pin exact versions

### Post-v1.0

Breaking changes require a major version bump and must:

1. Be documented in CHANGELOG with migration guide
2. Have deprecation warnings in the prior minor release (when feasible)
3. Include upgrade path examples

### Exceptions (never breaking)

These changes are **not** considered breaking:

- Adding new public items (structs, functions, enum variants)
- Adding new optional fields with defaults
- Performance improvements
- Bug fixes (even if they change observable behavior)
- Expanding accepted input formats
- Adding new feature flags

## Deprecation Policy

1. Deprecated items are marked with `#[deprecated(since = "X.Y.Z", note = "...")]`
2. Deprecated items remain for at least one major version
3. Migration documentation provided in deprecation note

## Minimum Supported Rust Version (MSRV)

- Current MSRV: None (nightly-2026-01-01 required)
- Post-v1.0: MSRV will be set to latest stable at v1.0 release
- MSRV bumps require a minor version increment
- Nightly features may be used behind feature flags

## Platform Support

### Tier 1 (fully tested)

- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`

### Tier 2 (builds, limited testing)

- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`
- `aarch64-linux-android`
- `wasm32-unknown-unknown`

### Tier 3 (best effort)

- Other targets supported by Rust

## Release Cadence

- No fixed schedule
- Releases driven by feature completeness
- Security fixes released as patch versions ASAP

## Versioning Examples

```
0.1.0  -> First public pre-release
0.2.0  -> Breaking API changes (pre-v1.0, allowed)
1.0.0  -> First stable release
1.1.0  -> New features, backwards-compatible
1.1.1  -> Bug fix
1.2.0  -> MSRV bump
2.0.0  -> Breaking API change
```

## Contact

For API stability questions, open an issue with the `api-stability` label.
