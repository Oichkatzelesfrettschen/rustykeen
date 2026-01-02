#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

use kenken_core::Puzzle;
use kenken_core::rules::Ruleset;
use kenken_solver::error::SolveError;
use kenken_solver::{DeductionTier, count_solutions_up_to_with_deductions};

pub mod generator;
pub mod minimizer;
pub mod seed;

pub use generator::{
    GenerateConfig, GeneratedPuzzle, GeneratedPuzzleWithStats, generate, generate_with_stats,
};
pub use minimizer::{MinimizeConfig, MinimizeResult, minimize_puzzle};

#[derive(thiserror::Error, Debug)]
pub enum GenError {
    #[error(transparent)]
    Core(#[from] kenken_core::CoreError),
    #[error(transparent)]
    Solve(#[from] SolveError),
    #[error("generation requires `kenken-gen/gen-dlx` (and `kenken-solver/solver-dlx`)")]
    DlxRequired,
    #[error("generation exhausted attempts ({attempts})")]
    AttemptsExhausted { attempts: u32 },
}

pub fn count_solutions_batch(
    puzzles: &[Puzzle],
    rules: Ruleset,
    tier: DeductionTier,
    limit: u32,
) -> Result<Vec<u32>, GenError> {
    #[cfg(feature = "parallel-rayon")]
    {
        use rayon::prelude::*;
        puzzles
            .par_iter()
            .map(|p| {
                Ok(count_solutions_up_to_with_deductions(
                    p, rules, tier, limit,
                )?)
            })
            .collect()
    }

    #[cfg(not(feature = "parallel-rayon"))]
    {
        puzzles
            .iter()
            .map(|p| {
                Ok(count_solutions_up_to_with_deductions(
                    p, rules, tier, limit,
                )?)
            })
            .collect()
    }
}

pub fn is_unique_batch(
    puzzles: &[Puzzle],
    rules: Ruleset,
    tier: DeductionTier,
) -> Result<Vec<bool>, GenError> {
    Ok(count_solutions_batch(puzzles, rules, tier, 2)?
        .into_iter()
        .map(|c| c == 1)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use kenken_core::format::sgt_desc::parse_keen_desc;

    #[test]
    fn batch_counts_work_for_small_example() {
        let puzzle = parse_keen_desc(2, "b__,a3a3").unwrap();
        let counts = count_solutions_batch(
            &[puzzle],
            Ruleset::keen_baseline(),
            DeductionTier::Normal,
            2,
        )
        .unwrap();
        assert_eq!(counts, vec![2]);
    }
}
