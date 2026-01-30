//! Benchmark SAT vs backtracking for Add/Mul cage puzzles
//!
//! Run with: cargo run --example benchmark_sat_vs_backtrack --release --features sat-varisat

#[cfg(feature = "sat-varisat")]
use kenken_core::format::sgt_desc::parse_keen_desc;

#[cfg(feature = "sat-varisat")]
use kenken_core::rules::Ruleset;
#[cfg(feature = "sat-varisat")]
use kenken_solver::{DeductionTier, count_solutions_up_to_with_deductions};

#[cfg(feature = "sat-varisat")]
use kenken_solver::sat_cages::puzzle_uniqueness_via_sat;
#[cfg(feature = "sat-varisat")]
use kenken_solver::sat_latin::SatUniqueness;

#[cfg(feature = "sat-varisat")]
use std::time::Instant;

fn main() {
    #[cfg(not(feature = "sat-varisat"))]
    {
        eprintln!("ERROR: This benchmark requires the 'sat-varisat' feature.");
        eprintln!(
            "Run with: cargo run --example benchmark_sat_vs_backtrack --release --features sat-varisat"
        );
        std::process::exit(1);
    }

    #[cfg(feature = "sat-varisat")]
    {
        let rules = Ruleset::keen_baseline();

        println!("=== SAT vs Backtracking Benchmark ===\n");
        println!("Comparing uniqueness checking performance:\n");

        // Test cases with increasing complexity
        let test_cases = vec![
            // Simple 3x3 with Add cage
            ("3x3 Add-2cell", 3, "aa_b__b__,m6a3a3a3a3a3a3a3"),
            // 4x4 with mix of ops (from existing test)
            ("4x4 Mixed", 4, "_25,a1a2a3a4a2a3a4a1a3a4a1a2a4a1a2a3"),
            // 6x6 Add-heavy from golden corpus
            (
                "6x6 Add-heavy",
                6,
                "_a_aa__aa__a3_aabaa_9aa_4a_3ab,s3a7m18s1s5d4a7a12a10a7s1m72d3m10s4a9",
            ),
            // 6x6 Mul-heavy from golden corpus
            (
                "6x6 Mul-heavy",
                6,
                "aa_3a__ba__a__a__a_abb_a_3a_4b__b_a_,m15m24m6m12m24d5m24s4m20a15a7a7a5a11d3",
            ),
        ];

        for (label, n, desc) in test_cases {
            let puzzle = match parse_keen_desc(n, desc) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Failed to parse {}: {:?}", label, e);
                    continue;
                }
            };

            if puzzle.validate(rules).is_err() {
                eprintln!("Invalid puzzle: {}", label);
                continue;
            }

            println!("Test: {}", label);
            println!("  Grid size: {}x{}", n, n);
            println!("  Cages: {}", puzzle.cages.len());

            // Benchmark backtracking solver
            let start = Instant::now();
            let native_result =
                count_solutions_up_to_with_deductions(&puzzle, rules, DeductionTier::Hard, 2);
            let native_time = start.elapsed();

            let native_count = match native_result {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("  Backtracking error: {:?}", e);
                    continue;
                }
            };

            // Benchmark SAT solver
            let start = Instant::now();
            let sat_result = puzzle_uniqueness_via_sat(&puzzle, rules);
            let sat_time = start.elapsed();

            // Verify results match
            let sat_count = match sat_result {
                SatUniqueness::Unique => 1,
                SatUniqueness::Multiple => 2,
                SatUniqueness::Unsat => 0,
            };

            let results_match =
                (native_count == sat_count) || (native_count >= 2 && sat_count == 2);

            println!(
                "  Backtracking: {:?} (count: {})",
                native_time, native_count
            );
            println!("  SAT:          {:?} (count: {})", sat_time, sat_count);

            if results_match {
                let speedup = native_time.as_nanos() as f64 / sat_time.as_nanos() as f64;
                if speedup > 1.0 {
                    println!("  → SAT is {:.2}x faster", speedup);
                } else {
                    println!("  → Backtracking is {:.2}x faster", 1.0 / speedup);
                }
            } else {
                println!("  ⚠ Results mismatch!");
            }

            println!();
        }

        println!("=== Benchmark Complete ===");
    }
}
