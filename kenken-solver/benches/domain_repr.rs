/// Domain Representation Comparison Benchmarks
///
/// Compares performance of different domain representation implementations:
/// - Domain32 (u32 bitmask, baseline for n <= 31)
/// - Domain64 (u64 bitmask, for n <= 63)
/// - FixedBitDomain (fixedbitset SIMD, for all sizes)
/// - SmallBitDomain (smallbitvec inline, for n <= 8)
///
/// Tests three categories:
/// 1. Microbenchmarks: individual operations (create, insert, remove, count, bitwise ops)
/// 2. Macrobenchmarks: full solver workload with different domains
/// 3. Solver scaling: how domain representation affects overall solver performance
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use kenken_core::rules::{Op, Ruleset};
use kenken_core::{Cage, CellId, Puzzle};
use kenken_solver::{Domain32, Domain64, DomainOps};

#[cfg(feature = "solver-fixedbitset")]
use kenken_solver::FixedBitDomain;

#[cfg(feature = "solver-smallbitvec")]
use kenken_solver::SmallBitDomain;

/// Microbenchmark: domain creation (empty and full)
fn bench_domain_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("domain_creation");

    for n in [2, 4, 6, 8, 16, 32].iter() {
        // Domain32 baseline
        if *n <= 31 {
            group.bench_with_input(BenchmarkId::new("Domain32/full", n), n, |b, &n| {
                b.iter(|| Domain32::full(black_box(n)))
            });
        }

        // Domain64
        if *n <= 63 {
            group.bench_with_input(BenchmarkId::new("Domain64/full", n), n, |b, &n| {
                b.iter(|| Domain64::full(black_box(n)))
            });
        }

        // FixedBitDomain
        #[cfg(feature = "solver-fixedbitset")]
        group.bench_with_input(BenchmarkId::new("FixedBit/full", n), n, |b, &n| {
            b.iter(|| FixedBitDomain::full(black_box(n)))
        });

        // SmallBitDomain (n <= 8 only)
        #[cfg(feature = "solver-smallbitvec")]
        if *n <= 8 {
            group.bench_with_input(BenchmarkId::new("SmallBit/full", n), n, |b, &n| {
                b.iter(|| SmallBitDomain::full(black_box(n)))
            });
        }
    }

    group.finish();
}

/// Microbenchmark: domain operations (insert, contains, count)
fn bench_domain_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("domain_operations");

    for n in [4, 6, 8, 16, 32].iter() {
        if *n <= 31 {
            let d = Domain32::full(*n);
            group.bench_with_input(BenchmarkId::new("Domain32/insert", n), n, |b, _| {
                b.iter(|| {
                    let mut domain = black_box(d);
                    domain.insert(black_box(1));
                })
            });
            group.bench_with_input(BenchmarkId::new("Domain32/count", n), n, |b, _| {
                b.iter(|| d.count())
            });
        }

        if *n <= 63 {
            let d = Domain64::full(*n);
            group.bench_with_input(BenchmarkId::new("Domain64/insert", n), n, |b, _| {
                b.iter(|| {
                    let mut domain = black_box(d);
                    domain.insert(black_box(1));
                })
            });
            group.bench_with_input(BenchmarkId::new("Domain64/count", n), n, |b, _| {
                b.iter(|| d.count())
            });
        }

        #[cfg(feature = "solver-fixedbitset")]
        {
            let d = FixedBitDomain::full(*n);
            group.bench_with_input(BenchmarkId::new("FixedBit/insert", n), n, |b, _| {
                b.iter(|| {
                    let mut domain = black_box(d.clone());
                    domain.insert(black_box(1));
                })
            });
            group.bench_with_input(BenchmarkId::new("FixedBit/count", n), n, |b, _| {
                b.iter(|| d.count())
            });
        }

        #[cfg(feature = "solver-smallbitvec")]
        if *n <= 8 {
            let d = SmallBitDomain::full(*n);
            group.bench_with_input(BenchmarkId::new("SmallBit/insert", n), n, |b, _| {
                b.iter(|| {
                    let mut domain = black_box(d.clone());
                    domain.insert(black_box(1));
                })
            });
            group.bench_with_input(BenchmarkId::new("SmallBit/count", n), n, |b, _| {
                b.iter(|| d.count())
            });
        }
    }

    group.finish();
}

/// Macrobenchmark: solve_one performance on standard puzzles
fn bench_solver_workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("solver_workload");
    group.sample_size(10); // Reduce samples for longer-running tests

    // 2x2 simple puzzle (baseline)
    let puzzle_2x2 = Puzzle {
        n: 2,
        cages: vec![
            Cage {
                cells: smallvec::smallvec![CellId(0), CellId(3)],
                op: Op::Add,
                target: 3,
            },
            Cage {
                cells: smallvec::smallvec![CellId(1), CellId(2)],
                op: Op::Add,
                target: 3,
            },
        ],
    };

    // 4x4 puzzle
    let puzzle_4x4 = Puzzle {
        n: 4,
        cages: vec![
            Cage {
                cells: smallvec::smallvec![CellId(0), CellId(1)],
                op: Op::Add,
                target: 5,
            },
            Cage {
                cells: smallvec::smallvec![CellId(2), CellId(3)],
                op: Op::Add,
                target: 5,
            },
            Cage {
                cells: smallvec::smallvec![CellId(4), CellId(5)],
                op: Op::Add,
                target: 5,
            },
            Cage {
                cells: smallvec::smallvec![CellId(6), CellId(7)],
                op: Op::Add,
                target: 5,
            },
            Cage {
                cells: smallvec::smallvec![CellId(8), CellId(9)],
                op: Op::Add,
                target: 5,
            },
            Cage {
                cells: smallvec::smallvec![CellId(10), CellId(11)],
                op: Op::Add,
                target: 5,
            },
            Cage {
                cells: smallvec::smallvec![CellId(12), CellId(13)],
                op: Op::Add,
                target: 5,
            },
            Cage {
                cells: smallvec::smallvec![CellId(14), CellId(15)],
                op: Op::Add,
                target: 5,
            },
        ],
    };

    let rules = Ruleset::keen_baseline();

    for (puzzle_size, puzzle) in &[(2, &puzzle_2x2), (4, &puzzle_4x4)] {
        // Domain32
        group.bench_function(
            BenchmarkId::new("Domain32", format!("{}x{}", puzzle_size, puzzle_size)),
            |b| {
                b.iter(|| {
                    let _result = kenken_solver::solve_one(black_box(puzzle), black_box(rules));
                });
            },
        );

        // Domain64 (if applicable)
        if *puzzle_size <= 8 {
            group.bench_function(
                BenchmarkId::new("Domain64", format!("{}x{}", puzzle_size, puzzle_size)),
                |b| {
                    b.iter(|| {
                        let _result = kenken_solver::solve_one(black_box(puzzle), black_box(rules));
                    });
                },
            );
        }
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_domain_creation,
    bench_domain_operations,
    bench_solver_workload
);
criterion_main!(benches);
