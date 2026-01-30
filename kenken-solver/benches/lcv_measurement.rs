//! LCV (Least Constraining Value) Heuristic Measurement
//!
//! Measures backtracking behavior across puzzle sizes and difficulty tiers
//! to estimate the potential benefit of implementing LCV heuristic.
//!
//! Key metrics:
//! - Solve time per puzzle (baseline performance)
//! - Which puzzles require backtracking vs. pure deduction
//! - Performance scaling with grid size
//!
//! Strategy:
//! - Test on trivial puzzles (all Eq cages) to measure base case
//! - Vary puzzle complexity to see when backtracking dominates
//! - Estimate portfolio impact from observed backtracking frequency

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use kenken_core::{Cage, CellId, Puzzle, rules::Op, rules::Ruleset};
use kenken_solver::solve_one_with_stats;
use smallvec::smallvec;

/// Create trivial puzzle (all Eq cages) for baseline measurement
fn create_trivial_puzzle(n: u8) -> Puzzle {
    let mut cages = Vec::new();
    for cell_id in 0..(n as u16 * n as u16) {
        let val = (cell_id % (n as u16)) + 1;
        cages.push(Cage {
            cells: smallvec![CellId(cell_id)],
            op: Op::Eq,
            target: val as i32,
        });
    }
    Puzzle { n, cages }
}

/// Create mixed puzzle with Add cages (tests backtracking)
fn create_mixed_puzzle(n: u8) -> Puzzle {
    let mut cages = Vec::new();
    let mut cell_id = 0u16;
    let total_cells = n as u16 * n as u16;

    // Create some Add cages (2-3 cells each)
    while cell_id < total_cells {
        let cage_size = if (cell_id + 2) <= total_cells { 2 } else { 1 };
        let mut cage_cells = smallvec![];
        let mut sum = 0u32;

        for _ in 0..cage_size {
            if cell_id < total_cells {
                cage_cells.push(CellId(cell_id));
                sum += ((cell_id % (n as u16)) + 1) as u32;
                cell_id += 1;
            }
        }

        if cage_size == 2 {
            cages.push(Cage {
                cells: cage_cells,
                op: Op::Add,
                target: sum as i32,
            });
        } else {
            let val = sum as i32;
            cages.push(Cage {
                cells: cage_cells,
                op: Op::Eq,
                target: val,
            });
        }
    }

    Puzzle { n, cages }
}

/// Benchmark baseline: Trivial puzzles (all singletons)
fn benchmark_baseline_trivial(c: &mut Criterion) {
    let mut group = c.benchmark_group("lcv_baseline_trivial");
    group.sample_size(50);

    for n in [2, 3, 4, 5, 6].iter() {
        let puzzle = create_trivial_puzzle(*n);
        let rules = Ruleset::keen_baseline();

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}x{}", n, n)),
            n,
            |b, _| b.iter(|| solve_one_with_stats(&puzzle, rules)),
        );
    }

    group.finish();
}

/// Benchmark with mixed cages: More realistic for LCV estimation
fn benchmark_mixed_puzzles(c: &mut Criterion) {
    let mut group = c.benchmark_group("lcv_mixed_cages");
    group.sample_size(20);

    for n in [2, 3, 4, 5, 6].iter() {
        let puzzle = create_mixed_puzzle(*n);
        let rules = Ruleset::keen_baseline();

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}x{}", n, n)),
            n,
            |b, _| b.iter(|| solve_one_with_stats(&puzzle, rules)),
        );
    }

    group.finish();
}

criterion_group!(benches, benchmark_baseline_trivial, benchmark_mixed_puzzles);
criterion_main!(benches);
