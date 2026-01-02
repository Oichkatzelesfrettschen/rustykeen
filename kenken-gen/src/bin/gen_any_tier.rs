use kenken_core::format::sgt_desc::encode_keen_desc;
use kenken_core::rules::Ruleset;
use kenken_gen::GenerateConfig;
use kenken_gen::generate_with_stats;
use kenken_solver::DeductionTier;

fn main() {
    let rules = Ruleset::keen_baseline();

    println!("=== Generating 4x4 puzzles with tier analysis ===\n");

    let mut hard_tier_found = false;
    let mut normal_tier_found = false;

    // Try to find puzzles of various tiers
    for seed in 0..1000u64 {
        let config = GenerateConfig::keen_baseline(4, seed);

        if let Ok(result) = generate_with_stats(config)
            && let Some(tier) = result.tier_result.tier_required
        {
            if tier == DeductionTier::Hard
                && !hard_tier_found
                && let Ok(desc) = encode_keen_desc(&result.puzzle, rules)
            {
                let grid = result
                    .solution
                    .iter()
                    .map(|&v| format!("{}", v))
                    .collect::<Vec<_>>()
                    .join(", ");

                println!("Found 4x4 HARD-tier puzzle (seed {}):", seed);
                println!("  GoldenPuzzle {{");
                println!("      n: 4,");
                println!("      desc: \"{}\",", desc);
                println!("      solutions: 1,");
                println!("      difficulty: Some(DifficultyTier::Hard),");
                println!("      tier_required: Some(DeductionTier::Hard),");
                println!("      solution: Some(&[{}]),", grid);
                println!("      label: \"4x4 Hard-tier puzzle (seed {})\",", seed);
                println!("  }},");
                println!();
                hard_tier_found = true;
            }

            if tier == DeductionTier::Normal
                && !normal_tier_found
                && let Ok(desc) = encode_keen_desc(&result.puzzle, rules)
            {
                let grid = result
                    .solution
                    .iter()
                    .map(|&v| format!("{}", v))
                    .collect::<Vec<_>>()
                    .join(", ");

                println!("Found 4x4 NORMAL-tier puzzle (seed {}):", seed);
                println!("  GoldenPuzzle {{");
                println!("      n: 4,");
                println!("      desc: \"{}\",", desc);
                println!("      solutions: 1,");
                println!("      difficulty: Some(DifficultyTier::Normal),");
                println!("      tier_required: Some(DeductionTier::Normal),");
                println!("      solution: Some(&[{}]),", grid);
                println!("      label: \"4x4 Normal-tier puzzle (seed {})\",", seed);
                println!("  }},");
                println!();
                normal_tier_found = true;
            }

            if hard_tier_found && normal_tier_found {
                break;
            }
        }

        if (seed + 1) % 100 == 0 {
            eprintln!("Scanned {} seeds...", seed + 1);
        }
    }

    println!("\n=== Summary ===");
    println!("Hard-tier puzzle found: {}", hard_tier_found);
    println!("Normal-tier puzzle found: {}", normal_tier_found);
}
