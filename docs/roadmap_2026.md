# Roadmap 2026: Implementation Lacunae and Next Steps

This document provides a prioritized roadmap for addressing implementational gaps
identified through audit of `docs/plan.md`, `docs/lacunae_audit.md`, and codebase analysis.

Last updated: 2026-01-01

## Executive Summary

The rustykeen engine has achieved **Phase A/B maturity** with:
- Working solver with deduction tiers and difficulty classification
- DLX and SAT backends (Latin constraints)
- Basic generator with uniqueness verification
- Snapshot serialization (rkyv v2)
- UniFFI bindings and CLI

Key gaps for **Phase C/D** completion:
1. SAT encoding for full cage arithmetic (Add/Mul)
2. Generator hardening (minimizer, difficulty calibration)
3. Expanded test corpus and formal verification
4. Performance baselines and API stability policy

---

## Phase 1: SAT Cage Encoding Completion (High Priority)

### Status
- `sat_cages.rs` handles Eq (1-cell) and Sub/Div (2-cell) cages
- Add/Mul multi-cell cages fall back to tuple enumeration
- Threshold `SAT_TUPLE_THRESHOLD = 512` defined but fallback not wired

### Tasks

#### 1.1 Add/Mul Cage Tuple Encoding
**File**: `kenken-solver/src/sat_cages.rs`

Implement selector-variable encoding for Add/Mul cages:
```rust
fn add_addmul_cage_clauses(
    solver: &mut Solver,
    map: &LatinVarMap,
    cage: &Cage,
    rules: Ruleset,
) -> Result<bool, SatEncodeError>
```

Algorithm:
1. Enumerate valid tuples via `Cage::valid_permutations(n, rules, SAT_TUPLE_THRESHOLD)`
2. If `tuples.is_none()` (threshold exceeded), return `Err(TuplesExceeded)`
3. For each tuple, create selector variable `s_t`
4. Add implication clauses: `s_t -> (cell_0 = tuple[0]) AND (cell_1 = tuple[1]) ...`
5. Add at-least-one clause: `s_1 OR s_2 OR ... OR s_T`
6. Add at-most-one pairwise clauses

**Estimated complexity**: ~150 lines
**Dependencies**: Existing `LatinVarMap`, `Cage::valid_permutations`

#### 1.2 Fallback Strategy When Threshold Exceeded
**File**: `kenken-solver/src/sat_cages.rs`

When tuple enumeration exceeds threshold:
```rust
pub fn solve_with_sat_cages(
    puzzle: &Puzzle,
    rules: Ruleset,
) -> Result<SatCageResult, SolveError> {
    // Try full SAT encoding
    match encode_all_cages(puzzle, rules) {
        Ok(solver) => run_sat_solver(solver),
        Err(TuplesExceeded { cage_idx }) => {
            // Fall back to backtracking for this puzzle
            let count = count_solutions_up_to_with_deductions(puzzle, rules, DeductionTier::Hard, 2)?;
            Ok(SatCageResult::Fallback { count })
        }
    }
}
```

#### 1.3 Full Cage Uniqueness Proofs
Extend `sat_cages::is_unique_with_cages()` to:
1. Encode Latin + all cage constraints
2. Find one solution
3. Add blocking clause for that solution
4. Check for UNSAT (proves uniqueness)

---

## Phase 2: Generator Hardening (High Priority)

### Status
- `kenken-gen/src/generator.rs` produces unique puzzles
- No minimization (puzzles may have redundant cages)
- Difficulty scoring uses tier-required (now implemented)

### Tasks

#### 2.1 Puzzle Minimizer
**File**: `kenken-gen/src/minimizer.rs` (new)

Algorithm (greedy removal):
```rust
pub fn minimize_puzzle(
    puzzle: Puzzle,
    rules: Ruleset,
    tier: DeductionTier,
) -> Result<Puzzle, GenError> {
    let mut current = puzzle;
    loop {
        let mut improved = false;
        for i in 0..current.cages.len() {
            let mut candidate = current.clone();
            // Merge cage i with neighbor or convert to Eq cages
            let merged = try_merge_cage(&candidate, i);
            if merged.is_some() {
                let merged = merged.unwrap();
                if is_still_unique(&merged, rules, tier)? {
                    current = merged;
                    improved = true;
                    break;
                }
            }
        }
        if !improved { break; }
    }
    Ok(current)
}
```

