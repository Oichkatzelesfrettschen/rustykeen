# Regression Detection Thresholds

**Last Updated**: 2026-01-29
**Purpose**: Define quantitative thresholds for automated regression detection in CI/CD pipelines

---

## Executive Summary

This document establishes objective thresholds for regression detection across performance, test coverage, and code quality dimensions. These thresholds enable automated CI/CD failure/investigation triggers while preventing false positives from measurement noise.

**Primary Policy**: Thresholds are conservative (favor detection over false negatives) but account for measurement variance via baseline variance bands.

---

## Performance Regression Thresholds

### Solve Time Regression (Primary Metric)

**Threshold**: >5% slower than baseline
**Baseline**: Recorded in `docs/research/benchmark_baselines_2026-01-29.md` for grid sizes 2x2-9x9
**Measurement Tool**: `hyperfine` (100 runs per configuration)
**Precision**: Median latency ±95% CI

**Rationale**:
- 5% threshold balances genuine optimization regressions against measurement noise (~2-3% variance typical with hyperfine)
- Historical context: Phase Alpha PGO/BOLT achieved 17% speedup; >5% regression would consume ~30% of gains
- Small regressions (<2%) are expected noise; investigated regressions are >5%
- Applies to all grid sizes 2x2-9x9

**Measurement Method**:

```bash
# Establish baseline (single point)
hyperfine --runs 100 'cargo run --release -p kenken-cli -- solve --n 6 --desc <puzzle>'
# Record: median latency

# Measure in CI (after code change)
hyperfine --runs 100 'cargo run --release -p kenken-cli -- solve --n 6 --desc <puzzle>'
# Compare: if (new_median - baseline_median) / baseline_median > 0.05 => FAIL
```

**Escalation**:
- >5% regression: Investigate root cause; document or revert
- >10% regression: Automatic revert (unless explicitly approved)
- >20% regression: Critical failure; requires git blame + profiling

**Baseline Variance Band** (empirical from Phase Alpha):
- 2x2-4x4: ±2% typical variance
- 5x5-6x6: ±3% typical variance
- 8x8-9x9: ±4% typical variance (larger search space = more variance)

---

### Memory Regression (Secondary Metric)

**Threshold**: >10% more heap than baseline
**Baseline**: Recorded in `docs/benchmark_memory_2026-01-29.md` (dhat profiler)
**Measurement Tool**: `dhat` (heap allocation tracking, `--features dhat-heap`)
**Metric**: Peak heap memory in MB

**Rationale**:
- 10% threshold tolerates allocation pattern variance (depends on CPU cache behavior)
- Memory growth is often algorithmic (cache hits/misses affect peak); single-run noise can be 5-8%
- Historical context: Tier 1.1 cage tuple caching increased memory ~2-3% (well within threshold)
- Large memory regressions (>10%) indicate structural algorithm change

**Measurement Method**:

```bash
# Build with dhat support
cargo build --release --features dhat-heap

# Measure baseline
DHAT_OUT_FILE=/tmp/baseline.out ./target/release/kenken-cli solve --n 6 --desc <puzzle>
dhat_gui /tmp/baseline.out | grep "Peak"  # Record peak MB

# Measure after change
DHAT_OUT_FILE=/tmp/new.out ./target/release/kenken-cli solve --n 6 --desc <puzzle>
dhat_gui /tmp/new.out | grep "Peak"  # Compare

# Calculate: if (new_peak - baseline_peak) / baseline_peak > 0.10 => FAIL
```

**Escalation**:
- >10% regression: Investigate allocation sites; document trade-offs or fix
- >20% regression: Critical failure; likely indicates memory leak or structural change
- <-5% improvement: Profile and consider landing as optimization

**Grid-Size-Specific Baselines** (from benchmark_memory_2026-01-29.md):
- 2x2: <1 MB baseline (regress if >1.1 MB)
- 4x4: <3 MB baseline (regress if >3.3 MB)
- 6x6: <10 MB baseline (regress if >11 MB)
- 8x8: <20 MB baseline (regress if >22 MB)
- 9x9: <30 MB baseline (regress if >33 MB)

