//! Span-level profiling binary for KenKen solver
//!
//! Uses tracing-flame to capture and visualize solver operations:
//! - kenken.solve_one - Top-level solve operation
//! - kenken.propagate - Constraint propagation
//! - kenken.search.branch - Search tree branching
//! - kenken.search.backtrack - Backtracking operations
//!
//! # Usage
//!
//! ```sh
//! cargo run --bin profile_spans --release -- --n 6 --desc <puzzle> --tier normal
//! ```
//!
//! This generates `target/tracing_flame.html` showing the temporal flow of solver spans.

use kenken_core::format::sgt_desc::parse_keen_desc;
use kenken_core::rules::Ruleset;
use kenken_solver::DeductionTier;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

fn main() {
    // Parse command-line arguments
    let args: Vec<String> = std::env::args().collect();

    let mut n = 6u8;
    let mut desc = String::new();
    let mut tier = DeductionTier::Normal;
    let mut output = "target/tracing_flame.html".to_string();

    // Simple argument parsing
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--n" if i + 1 < args.len() => {
                n = args[i + 1].parse().unwrap_or(6);
                i += 2;
            }
            "--desc" if i + 1 < args.len() => {
                desc = args[i + 1].clone();
                i += 2;
            }
            "--tier" if i + 1 < args.len() => {
                tier = match args[i + 1].as_str() {
                    "none" => DeductionTier::None,
                    "easy" => DeductionTier::Easy,
                    "normal" => DeductionTier::Normal,
                    "hard" => DeductionTier::Hard,
                    _ => DeductionTier::Normal,
                };
                i += 2;
            }
            "--output" if i + 1 < args.len() => {
                output = args[i + 1].clone();
                i += 2;
            }
            _ => i += 1,
        }
    }

    // Set up tracing with flame layer for span output
    let (flame_layer, _guard) =
        tracing_flame::FlameLayer::with_file(&output).expect("Failed to create flame layer");

    tracing_subscriber::registry().with(flame_layer).init();

    // Log solver invocation
    info!(
        n = n,
        desc = %desc,
        tier = ?tier,
        "Starting KenKen solver profiling"
    );

    // Parse and validate puzzle
    let puzzle = match parse_keen_desc(n, &desc) {
        Ok(p) => {
            info!("Puzzle parsed successfully");
            p
        }
        Err(e) => {
            eprintln!("Error parsing puzzle: {}", e);
            std::process::exit(1);
        }
    };

    let rules = Ruleset::keen_baseline();
    if let Err(e) = puzzle.validate(rules) {
        eprintln!("Invalid puzzle: {}", e);
        std::process::exit(1);
    }

    info!("Puzzle validation passed");

    // Solve with tracing enabled
    match kenken_solver::solve_one_with_deductions(&puzzle, rules, tier) {
        Ok(Some(solution)) => {
            info!("Puzzle solved successfully");
            println!("Solution found:");
            for (i, &digit) in solution.grid.iter().enumerate() {
                if i > 0 && i % (n as usize) == 0 {
                    println!();
                }
                print!("{} ", digit);
            }
            println!();
        }
        Ok(None) => {
            info!("No solution exists for this puzzle");
            println!("No solution found");
        }
        Err(e) => {
            info!("Solver error: {}", e);
            eprintln!("Solver error: {}", e);
            std::process::exit(1);
        }
    }

    // The flame guard is dropped here, finalizing the output
    println!("\nTracing flame graph written to: {}", output);
    println!(
        "Convert to SVG with: inferno {} > tracing_flame.svg",
        output
    );
}
