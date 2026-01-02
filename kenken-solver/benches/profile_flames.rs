/// CPU Flamegraph Profiling Binary
///
/// Generates detailed CPU flamegraphs via perf/pprof for performance analysis.
///
/// Usage:
///   cargo build --release --bench profile_flames
///   ./scripts/profile_solver.sh
///
/// Or manually:
///   cargo flamegraph --release --bench profile_flames -o /tmp/solver_flame_2x2.svg
///   cargo flamegraph --release --bench profile_flames -o /tmp/solver_flame_6x6.svg

use std::hint::black_box;
use kenken_core::format::sgt_desc::parse_keen_desc;
use kenken_core::rules::Ruleset;
use kenken_solver::{DeductionTier, solve_one_with_deductions};

fn main() {
    let rules = Ruleset::keen_baseline();

    // Representative puzzles for flamegraph profiling
    let puzzles = vec![
        // 2x2 - baseline, minimal overhead
        (2, "b__,a3a3", "2x2_add", 1000),

        // 3x3 - small multi-cell cages
        (3, "f_6,a6a6a6", "3x3_rows", 500),

        // 4x4 - moderate complexity
        (4, "a__b,a5bc,d5ec,dee", "4x4_mixed", 200),

        // 6x6 - real puzzle, significant enumeration
        (6, "a___bc,a_def,gdhij,gkhij,kllm,nompm", "6x6_standard", 100),
    ];

    for (n, desc, label, iterations) in puzzles {
        if let Ok(puzzle) = parse_keen_desc(n, desc) {
            if puzzle.validate(rules).is_ok() {
                eprintln!("\nProfiler: {} ({} iterations)", label, iterations);

                // Warm up (establish steady state)
                for _ in 0..10 {
                    let mut p = puzzle.clone();
                    let _ = solve_one_with_deductions(&p, rules, DeductionTier::Normal);
                }

                // Profile hot loop
                for i in 0..iterations {
                    let mut p = puzzle.clone();
                    let _ = solve_one_with_deductions(
                        black_box(&p),
                        rules,
                        black_box(DeductionTier::Normal),
                    );

                    if (i + 1) % (iterations / 5) == 0 {
                        eprintln!("  {} / {} iterations", i + 1, iterations);
                    }
                }

                eprintln!("  Profiling complete for {}", label);
            }
        }
    }

    eprintln!("\nProfiler finished. Use flamegraph to visualize results.");
}
