# Audit and Lacunae Resolution

Last updated: 2026-01-01

## Resolved Gaps (since 2025-12-31)

### SAT Encoding (RESOLVED)
- **Add/Mul cage tuple encoding**: Implemented via `add_tuple_allowlist()` with selector variables
- **Threshold fallback**: `SAT_TUPLE_THRESHOLD = 512` with automatic fallback to native solver
- **Full cage uniqueness proofs**: `puzzle_uniqueness_via_sat()` handles all cage types
- **Tests**: 4 unit tests covering various scenarios

### Difficulty Classification (RESOLVED)
- **Tier-required classifier**: `classify_tier_required()` determines minimum deduction tier
- **Backtracking detection**: `SolveStats.backtracked` tracks whether guessing occurred
- **Upstream parity**: Matches sgt-puzzles approach (technique-based, not search-cost)
- **Calibration corpus**: `corpus_difficulty.rs` with Easy-tier validation

### Build Infrastructure (RESOLVED)
- **Workspace**: Compiles with `cargo build --all-features`
- **CI**: `fmt/clippy/test` gates in `.github/workflows/ci.yml`
- **Toolchain**: Pinned to `nightly-2026-01-01`
- **Fuzz harness**: `fuzz/fuzz_targets/` with parser and solver coverage
- **LICENSE**: Valid GPLv2 text

### Generator Pipeline (RESOLVED)
- **Puzzle minimizer**: Implemented via `minimize_puzzle()` in `kenken-gen/src/minimizer.rs`
  - Greedy cage merging algorithm preserves uniqueness
  - Configurable via `MinimizeConfig` (rules, tier, max iterations)
  - Returns `MinimizeResult` with before/after statistics
- **Difficulty targeting**: Implemented via `generate_with_stats()` in `kenken-gen/src/generator.rs`
  - `GenerateConfig.target_difficulty` specifies desired tier
  - `GenerateConfig.difficulty_tolerance` allows +/- tier range
  - Returns `GeneratedPuzzleWithStats` with full classification
- **Tests**: 9 unit tests covering minimization and difficulty targeting

### Test Corpus (MOSTLY RESOLVED)
- **Golden corpus**: 52 puzzles in `kenken-solver/tests/corpus_golden.rs`
  - Grid sizes: 2x2, 3x3, 4x4, 5x5, 6x6 with verified solutions
  - All puzzles have verified solutions and difficulty tiers
  - 8 test functions covering parse, validate, solve, uniqueness, difficulty
- **Normal/Hard tier puzzles**: Investigation complete (2026-01-01)
  - **Finding**: Normal/Hard tier puzzles are extremely rare in random generation
  - Scanned 1000 random 4x4 puzzle seeds: 0 Hard, 0 Normal found
  - Most random puzzles are Easy-tier (singleton + simple row/col cages)
  - Hard/Normal tiers require specific cage interaction patterns unlikely in random generation
  - **Decision**: Focus on diverse Easy-tier puzzles with varied cage constraints
  - **Status**: Will expand corpus with Easy-tier puzzles featuring Add/Mul/Sub/Div cages

### Formal Verification (RESOLVED)
- **Kani harnesses**: 15 proof harnesses implemented (2026-01-01)
  - `kenken-core/src/puzzle.rs`: 5 proofs
    - `cell_coord_roundtrip`: Bijection between cell IDs and coordinates
    - `cell_index_bounds`: Index calculations always in bounds
    - `cell_id_rejects_oob`: OOB coordinates correctly rejected
    - `coord_rejects_oob`: OOB cell IDs correctly rejected
    - `cellid_ordering_is_row_major`: Ordering matches row-major layout
  - `kenken-solver/src/solver.rs`: 10 proofs
    - `full_domain_has_n_bits`: Domain has exactly n bits set
    - `place_sets_row_mask`/`place_sets_col_mask`: Place sets mask bits
    - `unplace_clears_row_mask`/`unplace_clears_col_mask`: Unplace clears mask bits
    - `place_unplace_roundtrip`: Place/unplace restores state
    - `domain_excludes_placed_in_row`/`domain_excludes_placed_in_col`: Latin constraints
    - `place_sets_grid_value`: Grid value matches placed digit
