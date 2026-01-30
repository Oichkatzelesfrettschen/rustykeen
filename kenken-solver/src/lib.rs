#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

#[cfg(feature = "solver-dlx")]
mod dlx;

/// Dancing Links exact cover solver for Latin square constraints.
///
/// Internal implementation module used by kenken-gen for puzzle generation.
/// Not part of the primary public API but accessible for advanced use.
#[cfg(feature = "solver-dlx")]
pub mod dlx_latin;

/// Domain representation traits and 32/64-bit implementations.
///
/// Part of the stable public API for custom solver integrations.
pub mod domain_ops;

/// Fixed bitset domain representation (requires `solver-fixedbitset` feature).
///
/// Part of the stable public API for custom solver integrations.
#[cfg(feature = "solver-fixedbitset")]
pub mod domain_fixedbitset;

/// 128-bit SIMD domain representation (requires `solver-u128` feature).
///
/// Part of the stable public API for custom solver integrations.
#[cfg(feature = "solver-u128")]
pub mod domain_simd128;

/// 256-bit SIMD domain representation (requires `solver-u256` feature).
///
/// Part of the stable public API for custom solver integrations.
#[cfg(feature = "solver-u256")]
pub mod domain_simd256;

/// Solver error types and diagnostics.
pub mod error;

/// Instrumentation and event callback system for solver visualization (requires `ui-instrumentation` feature).
///
/// Part of the stable public API for visualization and UI integration.
#[cfg(feature = "ui-instrumentation")]
pub mod instrumentation;

/// Conflict-Driven Learning (CDL) and nogood recording.
///
/// Internal implementation module for advanced search pruning.
/// Not part of the public API.
#[cfg(feature = "nogood-learning")]
pub(crate) mod nogood;

/// Parallel search with rayon work-stealing scheduler.
///
/// Internal implementation module for multi-threaded solving.
/// Not part of the public API.
#[cfg(feature = "parallel-search")]
pub(crate) mod parallel;

/// SAT-based cage constraint encoding with Varisat.
///
/// Internal implementation module for advanced constraint encoding.
/// Accessible for specialized use cases and verification workflows.
#[cfg(feature = "sat-varisat")]
pub mod sat_cages;

/// Symmetry breaking reductions for search pruning.
///
/// Internal implementation module for symmetry handling.
/// Not part of the public API.
#[cfg(feature = "symmetry-breaking")]
pub(crate) mod symmetry;

/// Common SAT utility functions for both Latin and cage constraints.
///
/// Shared utility types and functions for SAT-based constraint encoding.
#[cfg(feature = "sat-varisat")]
pub mod sat_common;

/// SAT-based Latin square constraint encoding.
///
/// Provides SAT-based uniqueness verification and constraint encoding for Latin squares.
#[cfg(feature = "sat-varisat")]
pub mod sat_latin;

/// Core deterministic backtracking solver with deduction tiers.
///
/// Part of the stable public API. Provides `solve_one()`, `count_solutions_up_to()`,
/// and difficulty classification functions.
pub mod solver;

/// Z3 SMT solver verification backend.
///
/// Provides formal verification of puzzle uniqueness using Z3.
/// Part of the extended API for verification workflows.
#[cfg(feature = "verify")]
pub mod z3_verify;

/// Domain representation trait for constraint propagation.
pub use crate::domain_ops::{Domain32, Domain64, DomainOps};

/// Fixed bitset domain (requires `solver-fixedbitset` feature).
#[cfg(feature = "solver-fixedbitset")]
pub use crate::domain_fixedbitset::FixedBitDomain;

/// 128-bit SIMD domain (requires `solver-u128` feature).
#[cfg(feature = "solver-u128")]
pub use crate::domain_simd128::Domain128;

/// 256-bit SIMD domain (requires `solver-u256` feature).
#[cfg(feature = "solver-u256")]
pub use crate::domain_simd256::Domain256;

/// Solver error type for all public API functions.
pub use crate::error::SolveError;

/// Instrumentation types for solver visualization (requires `ui-instrumentation` feature).
#[cfg(feature = "ui-instrumentation")]
pub use crate::instrumentation::{EventCollector, SolverEvent, SolverEventCallback};

/// Main public API exports from solver module.
pub use crate::solver::{
    DeductionTier, DifficultyTier, Solution, SolveStats, TierRequiredResult, classify_difficulty,
    classify_difficulty_from_tier, classify_tier_required, count_solutions_up_to,
    count_solutions_up_to_with_deductions, solve_one, solve_one_with_deductions,
    solve_one_with_stats,
};

/// Puzzle model from kenken-core.
pub use kenken_core::Puzzle;

/// Ruleset configuration for puzzle validation and solving.
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
            hint: "Grid size exceeds 63. Enable 'solver-bitdomain' feature for >63 support"
                .to_string(),
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
pub fn solve_one_dispatched(
    puzzle: &Puzzle,
    rules: Ruleset,
) -> Result<Option<Solution>, SolveError> {
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