---

### Generator Performance Regression

**Threshold**: >5% slower than baseline (generation phase only)
**Baseline**: Recorded in `docs/benchmark_generator_2026-01-29.md`
**Measurement Tool**: `/usr/bin/time -v` or `hyperfine`
**Metric**: Generation time (before minimization) in milliseconds

**Rationale**:
- Generator performance is less critical than solver (runs offline); use same 5% threshold for consistency
- Acceptance rate (% of candidates passing uniqueness check) is more important than raw generation speed
- Minimization stage can be 5-15% of total time; exclude from regression metric

**Measurement Method**:

```bash
# Baseline generation (10 trials for stability)
for i in {1..10}; do
  /usr/bin/time -v cargo run --release -p kenken-cli -- \
    generate --n 5 --difficulty normal --seed $((12345 + i)) 2>&1 | grep "User time"
done | awk '{sum+=$3} END {print sum/NR}'  # Average user time

# Compare in CI: if (new_avg - baseline_avg) / baseline_avg > 0.05 => WARN
```

**Escalation**:
- >5% regression: Warning (log but do not fail CI)
- >15% regression: Investigate; likely algorithmic change
- Acceptance rate >20% drop: Critical failure

---

## Test Coverage Regression Thresholds

### Minimum Test Count

**Threshold**: <29 tests = automatic failure
**Current Count**: 29+ unit/integration tests (as of 2026-01-29)
**Test Command**: `cargo test --all-targets`

**Rationale**:
- Test deletion is always a regression (coverage decrease)
- Threshold of 29 is exact current count; losing any test is failure
- Future: As tests grow, update threshold (e.g., <40 tests after Phase Gamma)

**Measurement Method**:

```bash
# Count total test cases
cargo test --all-targets 2>&1 | grep "test result:" | awk '{print $3}'

# If < 29 => FAIL and block merge
```

