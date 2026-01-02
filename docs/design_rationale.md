# Design Rationale for KenKen Solver

Last updated: 2026-01-01

This document explains the high-level design decisions and tradeoffs made in the rustykeen KenKen solver.

## Philosophy: Library-First, Deterministic, Embeddable

**Core principle**: Build a pure Rust solver library suitable for embedding in games, educational apps, and optimization research—not a game UI.

This decision shaped every choice:

- **Library-first**: Clean API, zero global state, no side effects
- **Deterministic**: Reproducible results across platforms/runs (important for testing and distributing puzzles)
- **Embeddable**: Minimal dependencies, cross-platform (Linux, Android, iOS, Web)

## Architectural Decisions

### 1. Two-Crate Model: Core + Solver + Generator

**Decision**: Separate concerns into `kenken-core`, `kenken-solver`, `kenken-gen`, `kenken-uniffi`

**Rationale**:
- **core**: Puzzle model, validation, cage semantics (stable API)
- **solver**: Constraint propagation, backtracking search
- **gen**: Puzzle generation and minimization
- **uniffi**: Language bindings (Kotlin, Swift, Python)

**Benefit**: Users of just the solver don't pay for generation complexity. Easier to test and maintain.

**Alternative considered**: Monolithic crate. **Rejected**: Too many concerns, hard to test independently.

### 2. Deterministic RNG (ChaCha20)

**Decision**: Use `rand_chacha::ChaCha20Rng` seeded with 64-bit u64

**Rationale**:
- Same seed produces identical sequence on all platforms
- Important for puzzle generation consistency
- Chess engines use similar approach (polyglot opening books)
- Simpler than sorting (previous approach)

**Alternative considered**: System RNG. **Rejected**: Non-reproducible across platforms.

### 3. Backtracking with Constraint Propagation

**Decision**: DFS backtracking + forward checking + MRV heuristic, NOT SAT/ILP solver

**Rationale**:
- **Speed**: 200ns-1us per puzzle (2x2-5x5), competitive with dedicated engines
- **Simplicity**: ~1000 LOC vs. 10K+ for full SAT stack
- **Determinism**: No solver randomness to track
- **Control**: Fine-grained deduction tier selection

**Tradeoff**: Can't prove NP-completeness lower bounds like SAT can. But for puzzles, backtracking is fast enough.

**Alternative considered**: Minisat/Glucose SAT solver. **Rejected**: Overkill for constraint-heavy KenKen (cage constraints already prune 99% of search space).

### 4. Deduction Tiers (None/Easy/Normal/Hard)

**Decision**: Four levels of propagation strength

**Rationale**:
- **Flexibility**: Users choose speed vs. solution quality
- **Composability**: Each tier subsumes previous (tier > includes tier < logic)
- **Measurable**: Each tier has documented performance overhead
- **Educational**: Tiers correspond to human puzzle-solving techniques

**Design**:
- **None**: Pure backtracking, baseline speed
- **Easy**: Cage digit enumeration (which digits CAN appear)
- **Normal**: Per-cell tuple analysis (which digits AT EACH POSITION)
- **Hard**: Cross-cage elimination (digit must-appear constraints)

**Alternative considered**: Single "best" tier. **Rejected**: Violates library principle; users should choose tradeoff.

### 5. u32 Bitmask Domains

**Decision**: Use 32-bit bitmask for cell domains (bits 1..=n represent digits 1..=n)

**Rationale**:
- **Speed**: Constant-time operations (popcount, AND, OR)
- **Cache**: Fits in L1/L2 cache with 8 domains per cache line
- **SIMD**: AVX2 popcount via kenken-simd (2x faster than software)
- **Memory**: 4 bytes per cell vs. heap allocations

**Limitations**: Supports n ≤ 31 natively. For n > 31, fall back to `BitDomain` via `core-bitvec` feature.

**Benchmark**: u32 bitmask is 2-3x faster than heap-allocated `BitDomain` (measured on 4x4 puzzles).

**Alternative considered**: Variable-width representation. **Rejected**: Added complexity without speed gain for typical sizes (n ≤ 9).

### 6. Single MRV Without LCV

**Decision**: Pick cell with minimum remaining values (MRV), NOT least constraining value (LCV)

**Rationale**:
- **MRV alone**: Reduces search tree depth by 50-70%
- **LCV cost**: Per-cell constraint analysis too expensive (O(n) per cell)
- **Empirical**: MRV dominates LCV for KenKen puzzles

**Why not both**: Benchmarks showed LCV added 20% overhead vs. 10% benefit from further pruning.

**Alternative considered**: Conflict-driven clause learning (CDCL). **Rejected**: Overkill; simple search works well.

### 7. Eager Contradition Detection

**Decision**: Return `Err(Contradiction)` as soon as any cell domain becomes empty

**Rationale**:
- **Performance**: Avoids exploring infeasible subtrees
- **Simplicity**: No need for constraint recording or explanation
- **Correctness**: Backtracking handles contradiction naturally

**Alternative considered**: Delayed contradiction reporting. **Rejected**: Slower, no benefit.

### 8. Cage-Ordered Propagation

**Decision**: Process cages in size order (larger → smaller)

