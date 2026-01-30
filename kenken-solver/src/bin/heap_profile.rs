use kenken_core::format::sgt_desc::parse_keen_desc;
use kenken_core::rules::Ruleset;
use kenken_solver::{DeductionTier, solve_one_with_deductions};
/// Heap allocation profiling for KenKen solver
///
/// Uses dhat-rs for detailed heap profiling to identify allocation patterns
/// and memory bottlenecks.
///
/// Requires: `dhat` crate (profiling feature)
///
/// Usage:
///   cargo build --release --bin heap_profile --features "dhat-heap"
///   ./target/release/heap_profile --n 3 --desc "f_6,a6a6a6" --tier normal
use std::env;

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    let args: Vec<String> = env::args().collect();
    let mut n = 2u8;
    let mut desc = "b__,a3a3".to_string();
    let mut tier_str = "normal".to_string();
    let mut iterations = 100;

    // Parse arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--n" => {
                i += 1;
                if i < args.len() {
                    n = args[i].parse().unwrap_or(2);
                }
            }
            "--desc" => {
                i += 1;
                if i < args.len() {
                    desc = args[i].clone();
                }
            }
            "--tier" => {
                i += 1;
                if i < args.len() {
                    tier_str = args[i].clone();
                }
            }
            "--iterations" => {
                i += 1;
                if i < args.len() {
                    iterations = args[i].parse().unwrap_or(100);
                }
            }
            _ => {}
        }
        i += 1;
    }

    let tier = match tier_str.to_lowercase().as_str() {
        "none" => DeductionTier::None,
        "easy" => DeductionTier::Easy,
        "normal" => DeductionTier::Normal,
        "hard" => DeductionTier::Hard,
        _ => DeductionTier::Normal,
    };

    let rules = Ruleset::keen_baseline();

    eprintln!(
        "Heap Profiler: {}x{} puzzle, tier={:?}, {} iterations",
        n, n, tier, iterations
    );

    match parse_keen_desc(n, &desc) {
        Ok(puzzle) => {
            if let Err(e) = puzzle.validate(rules) {
                eprintln!("Puzzle validation failed: {}", e);
                std::process::exit(1);
            }

            eprintln!("Puzzle loaded: {} cages", puzzle.cages.len());

            // Warm up
            for _ in 0..10 {
                let p = puzzle.clone();
                let _ = solve_one_with_deductions(&p, rules, tier);
            }

            eprintln!("Warmup complete, starting allocation profiling...");

            // Profile hot loop
            for i in 0..iterations {
                let p = puzzle.clone();
                let _ = solve_one_with_deductions(&p, rules, tier);

                if (i + 1) % (iterations.max(10) / 10) == 0 {
                    eprintln!("  {} / {} iterations", i + 1, iterations);
                }
            }

            eprintln!(
                "Heap profiling complete: {} iterations of {}x{}",
                iterations, n, n
            );
            #[cfg(feature = "dhat-heap")]
            eprintln!("Heap profile saved to dhat-heap.json");
        }
        Err(e) => {
            eprintln!("Failed to parse puzzle: {}", e);
            std::process::exit(1);
        }
    }
}
