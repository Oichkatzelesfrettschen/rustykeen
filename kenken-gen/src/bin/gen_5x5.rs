use kenken_core::format::sgt_desc::encode_keen_desc;
use kenken_core::rules::Ruleset;
use kenken_gen::generate_with_stats;
use kenken_gen::GenerateConfig;

fn main() {
    let rules = Ruleset::keen_baseline();

    println!("=== Generating 5x5 puzzles ===\n");

    let mut count = 0;
    let target = 3; // Add 3 more 5x5 puzzles

    for seed in 0..2000u64 {
        let config = GenerateConfig::keen_baseline(5, seed);

        if let Ok(result) = generate_with_stats(config) {
            if let Ok(desc) = encode_keen_desc(&result.puzzle, rules) {
                let grid = result.solution.iter()
                    .map(|&v| format!("{}", v))
                    .collect::<Vec<_>>()
                    .join(", ");

                println!("Found 5x5 puzzle (seed {}, tier: {:?}):", seed, result.tier_result.tier_required);
                println!("  GoldenPuzzle {{");
                println!("      n: 5,");
                println!("      desc: \"{}\",", desc);
                println!("      solutions: 1,");
                if let Some(diff) = result.tier_result.tier_required {
                    println!("      difficulty: Some(DifficultyTier::Easy),  // Conservative estimate");
                    println!("      tier_required: Some(DeductionTier::{:?}),", diff);
                } else {
                    println!("      difficulty: None,");
                    println!("      tier_required: None,  // Requires guessing");
                }
                println!("      solution: Some(&[{}]),", grid);
                println!("      label: \"5x5 puzzle (seed {})\",", seed);
                println!("  }},");
                println!();

                count += 1;
                if count >= target {
                    break;
                }
            }
        }

        if (seed + 1) % 200 == 0 {
            eprintln!("Scanned {} seeds...", seed + 1);
        }
    }

    println!("Found {} 5x5 puzzles", count);
}
