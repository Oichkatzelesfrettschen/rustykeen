use kenken_core::format::sgt_desc::encode_keen_desc;
use kenken_core::rules::Ruleset;
use kenken_gen::GenerateConfig;
use kenken_gen::generate_with_stats;
use kenken_solver::DeductionTier;

fn main() {
    let rules = Ruleset::keen_baseline();

    println!("=== Generating diverse Easy-tier 3x3 puzzles ===\n");

    let mut easy_count = 0;
    let target_count = 2; // Add 2 new 3x3 Easy puzzles

    for seed in 0..500u64 {
        let config = GenerateConfig::keen_baseline(3, seed);

        if let Ok(result) = generate_with_stats(config)
            && let Some(tier) = result.tier_result.tier_required
            && tier == DeductionTier::Easy
            && let Ok(desc) = encode_keen_desc(&result.puzzle, rules)
        {
            let grid = result
                .solution
                .iter()
                .map(|&v| format!("{}", v))
                .collect::<Vec<_>>()
                .join(", ");

            println!("Found 3x3 Easy-tier puzzle (seed {}):", seed);
            println!("  GoldenPuzzle {{");
            println!("      n: 3,");
            println!("      desc: \"{}\",", desc);
            println!("      solutions: 1,");
            println!("      difficulty: Some(DifficultyTier::Easy),");
            println!("      tier_required: Some(DeductionTier::Easy),");
            println!("      solution: Some(&[{}]),", grid);
            println!(
                "      label: \"3x3 Easy-tier puzzle with cages (seed {})\",",
                seed
            );
            println!("  }},");
            println!();

            easy_count += 1;
            if easy_count >= target_count {
                break;
            }
        }

        if (seed + 1) % 100 == 0 {
            eprintln!("Scanned {} seeds for 3x3...", seed + 1);
        }
    }

    println!("\n=== Generating diverse Easy-tier 4x4 puzzles ===\n");

    easy_count = 0;
    let target_count_4x4 = 2; // Add 2 new 4x4 Easy puzzles

    for seed in 0..500u64 {
        let config = GenerateConfig::keen_baseline(4, seed);

        if let Ok(result) = generate_with_stats(config)
            && let Some(tier) = result.tier_result.tier_required
            && tier == DeductionTier::Easy
            && let Ok(desc) = encode_keen_desc(&result.puzzle, rules)
        {
            let grid = result
                .solution
                .iter()
                .map(|&v| format!("{}", v))
                .collect::<Vec<_>>()
                .join(", ");

            println!("Found 4x4 Easy-tier puzzle (seed {}):", seed);
            println!("  GoldenPuzzle {{");
            println!("      n: 4,");
            println!("      desc: \"{}\",", desc);
            println!("      solutions: 1,");
            println!("      difficulty: Some(DifficultyTier::Easy),");
            println!("      tier_required: Some(DeductionTier::Easy),");
            println!("      solution: Some(&[{}]),", grid);
            println!(
                "      label: \"4x4 Easy-tier puzzle with cages (seed {})\",",
                seed
            );
            println!("  }},");
            println!();

            easy_count += 1;
            if easy_count >= target_count_4x4 {
                break;
            }
        }

        if (seed + 1) % 100 == 0 {
            eprintln!("Scanned {} seeds for 4x4...", seed + 1);
        }
    }
}
