use kenken_core::format::sgt_desc::to_keen_desc;
use kenken_gen::generate_with_stats;
use kenken_gen::GenerateConfig;
use kenken_solver::{DifficultyTier, DeductionTier};

fn main() {
    // Generate 3x3 Normal-tier puzzles
    println!("=== Generating 3x3 Normal-tier puzzles ===\n");
    let mut count_3x3 = 0;
    for seed in 0..100u64 {
        let config = GenerateConfig::with_difficulty(3, seed, DifficultyTier::Normal);

        match generate_with_stats(config) {
            Ok(result) => {
                if let Some(tier) = result.tier_required {
                    if tier == DeductionTier::Normal {
                        if let Ok(desc) = to_keen_desc(&result.puzzle) {
                            let grid = result.solution.iter()
                                .map(|&v| format!("{}", v))
                                .collect::<Vec<_>>()
                                .join(", ");

                            println!("Found 3x3 Normal-tier puzzle (seed {}):", seed);
                            println!("  GoldenPuzzle {{");
                            println!("      n: 3,");
                            println!("      desc: \"{}\",", desc);
                            println!("      solutions: 1,");
                            println!("      difficulty: Some(DifficultyTier::Normal),");
                            println!("      tier_required: Some(DeductionTier::Normal),");
                            println!("      solution: Some(&[{}]),", grid);
                            println!("      label: \"3x3 Normal-tier puzzle (seed {})\",", seed);
                            println!("  }},");
                            println!();

                            count_3x3 += 1;
                            if count_3x3 >= 2 {
                                break;
                            }
                        }
                    }
                }
            }
            Err(_) => {
                // Silently continue on generation failures
            }
        }
    }

    // Generate 4x4 Normal-tier puzzles
    println!("\n=== Generating 4x4 Normal-tier puzzles ===\n");
    let mut count_4x4 = 0;
    for seed in 0..150u64 {
        let config = GenerateConfig::with_difficulty(4, seed, DifficultyTier::Normal);

        match generate_with_stats(config) {
            Ok(result) => {
                if let Some(tier) = result.tier_required {
                    if tier == DeductionTier::Normal {
                        if let Ok(desc) = to_keen_desc(&result.puzzle) {
                            let grid = result.solution.iter()
                                .map(|&v| format!("{}", v))
                                .collect::<Vec<_>>()
                                .join(", ");

                            println!("Found 4x4 Normal-tier puzzle (seed {}):", seed);
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

                            count_4x4 += 1;
                            if count_4x4 >= 2 {
                                break;
                            }
                        }
                    }
                }
            }
            Err(_) => {
                // Silently continue on generation failures
            }
        }
    }

    println!("\nGeneration complete. Found {} 3x3 and {} 4x4 Normal-tier puzzles.", count_3x3, count_4x4);
}