#### 2.2 Difficulty Scoring Integration
**File**: `kenken-gen/src/generator.rs`

Update `generate()` to:
1. Use `classify_tier_required()` to determine puzzle difficulty
2. Accept/reject based on target difficulty
3. Track acceptance rate for tuning

```rust
pub struct GenerateConfig {
    // ... existing fields ...
    pub target_difficulty: DifficultyTier,
    pub difficulty_tolerance: u8, // Allow +/- N tiers
}

pub fn generate(config: GenerateConfig) -> Result<GeneratedPuzzle, GenError> {
    for attempt in 0..config.max_attempts {
        let puzzle = generate_candidate(&config)?;
        let result = classify_tier_required(&puzzle, config.rules)?;
        let difficulty = classify_difficulty_from_tier(result);

        if within_tolerance(difficulty, config.target_difficulty, config.difficulty_tolerance) {
            return Ok(GeneratedPuzzle { puzzle, solution, difficulty });
        }
    }
    Err(GenError::AttemptsExhausted { attempts: config.max_attempts })
}
```

---

## Phase 3: Expanded Test Corpus (Medium Priority)

### Status
- `corpus_sgt_desc.rs`: 7 puzzles (2x2, 3x3, 4x4)
- `corpus_difficulty.rs`: 4 Easy-tier puzzles
- Property tests for cage semantics

### Tasks

#### 3.1 Golden Corpus Expansion
**File**: `kenken-solver/tests/corpus_golden.rs` (new)

Target: 50+ puzzles across:
- Grid sizes: 4x4, 5x5, 6x6, 7x7, 8x8, 9x9
- Difficulty tiers: Easy, Normal, Hard, Extreme
- Cage types: Add-heavy, Mul-heavy, Mixed, Sub/Div

Sources:
1. Generate with different seeds and difficulties
2. Import from upstream sgt-puzzles (if format compatible)
3. Hand-craft edge cases (large cages, minimal cages)

Format:
```rust
struct GoldenPuzzle {
    n: u8,
    desc: &'static str,
    expected_solutions: u32,
    expected_tier: Option<DeductionTier>,
    expected_difficulty: DifficultyTier,
    solution: Option<&'static [u8]>, // Known solution if unique
    label: &'static str,
}
```

#### 3.2 Normal/Hard Tier Puzzles
Create puzzles that specifically require:
- **Normal tier**: Per-cell cage tuple pruning
- **Hard tier**: Cross-cage row/col elimination

Method:
1. Generate many puzzles with target tier
2. Verify with `classify_tier_required()`
3. Select diverse examples

---

## Phase 4: Solver Optimizations (Medium Priority)

### Status
- BitDomain exists in `kenken-core/src/domain.rs` but unused
- Solver uses `u32` bitmasks for domains
- Partial evaluation not formalized

### Tasks

#### 4.1 BitDomain Integration
**File**: `kenken-solver/src/solver.rs`

Evaluate whether `BitDomain` (bitvec-based) offers advantages:
- Larger N support (N > 16)
- Better cache behavior for sparse domains
- Profile before adopting

If beneficial:
```rust
#[cfg(feature = "core-bitvec")]
type Domain = BitDomain;

#[cfg(not(feature = "core-bitvec"))]
type Domain = u32;
```

#### 4.2 Partial Evaluation Semantics
**File**: `kenken-solver/src/propagate.rs` (new module)

Formalize partial constraint checking:
```rust
pub trait PartialConstraint {
    /// Check if constraint can still be satisfied with partial assignment.
    fn is_feasible(&self, assigned: &[Option<u8>]) -> bool;

    /// Compute domain restrictions from partial assignment.
    fn restrict_domains(&self, assigned: &[Option<u8>], domains: &mut [u32]);
}

impl PartialConstraint for Cage {
    fn is_feasible(&self, assigned: &[Option<u8>]) -> bool {
        match self.op {
            Op::Add => partial_add_feasible(self, assigned),
            Op::Mul => partial_mul_feasible(self, assigned),
            // ...
        }
    }
}
```

---

## Phase 5: Formal Verification (Low Priority)

