#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

#[cfg(feature = "solver-dlx")]
pub mod dlx_latin;
pub mod domain_ops;
#[cfg(feature = "solver-fixedbitset")]
pub mod domain_fixedbitset;
#[cfg(feature = "solver-smallbitvec")]
pub mod domain_smallbitvec;
pub mod error;
#[cfg(feature = "sat-varisat")]
pub mod sat_cages;
#[cfg(feature = "sat-varisat")]
pub mod sat_common;
#[cfg(feature = "sat-varisat")]
pub mod sat_latin;
pub mod solver;
#[cfg(feature = "verify")]
pub mod z3_verify;

pub use crate::domain_ops::{Domain32, Domain64, DomainOps};
#[cfg(feature = "solver-fixedbitset")]
pub use crate::domain_fixedbitset::FixedBitDomain;
#[cfg(feature = "solver-smallbitvec")]
pub use crate::domain_smallbitvec::SmallBitDomain;
pub use crate::error::SolveError;
pub use crate::solver::{
    DeductionTier, DifficultyTier, Solution, SolveStats, TierRequiredResult, classify_difficulty,
    classify_difficulty_from_tier, classify_tier_required, count_solutions_up_to,
    count_solutions_up_to_with_deductions, solve_one, solve_one_with_deductions,
    solve_one_with_stats,
};
pub use kenken_core::Puzzle;
pub use kenken_core::rules::Ruleset;

/// Validates that the puzzle grid size is supported by the current feature configuration.
///
/// Returns `Ok(())` if the grid size is valid for the current features.
/// Returns `Err(SolveError::GridSizeTooLarge)` if the grid size exceeds supported limits.
fn validate_grid_size(n: u8) -> Result<(), SolveError> {
    // Feature-gated grid size validation matching kenken-core
    #[cfg(not(any(feature = "solver-u64", feature = "solver-bitdomain")))]
    if n > 31 {
        return Err(SolveError::GridSizeTooLarge {
            n,
            hint: "Grid size exceeds 31. Enable 'solver-u64' feature for 32-63 support".to_string(),
        });
    }

    #[cfg(all(feature = "solver-u64", not(feature = "solver-bitdomain")))]
    if n > 63 {
        return Err(SolveError::GridSizeTooLarge {
            n,
            hint: "Grid size exceeds 63. Enable 'solver-bitdomain' feature for >63 support".to_string(),
        });
    }

    #[cfg(feature = "solver-bitdomain")]
    {
        // BitDomain supports up to u8::MAX (255), which is the natural limit for n
        // so we don't need an explicit check here
        let _ = n; // Use n to avoid unused variable warning
    }

    Ok(())
}

/// Solves a puzzle with grid size validation.
///
/// This is a dispatch wrapper that validates the puzzle can be solved
/// with the current feature configuration before attempting to solve it.
pub fn solve_one_dispatched(puzzle: &Puzzle, rules: Ruleset) -> Result<Option<Solution>, SolveError> {
    validate_grid_size(puzzle.n)?;
    solver::solve_one(puzzle, rules)
}

/// Solves a puzzle with statistics and grid size validation.
pub fn solve_one_with_stats_dispatched(
    puzzle: &Puzzle,
    rules: Ruleset,
) -> Result<(Option<Solution>, SolveStats), SolveError> {
    validate_grid_size(puzzle.n)?;
    solver::solve_one_with_stats(puzzle, rules)
}

/// Solves a puzzle with custom deduction tier and grid size validation.
pub fn solve_one_with_deductions_dispatched(
    puzzle: &Puzzle,
    rules: Ruleset,
    tier: DeductionTier,
) -> Result<Option<Solution>, SolveError> {
    validate_grid_size(puzzle.n)?;
    solver::solve_one_with_deductions(puzzle, rules, tier)
}

/// Counts solutions up to a limit with grid size validation.
pub fn count_solutions_up_to_dispatched(
    puzzle: &Puzzle,
    rules: Ruleset,
    limit: u32,
) -> Result<u32, SolveError> {
    validate_grid_size(puzzle.n)?;
    solver::count_solutions_up_to(puzzle, rules, limit)
}

/// Counts solutions up to a limit with custom deduction tier and grid size validation.
pub fn count_solutions_up_to_with_deductions_dispatched(
    puzzle: &Puzzle,
    rules: Ruleset,
    tier: DeductionTier,
    limit: u32,
) -> Result<u32, SolveError> {
    validate_grid_size(puzzle.n)?;
    solver::count_solutions_up_to_with_deductions(puzzle, rules, tier, limit)
}

/// Classifies the minimum deduction tier required to solve a puzzle with grid size validation.
pub fn classify_tier_required_dispatched(
    puzzle: &Puzzle,
    rules: Ruleset,
) -> Result<TierRequiredResult, SolveError> {
    validate_grid_size(puzzle.n)?;
    solver::classify_tier_required(puzzle, rules)
}