**Escalation**:
- Any test deletion: Requires explicit justification and approval
- Disabled tests (via #[ignore]): Document why and expected duration
- Test refactoring: Ensure replacement tests are added

---

### Test Pass Rate

**Threshold**: <100% pass rate = automatic failure
**Current Status**: 100% pass rate (29/29 tests)

**Rationale**:
- CI enforces all tests pass; no exceptions
- Flaky tests (intermittent failure) must be fixed or removed
- Feature flags (solver-dlx, sat-varisat, etc.) must be tested in all combinations

**Test Execution Matrix**:

```bash
# All targets, default features
cargo test --all-targets                       # Must pass

# All targets, all features
cargo test --all-targets --all-features        # Must pass

# All targets, no default features
cargo test --all-targets --no-default-features # Must pass
```

**Escalation**:
- Single test failure: Blocks merge until fixed
- Flaky test (fails intermittently): Add retry logic or disable pending fix
- Timeout: Increase timeout or optimize test (e.g., reduce search space)

---

## Code Quality Regression Thresholds

### Clippy Warnings

**Threshold**: >0 warnings = automatic failure
**Current Status**: 0 warnings (after API stability work, Task 33)
**Lint Command**: `cargo clippy --all-targets --all-features`
**Policy**: `warnings = "deny"` in Cargo.toml workspace

**Rationale**:
- Zero-warning policy prevents lint debt accumulation
- Each warning is actionable and should be fixed or explicitly allowed
- Feature-gated code allowed with `#![allow(dead_code)]` + documentation

**Acceptable Exceptions** (documented):
- Dead code in feature-gated modules (e.g., `#![allow(dead_code)]` in dlx, sat_*, z3_verify)
- Unused variables in example code (feature-gated)
- Unsafe code in kenken-simd (FFI crate only; audited for safety)

**Measurement Method**:

```bash
# CI gate (must pass)
cargo clippy --all-targets --all-features 2>&1 | grep "warning:"

# If any "warning:" output => FAIL
```

**Escalation**:
- New warning: Fix or add `#[allow(...)]` with documentation
- Warning in main code path: Always fix (allow only in justified cases)
- Warning in test/example: Fix or feature-gate

---

### Formatting Compliance

**Threshold**: Non-compliant with `cargo fmt` = automatic failure
**Format Tool**: `rustfmt` (via `cargo fmt`)
**Policy**: All code must pass `cargo fmt --check`

**Rationale**:
- Consistent formatting reduces diff noise and improves readability
- Rustfmt is deterministic; formatting is non-negotiable
- CI runs before clippy; formatting must pass first

**Measurement Method**:

```bash
# CI gate (must pass)
cargo fmt --check

# If fails, apply fix
cargo fmt
```

**Escalation**:
- Formatting failure: Automatic fix with `cargo fmt`; blocking failure if not fixed in PR

---

## Regression Detection Workflow (CI/CD Integration)

### Pre-Commit Local Checks

Developers run before pushing:

```bash
# 1. Format
cargo fmt --check

# 2. Lint
cargo clippy --all-targets --all-features

# 3. Test
cargo test --all-targets --all-features

# 4. (Optional) Performance regression check
hyperfine --runs 10 'cargo run --release -p kenken-cli -- solve --n 4 --desc <puzzle>'
# Compare against known baseline
```

### CI Pipeline Checks

Automated checks on every push and PR:

```bash
# Stage 1: Code Quality (fail fast)
cargo fmt --check
cargo clippy --all-targets --all-features

# Stage 2: Correctness (must pass)
cargo test --all-targets
cargo test --all-targets --all-features
cargo test --all-targets --no-default-features

# Stage 3: Performance Regression (warning)
for N in 4 6 8; do
  hyperfine --runs 100 'cargo run --release -p kenken-cli -- solve --n $N --desc <puzzle>'
  # Log result; warn if >5% slower
done

# Stage 4: Memory Regression (warning)
cargo build --release --features dhat-heap
DHAT_OUT_FILE=/tmp/regression_test.out \
  ./target/release/kenken-cli solve --n 6 --desc <puzzle>
dhat_gui /tmp/regression_test.out | tee /tmp/dhat_result.txt
# Check peak; warn if >10% larger than baseline
```

### Regression Report Format

When regression is detected, CI logs:

```
REGRESSION DETECTED
==================================================
Type: Performance (solve_one/4x4)
Metric: Median latency
Baseline: 2.345 ms
New: 2.462 ms
Change: +5.0% (THRESHOLD: >5%)
Status: THRESHOLD_MET - REQUIRES INVESTIGATION

Root Cause Analysis Steps:
1. git log --oneline <commit>..<current> (what changed?)
2. git diff <commit> | head -50 (what code changed?)
3. cargo flamegraph --bin kenken-cli -- solve ... (where is time spent?)
4. Revert change or optimize bottleneck
5. Re-run hyperfine to verify fix
==================================================
```

---

## Baseline Recording Procedure

When establishing new baselines (e.g., after major optimization):

### 1. Document Environment

```
Hardware: Linux x86_64 (CachyOS kernel 6.18.7-2)
CPU: Intel Core i7-8700K (6 cores, AVX2, no AVX-512)
RAM: 32 GB
Build: Release mode (opt-level=3, LTO=thin, codegen-units=1, strip=symbols)
Compiler: rustc nightly-2026-01-29
Date: 2026-01-29
Commit: <git rev-parse HEAD>
```

### 2. Record Solve_One Performance

```bash
for N in 2 3 4 5 6 8 9; do
  PUZZLE=$(cargo run -p kenken-cli -- generate --n $N --seed 12345 --difficulty normal 2>/dev/null | grep "puzzle:" | awk '{print $2}')
  echo "Grid $N:"
  hyperfine --runs 100 "cargo run --release -p kenken-cli -- solve --n $N --desc $PUZZLE"
done > /tmp/baselines_$(date +%Y-%m-%d).txt
```

Update `docs/research/benchmark_baselines_2026-01-29.md` with results.

### 3. Record Memory Usage

```bash
cargo build --release --features dhat-heap
for N in 2 4 6 8 9; do
  PUZZLE=$(cargo run -p kenken-cli -- generate --n $N --seed 12345 --difficulty normal 2>/dev/null | grep "puzzle:" | awk '{print $2}')
  DHAT_OUT_FILE=/tmp/dhat_${N}x${N}.out \
    ./target/release/kenken-cli solve --n $N --desc $PUZZLE
  dhat_gui /tmp/dhat_${N}x${N}.out 2>&1 | grep "Peak" >> /tmp/memory_baselines.txt
done
```

Update `docs/benchmark_memory_2026-01-29.md` with results.

### 4. Commit Baseline Update

```bash
git add docs/research/benchmark_baselines_2026-01-29.md docs/benchmark_memory_2026-01-29.md
git commit -m "docs: record performance baselines after Phase Alpha"
git tag v0.1.0-alpha.1
```

---

## Threshold Adjustment Procedure

**When to Adjust**:
1. Systematic measurement shows consistent variance outside threshold band
2. New algorithmic change intentionally increases baseline (e.g., adding feature)
3. Hardware upgrade changes baseline (document hardware change in baseline file)

**Adjustment Steps**:
1. Measure new baseline (5+ runs, average them)
2. Update `docs/research/benchmark_baselines_2026-01-29.md` with new values
3. Document rationale in commit message
4. Update this file's thresholds if needed
5. Notify team of new baselines via CHANGELOG entry

**Example Adjustment** (hypothetical):

```
OLD: solve_one/6x6 median = 2.3 ms, threshold = 2.415 ms (+5%)
REASON: Phase Alpha PGO/BOLT deployed, baseline improved to 2.1 ms
NEW: solve_one/6x6 median = 2.1 ms, threshold = 2.205 ms (+5%)
RATIONALE: Rebaselining after intentional optimization. New threshold reflects new baseline.
```

---

## Historical Regression Examples

### Example 1: Benign Variance (No Action)

```
Baseline: 4.5 ms
New: 4.58 ms
Change: +1.78%
Status: BELOW_THRESHOLD (threshold: 5%)
Action: No action; variance within expected band
```

### Example 2: Real Regression (Investigation Required)

```
Baseline: 4.5 ms
New: 4.95 ms
Change: +10.0%
Status: EXCEEDS_THRESHOLD (threshold: 5%)
Action:
  1. Identify commits since last baseline
  2. git bisect to find culprit
  3. Run flamegraph to find hot spot
  4. Optimize or revert
  5. Re-baseline and confirm fix
```

### Example 3: Improvement (Opportunity to Optimize)

```
Baseline: 4.5 ms
New: 3.87 ms
Change: -14.0%
Status: IMPROVEMENT
Action:
  1. Investigate what caused improvement
  2. Document optimization
  3. Re-baseline to reflect improvement (prevents false "regressions" later)
  4. Consider this for portfolio of optimizations
```

---

## Related Documents

- [docs/research/benchmark_baselines_2026-01-29.md](research/benchmark_baselines_2026-01-29.md) - Performance baselines
- [docs/benchmark_memory_2026-01-29.md](benchmark_memory_2026-01-29.md) - Memory baselines
- [docs/benchmark_generator_2026-01-29.md](benchmark_generator_2026-01-29.md) - Generator baselines
- [docs/optimization_roadmap.md](optimization_roadmap.md) - Future optimization tiers
- [CONTRIBUTING.md](../CONTRIBUTING.md) - Quality gates for PRs
- [CLAUDE.md](../CLAUDE.md) - Project standards and build commands

---

**Version**: 1.0
**Author**: Claude Code
**Review**: Pending first baseline measurements
