//! Generate diverse 6x6 puzzles for golden corpus
//!
//! Run with: cargo run --example gen_6x6_corpus --release

use kenken_core::format::sgt_desc::encode_keen_desc;
use kenken_core::rules::{Op, Ruleset};
use kenken_gen::{GenerateConfig, generate_with_stats};
use kenken_solver::DeductionTier;

fn main() {
    let rules = Ruleset::keen_baseline();

    println!("=== Generating 6x6 Puzzle Corpus ===\n");

    // Track operation distribution
    let mut puzzles = Vec::new();
    let mut add_count = 0;
    let mut mul_count = 0;
    let mut subdiv_count = 0;

    // Generate until we have desired distribution
    for seed in 0..1000u64 {
        let config = GenerateConfig::keen_baseline(6, seed);

        match generate_with_stats(config) {
            Ok(result) => {
                // Analyze operation distribution
                let mut add = 0;
                let mut mul = 0;
                let mut sub = 0;
                let mut div = 0;
                let mut eq = 0;
                for cage in &result.puzzle.cages {
                    match cage.op {
                        Op::Add => add += 1,
                        Op::Mul => mul += 1,
                        Op::Sub => sub += 1,
                        Op::Div => div += 1,
                        Op::Eq => eq += 1,
                    }
                }
                let subdiv = sub + div;

                // Debug: print first puzzle to see what we're getting
                if seed == 0 {
                    eprintln!("DEBUG: First puzzle (seed 0) op distribution:");
                    eprintln!(
                        "  Add: {}, Mul: {}, Sub: {}, Div: {}, Eq: {}",
                        add, mul, sub, div, eq
                    );
                    eprintln!("  Total cages: {}", result.puzzle.cages.len());
                }

                // Classify puzzle by operation distribution (relaxed criteria)
                let is_add_heavy = add >= 3 && add > mul;
                let is_mul_heavy = mul >= 2 && mul > add;
                let is_subdiv_mixed = subdiv >= 2;

                if is_add_heavy
                    && add_count < 5
                    && let Ok(desc) = encode_keen_desc(&result.puzzle, rules)
                {
                    println!("Found Add-heavy 6x6 puzzle (seed {}):", seed);
                    print_puzzle_entry(&result, &desc, seed, "Add-heavy");
                    puzzles.push((seed, desc.clone(), "Add-heavy"));
                    add_count += 1;
                } else if is_mul_heavy
                    && mul_count < 3
                    && let Ok(desc) = encode_keen_desc(&result.puzzle, rules)
                {
                    println!("Found Mul-heavy 6x6 puzzle (seed {}):", seed);
                    print_puzzle_entry(&result, &desc, seed, "Mul-heavy");
                    puzzles.push((seed, desc.clone(), "Mul-heavy"));
                    mul_count += 1;
                } else if is_subdiv_mixed
                    && subdiv_count < 2
                    && let Ok(desc) = encode_keen_desc(&result.puzzle, rules)
                {
                    println!("Found Sub/Div-mixed 6x6 puzzle (seed {}):", seed);
                    print_puzzle_entry(&result, &desc, seed, "Sub/Div-mixed");
                    puzzles.push((seed, desc.clone(), "Sub/Div-mixed"));
                    subdiv_count += 1;
                }

                if add_count >= 5 && mul_count >= 3 && subdiv_count >= 2 {
                    break;
                }
            }
            Err(e) => {
                if seed < 10 {
                    eprintln!("DEBUG: Generation failed for seed {}: {:?}", seed, e);
                }
            }
        }

        if (seed + 1) % 100 == 0 {
            eprintln!(
                "Scanned {} seeds (Add: {}, Mul: {}, Sub/Div: {})",
                seed + 1,
                add_count,
                mul_count,
                subdiv_count
            );
        }
    }

    println!("\n=== Summary ===");
    println!("Generated {} Add-heavy puzzles", add_count);
    println!("Generated {} Mul-heavy puzzles", mul_count);
    println!("Generated {} Sub/Div-mixed puzzles", subdiv_count);
    println!("Total: {} puzzles", puzzles.len());
}

fn print_puzzle_entry(
    result: &kenken_gen::GeneratedPuzzleWithStats,
    desc: &str,
    seed: u64,
    label: &str,
) {
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
    println!("      n: 6,");
    println!("      desc: \"{}\",", desc);
    println!("      solutions: 1,");
    println!("      difficulty: None,  // Conservative estimate");
    println!("      tier_required: Some({}),", tier_str);
    println!("      solution: Some(&[{}]),", grid);
    println!("      label: \"6x6 {} puzzle (seed {})\",", label, seed);
    println!("  }},");
    println!();
}
