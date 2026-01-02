/// CPU Flamegraph Profiling Binary
///
/// Generates detailed CPU flamegraphs via perf/pprof for performance analysis.
///
/// Usage:
///   cargo build --release --bench profile_flames
///   cargo flamegraph --release --bench profile_flames -o /tmp/solver_flame.svg
///
/// Then open /tmp/solver_flame.svg in a browser to analyze where time is spent.

use std::hint::black_box;
use kenken_core::format::sgt_desc::parse_keen_desc;
use kenken_core::rules::Ruleset;
use kenken_solver::{DeductionTier, solve_one_with_deductions};

fn main() {
    let rules = Ruleset::keen_baseline();

    // Representative puzzles for flamegraph profiling
    // Using valid SGT format: comma-separated rows, letters = cage IDs, numbers = targets
    let puzzles = vec![
        // 2x2 - baseline, minimal overhead (valid SGT: "b__,a3a3")
        (2, "b__,a3a3", "2x2_simple", 2000),

        // 3x3 - need valid SGT format (3 rows of 3 cells each with letters)
        (3, "aab,cdb,eeb", "3x3_simple", 1000),

        // 4x4 - all singleton cages (easiest case)
        (4, "abcd,efgh,ijkl,mnop", "4x4_trivial", 500),

        // 5x5 - all singleton cages
        (5, "abcde,fghij,klmno,pqrst,uvwxy", "5x5_trivial", 200),
    ];

    for (n, desc, label, iterations) in puzzles {
        if let Ok(puzzle) = parse_keen_desc(n, desc) {
            if puzzle.validate(rules).is_ok() {
                eprintln!("\nProfiler: {} ({} iterations)", label, iterations);

                // Warm up (establish steady state)
                for _ in 0..10 {
                    let p = puzzle.clone();
                    let _ = solve_one_with_deductions(&p, rules, DeductionTier::Normal);
                }

                // Profile hot loop
                for i in 0..iterations {
                    let p = puzzle.clone();
                    let _ = solve_one_with_deductions(
                        black_box(&p),
                        rules,
                        black_box(DeductionTier::Normal),
                    );

                    if iterations > 100 && (i + 1) % (iterations / 5) == 0 {
                        eprintln!("  {} / {} iterations", i + 1, iterations);
                    }
                }

                eprintln!("  Profiling complete for {}", label);
            } else {
                eprintln!("\nProfiler: {} - validation failed", label);
            }
        } else {
            eprintln!("\nProfiler: {} - parse failed (invalid SGT format)", label);
        }
    }

    eprintln!("\nProfiler finished. Use: cargo flamegraph --release --bench profile_flames -o /tmp/solver_flame.svg");
}
