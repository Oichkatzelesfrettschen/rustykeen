//! Generate diverse 8x8 puzzles for golden corpus
//!
//! Run with: cargo run --example gen_8x8_corpus --release --features gen-dlx

use kenken_core::format::sgt_desc::encode_keen_desc;
use kenken_core::rules::{Op, Ruleset};
use kenken_gen::{GenerateConfig, generate_with_stats};
use kenken_solver::DeductionTier;

fn main() {
    let rules = Ruleset::keen_baseline();

    println!("=== Generating 8x8 Puzzle Corpus ===\n");

    let mut puzzles = Vec::new();

    // Generate 5 diverse 8x8 puzzles with mixed operations
    // Target: Normal difficulty tier (reasonable for 8x8)
    for seed in 0..500u64 {
        let config = GenerateConfig::keen_baseline(8, seed);

        match generate_with_stats(config) {
            Ok(result) => {
                // Count operations
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

                // Accept diverse puzzles: at least 3 different operation types
                let op_types = [add > 0, mul > 0, sub > 0, div > 0]
                    .iter()
                    .filter(|&&x| x)
                    .count();
                let is_diverse = op_types >= 3;

                // Prefer Normal tier, but accept any tier
                if is_diverse && let Ok(desc) = encode_keen_desc(&result.puzzle, rules) {
                    println!("Found diverse 8x8 puzzle (seed {}):", seed);
                    print_puzzle_entry(&result, &desc, seed);
                    puzzles.push((seed, desc));

                    if puzzles.len() >= 5 {
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
                "Scanned {} seeds (found {} puzzles)",
                seed + 1,
                puzzles.len()
            );
        }
    }

    println!("\n=== Summary ===");
    println!("Generated {} diverse 8x8 puzzles", puzzles.len());
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
    println!("      n: 8,");
    println!("      desc: \"{}\",", desc);
    println!("      solutions: 1,");
    println!("      difficulty: None,  // Conservative estimate");
    println!("      tier_required: Some({}),", tier_str);
    println!("      solution: Some(&[{}]),", grid);
    println!("      label: \"8x8 diverse puzzle (seed {})\",", seed);
    println!("  }},");
    println!();
}