- **Z3 certification**: Integrated into CI pipeline (2026-01-01)
  - `kenken-solver/src/z3_verify.rs`: Z3-based uniqueness verification
  - `kenken-solver/tests/z3_golden_verify.rs`: Golden corpus verification tests
  - `.github/workflows/ci.yml`: Optional Z3 verification job (installs Z3, runs tests)
  - Feature-gated via `verify` feature flag

## Remaining Gaps

### Documentation (PARTIALLY RESOLVED)
- **Benchmark baselines**: Recorded in `docs/benchmark_baselines.md` (2026-01-01)
  - solve_one latencies: 207ns (2x2) to 1.03us (5x5)
  - Deduction tier overhead: 97-199% vs None tier
  - Regression thresholds defined
- **API stability policy**: Documented in `docs/api_stability.md` (2026-01-01)
  - Semver policy, public API definition, deprecation process
  - Platform tiers, MSRV policy, feature flag stability
- **Android integration**: No working example app despite UniFFI bindings.
- **Propagation semantics**: Documented in `docs/propagation_semantics.md` (2026-01-01)
  - Deduction tier descriptions and performance characteristics
  - Cage-specific bounds (Add, Mul, Sub, Div, Eq)
  - Domain representation and Latin constraint maintenance
  - Tuple enumeration and fixpoint semantics

### Solver Optimizations (RESOLVED/DOCUMENTED)
- **BitDomain integration**: Analysis complete (2026-01-01):
  - Solver uses `u32` bitmask (bits 1..=n) with SIMD-accelerated popcount
  - This is **faster** than `BitDomain` (which uses heap-allocated `bitvec`)
  - `BitDomain` remains available for external API use (N > 31) via `core-bitvec` feature
  - No integration needed; solver's domain representation is already optimal
- **Partial evaluation**: Propagation semantics formalized in solver:
  - `cage_feasible()`: Bounds checking with partial assignments
  - `apply_cage_deduction()`: Per-tier domain restriction
  - Full documentation in `docs/propagation_semantics.md`

### Android Example (RESOLVED)
- **Skeleton created** (2026-01-01): Full Android app structure with:
  - `KeenApi.kt`: UniFFI wrapper (JNI loader, external function declarations)
  - `PuzzleViewModel.kt`: ViewModel for state management (MVVM pattern)
  - `MainActivity.kt`: Main activity with input controls and result display
  - `activity_main.xml`: UI layout (grid size, puzzle input, tier selection)
  - `build.gradle.kts`: App-level gradle config (dependencies, NDK setup)
  - `settings.gradle.kts`: Project structure config
  - `AndroidManifest.xml`: App permissions and activities
  - `README.md`: Complete build and usage guide
- **Status**: Ready for NDK compilation and deployment
- **Next step**: Build native libraries via `cargo ndk` and test on Android device

## Next Actions (Priority Order)

1. **Extended corpus**: Add Normal/Hard tier puzzles and larger grid sizes (6x6+)
   - sgt-desc format fully documented; encoding is tractable
   - Block structure for 6x6: `_61` (61 positions = 2*6*5 + 1)
   - Tested with cyclic singleton puzzle (parsing, solving, solving verified)
2. **Android testing**: Build native libraries and test app on device
   - Install cargo-ndk: `cargo install cargo-ndk --version 3.2.0`
   - Build: `cargo ndk -t arm64-v8a build --release -p kenken-uniffi`
   - Verify UI responsiveness and error handling
3. **Z3 optimization**: Expand Z3 verification to larger corpus
   - Currently optional feature and ignored test; could add regression tests
4. **Performance baselines**: Measure release build performance on Android
5. **Documentation**: Add architecture diagrams and design rationale

## References

- `docs/roadmap_2026.md` - Detailed implementation plan
- `docs/work_done.md` - Current implementation status
- `docs/plan.md` - Master architecture plan