### Status
- `docs/formal_verification.md` outlines strategy
- No Kani harnesses implemented
- Z3 uniqueness verification stub exists

### Tasks

#### 5.1 Kani Harnesses
**File**: `kenken-core/src/puzzle.rs` (verification module)

```rust
#[cfg(kani)]
mod verification {
    use super::*;

    #[kani::proof]
    fn cell_coord_roundtrip() {
        let n: u8 = kani::any();
        kani::assume(n >= 2 && n <= 9);
        let idx: u16 = kani::any();
        kani::assume(idx < (n as u16) * (n as u16));

        let cell = CellId(idx);
        let coord = cell.to_coord(n);
        let back = CellId::from_coord(coord, n);
        kani::assert(cell == back, "roundtrip failed");
    }

    #[kani::proof]
    fn latin_row_constraint() {
        let n: u8 = kani::any();
        kani::assume(n >= 2 && n <= 4);
        // Verify row constraint logic
    }
}
```

---

## Phase 6: Documentation and API Stability (Ongoing)

### Tasks

#### 6.1 Benchmark Baseline Table
**File**: `docs/benchmark_baselines.md` (new)

Record performance on reference hardware:
```markdown
## Solver Benchmarks (AMD Ryzen 9, 64GB RAM)

| Grid | Difficulty | Median Solve (ms) | P99 Solve (ms) | Memory (KB) |
|------|------------|-------------------|----------------|-------------|
| 4x4  | Easy       | 0.05              | 0.1            | 4           |
| 6x6  | Normal     | 2.1               | 5.0            | 16          |
| 9x9  | Hard       | 45.0              | 120.0          | 64          |

## Generator Benchmarks

| Grid | Target     | Median Gen (ms) | Accept Rate | Attempts |
|------|------------|-----------------|-------------|----------|
| 6x6  | Normal     | 150             | 12%         | ~8       |
| 9x9  | Hard       | 2500            | 3%          | ~33      |
```

#### 6.2 Public API Stability Policy
**File**: `docs/api_stability.md` (new)

```markdown
# API Stability Policy

## Version 0.x (Current)
- API is unstable; breaking changes may occur in minor versions
- IO formats are versioned and backwards-compatible
- Feature flags may change

## Version 1.x (Future)
- Public API follows semver strictly
- Deprecated items have 1 minor version grace period
- IO formats maintain backwards compatibility

## What Constitutes Breaking Change
- Removing public types, functions, or traits
- Changing function signatures
- Changing enum variants
- Changing struct field visibility
- Changing behavior of existing functions

## What Is NOT Breaking
- Adding new public items
- Adding new optional features
- Performance improvements
- Bug fixes that change incorrect behavior
```

---

## Implementation Priority Matrix

| Phase | Task | Priority | Effort | Dependencies |
|-------|------|----------|--------|--------------|
| 1.1 | Add/Mul SAT encoding | High | Medium | None |
| 1.2 | SAT fallback logic | High | Low | 1.1 |
| 1.3 | Cage uniqueness proofs | High | Medium | 1.1, 1.2 |
| 2.1 | Puzzle minimizer | High | Medium | None |
| 2.2 | Difficulty scoring | High | Low | None (done!) |
| 3.1 | Golden corpus 50+ | Medium | Medium | None |
| 3.2 | Normal/Hard puzzles | Medium | Low | 3.1 |
| 4.1 | BitDomain integration | Medium | Medium | Profiling |
| 4.2 | Partial evaluation | Medium | High | None |
| 5.1 | Kani harnesses | Low | Medium | None |
| 6.1 | Benchmark baselines | Ongoing | Low | Benchmarks |
| 6.2 | API stability docs | Ongoing | Low | None |

---

## Next Actions (Immediate)

1. **Start with SAT Add/Mul encoding** (Phase 1.1) - highest impact
2. **Run benchmarks** and record baselines (Phase 6.1)
3. **Generate diverse corpus** for regression testing (Phase 3.1)

---

## References

- `docs/plan.md` - Master implementation plan
- `docs/lacunae_audit.md` - Gap analysis
- `docs/lacunae_deep_dive.md` - Detailed gap breakdown
- `docs/work_done.md` - Current implementation status
- `docs/sat_cage_encoding.md` - SAT encoding design
