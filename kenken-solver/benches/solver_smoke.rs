//! Benchmark suite for kenken-solver.
//!
//! Covers:
//! - solve_one for various grid sizes (2x2, 3x3, 4x4, 5x5)
//! - count_solutions_up_to for uniqueness verification
//! - Deduction tier comparison (None, Easy, Normal, Hard)
//!
//! # Baseline Recording
//!
//! Results are recorded in `docs/benchmark_baselines.md`.
//! Run with `cargo bench --bench solver_smoke` to update.
//!
//! # Flamegraph Output
//!
//! CPU flamegraphs are generated to target/criterion/*/profile/flamegraph.svg
//! for performance analysis and hotpath identification.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use kenken_core::format::sgt_desc::parse_keen_desc;
use kenken_core::rules::Ruleset;
use kenken_solver::{
    DeductionTier, count_solutions_up_to_with_deductions, solve_one_with_deductions,
};
use pprof::criterion::{Output, PProfProfiler};

/// Puzzles from the golden corpus for benchmarking.
fn benchmark_puzzles() -> Vec<(u8, &'static str, &'static str)> {
    vec![
        // 2x2 singleton
        (2, "_5,a1a2a2a1", "2x2_singleton"),
        // 2x2 add cages
        (2, "b__,a3a3", "2x2_add"),
        // 3x3 singleton
        (3, "_13,a1a2a3a2a3a1a3a1a2", "3x3_singleton"),
        // 3x3 row cages
        (3, "f_6,a6a6a6", "3x3_rows"),
        // 4x4 singleton
        (4, "_25,a1a2a3a4a2a1a4a3a3a4a1a2a4a3a2a1", "4x4_singleton"),
        // 5x5 singleton
        (
            5,
            "_41,a1a2a3a4a5a2a3a4a5a1a3a4a5a1a2a4a5a1a2a3a5a1a2a3a4",
            "5x5_singleton",
        ),
    ]
}

fn bench_solve_one(c: &mut Criterion) {
    let rules = Ruleset::keen_baseline();
    let mut group = c.benchmark_group("solve_one");

    for (n, desc, label) in benchmark_puzzles() {
        if let Ok(puzzle) = parse_keen_desc(n, desc)
            && puzzle.validate(rules).is_ok()
        {
            // Benchmark at Normal tier (most common use case)
            group.bench_with_input(
                BenchmarkId::new(label, "Normal"),
                &DeductionTier::Normal,
                |b, &tier| {
                    b.iter(|| solve_one_with_deductions(black_box(&puzzle), rules, tier));
                },
            );
        }
    }

    group.finish();
}

fn bench_count_solutions(c: &mut Criterion) {
    let rules = Ruleset::keen_baseline();
    let mut group = c.benchmark_group("count_solutions");

    // Simple 2x2 puzzle (known to have 2 solutions)
    let desc_2x2 = "b__,a3a3";
    if let Ok(puzzle) = parse_keen_desc(2, desc_2x2)
        && puzzle.validate(rules).is_ok()
    {
        for limit in [1, 2, 10] {
            group.bench_with_input(
                BenchmarkId::new("2x2", format!("limit_{limit}")),
                &limit,
                |b, &limit| {
                    b.iter(|| {
                        count_solutions_up_to_with_deductions(
                            black_box(&puzzle),
                            rules,
                            DeductionTier::Normal,
                            limit,
                        )
                    });
                },
            );
        }
    }

    group.finish();
}

fn bench_deduction_tiers(c: &mut Criterion) {
    let rules = Ruleset::keen_baseline();
    let mut group = c.benchmark_group("deduction_tiers");

    // Compare deduction tier performance on 2x2
    let desc = "b__,a3a3";
    if let Ok(puzzle) = parse_keen_desc(2, desc)
        && puzzle.validate(rules).is_ok()
    {
        for tier in [
            DeductionTier::None,
            DeductionTier::Easy,
            DeductionTier::Normal,
            DeductionTier::Hard,
        ] {
            group.bench_with_input(
                BenchmarkId::new("count_2x2", format!("{tier:?}")),
                &tier,
                |b, &tier| {
                    b.iter(|| {
                        count_solutions_up_to_with_deductions(black_box(&puzzle), rules, tier, 2)
                    });
                },
            );
        }
    }

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets =
        bench_solve_one,
        bench_count_solutions,
        bench_deduction_tiers
}
criterion_main!(benches);
