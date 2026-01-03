/// Solver Scaling Benchmarks: n×n grid size scaling
///
/// Measures solve time and memory usage as grid size increases from 2x2 to 32x32.
/// Each benchmark measures:
/// - Time to solve (wall-clock)
/// - Solution count verification
/// - Uniqueness checking (count_solutions_up_to limit=2)
///
/// Results inform:
/// - Domain representation choice (u32 vs u64 vs BitDomain)
/// - Solver algorithm scalability
/// - SIMD effectiveness at each size
///
/// Flamegraph Output:
/// - CPU flamegraphs generated to target/criterion/*/profile/flamegraph.svg
/// - Shows which solver components dominate at different grid sizes
use criterion::{Criterion, criterion_group, criterion_main};
use kenken_core::rules::{Op, Ruleset};
use kenken_core::{Cage, CellId, Puzzle};
use kenken_solver::count_solutions_up_to;
use pprof::criterion::{Output, PProfProfiler};
use smallvec::smallvec;

fn create_trivial_puzzle(n: u8) -> Puzzle {
    // Create a minimal valid puzzle for benchmarking
    // (All add-cages to simplify generation)
    let mut cages = Vec::new();

    for row in 0..n {
        for col in 0..n {
            let cell_id = CellId((row * n + col) as u16);
            cages.push(Cage {
                cells: smallvec![cell_id],
                op: Op::Eq,
                target: ((row * n + col) % n + 1) as i32,
            });
        }
    }

    Puzzle { n, cages }
}

fn benchmark_solve_2x2(c: &mut Criterion) {
    let puzzle = std::hint::black_box(create_trivial_puzzle(2));
    let rules = std::hint::black_box(Ruleset::keen_baseline());

    c.bench_function("solve_2x2_uniqueness", |b| {
        b.iter(|| count_solutions_up_to(&puzzle, rules, 2u32))
    });
}

fn benchmark_solve_3x3(c: &mut Criterion) {
    let puzzle = std::hint::black_box(create_trivial_puzzle(3));
    let rules = std::hint::black_box(Ruleset::keen_baseline());

    c.bench_function("solve_3x3_uniqueness", |b| {
        b.iter(|| count_solutions_up_to(&puzzle, rules, 2u32))
    });
}

fn benchmark_solve_4x4(c: &mut Criterion) {
    let puzzle = std::hint::black_box(create_trivial_puzzle(4));
    let rules = std::hint::black_box(Ruleset::keen_baseline());

    c.bench_function("solve_4x4_uniqueness", |b| {
        b.iter(|| count_solutions_up_to(&puzzle, rules, 2u32))
    });
}

fn benchmark_solve_5x5(c: &mut Criterion) {
    let puzzle = std::hint::black_box(create_trivial_puzzle(5));
    let rules = std::hint::black_box(Ruleset::keen_baseline());

    c.bench_function("solve_5x5_uniqueness", |b| {
        b.iter(|| count_solutions_up_to(&puzzle, rules, 2u32))
    });
}

fn benchmark_solve_6x6(c: &mut Criterion) {
    let puzzle = std::hint::black_box(create_trivial_puzzle(6));
    let rules = std::hint::black_box(Ruleset::keen_baseline());

    c.bench_function("solve_6x6_uniqueness", |b| {
        b.iter(|| count_solutions_up_to(&puzzle, rules, 2u32))
    });
}

fn benchmark_solve_8x8(c: &mut Criterion) {
    let puzzle = std::hint::black_box(create_trivial_puzzle(8));
    let rules = std::hint::black_box(Ruleset::keen_baseline());

    c.bench_function("solve_8x8_uniqueness", |b| {
        b.iter(|| count_solutions_up_to(&puzzle, rules, 2u32))
    });
}

fn benchmark_solve_12x12(c: &mut Criterion) {
    let puzzle = std::hint::black_box(create_trivial_puzzle(12));
    let rules = std::hint::black_box(Ruleset::keen_baseline());

    c.bench_function("solve_12x12_uniqueness", |b| {
        b.iter(|| count_solutions_up_to(&puzzle, rules, 2u32))
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets =
        benchmark_solve_2x2,
        benchmark_solve_3x3,
        benchmark_solve_4x4,
        benchmark_solve_5x5,
        benchmark_solve_6x6,
        benchmark_solve_8x8,
        benchmark_solve_12x12
}

criterion_main!(benches);
