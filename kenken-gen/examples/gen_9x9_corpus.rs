//! Generate diverse 9x9 Hard tier puzzles for golden corpus
//!
//! Run with: cargo run --example gen_9x9_corpus --release --features gen-dlx

use kenken_core::format::sgt_desc::encode_keen_desc;
use kenken_core::rules::{Op, Ruleset};
use kenken_gen::{GenerateConfig, generate_with_stats};
use kenken_solver::DeductionTier;

fn main() {
    let rules = Ruleset::keen_baseline();

    println!("=== Generating 9x9 Hard Tier Puzzle Corpus ===\n");

    let mut puzzles = Vec::new();

    // Generate 3 Hard tier 9x9 puzzles
    // Hard tier should require backtracking
    for seed in 0..500u64 {
        let config = GenerateConfig::keen_baseline(9, seed);

        match generate_with_stats(config) {
            Ok(result) => {
                // Accept only Hard tier puzzles
                let is_hard = matches!(result.tier_result.tier_required, Some(DeductionTier::Hard));

                if is_hard && let Ok(desc) = encode_keen_desc(&result.puzzle, rules) {
                    // Count operations for diversity check
                    let mut add = 0;
                    let mut mul = 0;
                    let mut sub = 0;
                    let mut div = 0;
                    for cage in &result.puzzle.cages {
                        match cage.op {
                            Op::Add => add += 1,
                            Op::Mul => mul += 1,
                            Op::Sub => sub += 1,
                            Op::Div => div += 1,
                            Op::Eq => {}
                        }
                    }

                    println!("Found Hard tier 9x9 puzzle (seed {}):", seed);
                    println!(
                        "  Operations: Add={}, Mul={}, Sub={}, Div={}",
                        add, mul, sub, div
                    );
                    print_puzzle_entry(&result, &desc, seed);
                    puzzles.push((seed, desc));

                    if puzzles.len() >= 3 {
                        break;
                    }
                }
            }
            Err(e) => {
                if seed < 5 {
                    eprintln!("DEBUG: Generation failed for seed {}: {:?}", seed, e);
                }
            }
        }

        if (seed + 1) % 50 == 0 {
            eprintln!(
                "Scanned {} seeds (found {} Hard tier puzzles)",
                seed + 1,
                puzzles.len()
            );
        }
    }

    println!("\n=== Summary ===");
    println!("Generated {} Hard tier 9x9 puzzles", puzzles.len());
}

fn print_puzzle_entry(result: &kenken_gen::GeneratedPuzzleWithStats, desc: &str, seed: u64) {
    let grid = result
        .solution
        .iter()
        .map(|&v| format!("{}", v))
        .collect::<Vec<_>>()
        .join(", ");

    let tier_str = match result.tier_result.tier_required {
        Some(DeductionTier::Easy) => "DeductionTier::Easy",
        Some(DeductionTier::Normal) => "DeductionTier::Normal",
        Some(DeductionTier::Hard) => "DeductionTier::Hard",
        Some(DeductionTier::None) => "DeductionTier::None",
        None => "None",
    };

    println!("  GoldenPuzzle {{");
    println!("      n: 9,");
    println!("      desc: \"{}\",", desc);
    println!("      solutions: 1,");
    println!("      difficulty: None,  // Conservative estimate");
    println!("      tier_required: Some({}),", tier_str);
    println!("      solution: Some(&[{}]),", grid);
    println!("      label: \"9x9 Hard tier puzzle (seed {})\",", seed);
    println!("  }},");
    println!();
}