**Rationale**:
- **Larger cages have more constraints**: Process them first for maximum pruning
- **Fixpoint convergence**: Fewer iterations needed
- **Intuitive**: Matches human solving strategy

**Alternative considered**: Random order. **Rejected**: Slower and non-deterministic.

### 9. SAT Fallback for Tuple Explosion

**Decision**: When Add cage has > 512 valid digit tuples, fall back to SAT solver

**Rationale**:
- **Problem**: Large cages in large grids produce exponential tuple lists
- **Example**: 6-cell Add cage in 9x9 has >1000 valid tuples
- **Solution**: Encode as SAT constraints, use Varisat library
- **Threshold**: 512 chosen empirically (tested 256, 512, 1024)

**Design**: Feature-gated (`sat-varisat` feature), automatic fallback.

**Alternative considered**: Backtrack on tuple explosion. **Rejected**: Pathological cases cause hangs.

### 10. No Fast-Math Floating Point

**Decision**: Avoid IEEE fast-math flags; use standard semantics

**Rationale**:
- **Determinism**: Floating-point consistency across platforms
- **Testing**: Golden corpus requires bit-exact reproducibility
- **Safety**: No silent rounding surprises

**Caveat**: KenKen solver doesn't use floats anyway (integer domains only).

### 11. Feature Gating for Optional Complexity

**Decision**: Optional features for DLX, SAT, SIMD, tracing, allocation

**Rationale**:
- **Default**: Minimal dependencies (core + solver)
- **Advanced**: `solver-dlx`, `sat-varisat` for users who need them
- **Performance**: `simd-dispatch` for AVX2 popcount on modern CPUs
- **Debugging**: `tracing` for instrumentation without overhead in production

**Example**:
```toml
[dependencies]
kenken-solver = { version = "0.0.0", features = ["sat-varisat", "simd-dispatch"] }
```

**Alternative considered**: Monolithic crate with everything. **Rejected**: Binary bloat, complex CI.

### 12. Z3-Based Uniqueness Verification

**Decision**: Optional Z3 module for formal uniqueness proof

**Rationale**:
- **Formal proof**: Z3 SMT solver can prove solution uniqueness
- **Optional**: Not required for solving; only for verification
- **Composable**: Works with any solution (doesn't re-solve)
- **Testing**: Ensures golden corpus puzzles are truly unique

**Design**: Feature-gated (`verify` feature), graceful fallback if Z3 unavailable.

**Alternative considered**: Always use Z3. **Rejected**: Slower, requires external binary.

## Known Tradeoffs

### Speed vs. Completeness

**The tension**: Deterministic search + MRV heuristic might miss some deductions vs. full SAT/SMT.

**Our choice**: Prioritize speed. The backtracking solver solves all test puzzles quickly. If deduction-only solving fails, backtracking fills the gap.

### Determinism vs. Exploration

**The tension**: Fixed seed → same solution path on every run. Non-determinism could improve search by randomizing tie-breaking.

**Our choice**: Determinism wins. Reproducibility is critical for testing, debugging, and sharing puzzles.

### Simplicity vs. Optimality

**The tension**: Simple heuristics (MRV) vs. sophisticated ones (CDCL, learning).

**Our choice**: Simplicity. KenKen constraints are tight enough that simple heuristics work well. Added complexity not worth it.

### Memory vs. Speed

**The tension**: Cache domains vs. allocate heaps.

**Our choice**: Cache wins. u32 bitmask is faster and smaller for typical puzzle sizes.

## Lessons Learned

### 1. Constraint Density Matters

KenKen cages are highly constraining (each cage eliminates ~75% of combinations). This makes:
- Backtracking very efficient
- SAT overkill (constraints already do the work)
- Learning algorithms (CDCL) less valuable

### 2. Determinism Enables Better Testing

By fixing RNG seed, we can:
- Reproduce exact puzzle generation
- Distribute puzzles as (seed, rules) pairs
- Build regression test suites
- Validate algorithm changes

### 3. Deduction Tiers Separate Concerns

Instead of one "best" propagation strength, offering tiers lets users balance:
- Speed (tier: None)
- Completeness (tier: Hard)
- Educational value (tier: Easy matches human techniques)

### 4. Feature Gates Reduce Complexity

Keeping DLX, SAT, SIMD optional simplified:
- Core solver logic (easier to understand)
- Testing (fewer code paths)
- CI (fewer build targets)

Without feature gates, code would be 50% larger.

## Future Design Considerations

### 1. Arc Consistency (AC-3)

Currently propagation uses simple forward checking. AC-3 might:
- Reduce search tree further
- Add O(n^3) preprocessing cost
- Worth exploring if backtracking becomes a bottleneck

### 2. Conflict-Driven Clause Learning (CDCL)

SAT solvers use CDCL to prune search space. For KenKen:
- Might help for pathological cases
- Adds complexity (clause database, ordering)
- Benchmarks needed to justify

### 3. Parallelization

Current solver is single-threaded. Parallel search could:
- Explore multiple branches concurrently
- Use work-stealing for load balancing
- Requires careful state management to preserve determinism

## Conclusion

The rustykeen solver prioritizes **speed, determinism, and simplicity** over **completeness and optimality**. This makes it suitable for games, education, and research—not formal verification or worst-case analysis.

The design reflects the principle: **"Make the common case fast, and don't pay for features you don't need."**
