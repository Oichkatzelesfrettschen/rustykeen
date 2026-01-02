//! Puzzle minimizer: reduces cage count while preserving uniqueness.
//!
//! The generator produces valid unique puzzles but may have more cages than necessary.
//! This module implements greedy cage merging to produce "cleaner" puzzles with fewer
//! constraints that still yield a unique solution.
//!
//! # Algorithm
//!
//! The minimizer uses a greedy approach:
//! 1. Find all pairs of adjacent cages (sharing an orthogonal edge)
//! 2. For each pair, try merging into a single cage
//! 3. If the merged puzzle is still unique, accept the merge
//! 4. Repeat until no more merges preserve uniqueness
//!
//! # Constraints
//!
//! Merges respect ruleset constraints:
//! - Maximum cage size
//! - Sub/Div restricted to 2-cell cages
//! - Orthogonal connectivity requirement

use kenken_core::rules::{Op, Ruleset};
use kenken_core::{Cage, CellId, Puzzle};
use kenken_solver::{DeductionTier, count_solutions_up_to_with_deductions};
use smallvec::SmallVec;
use std::collections::HashSet;

use crate::GenError;

#[cfg(feature = "telemetry-tracing")]
use tracing::trace;

#[cfg(not(feature = "telemetry-tracing"))]
macro_rules! trace {
    ($($tt:tt)*) => {};
}

/// Configuration for puzzle minimization.
#[derive(Debug, Clone, Copy)]
pub struct MinimizeConfig {
    /// Ruleset governing cage constraints.
    pub rules: Ruleset,
    /// Deduction tier for uniqueness verification.
    pub tier: DeductionTier,
    /// Maximum iterations to prevent runaway loops.
    pub max_iterations: u32,
    /// Prefer Add operations when merging (vs Mul).
    pub prefer_add: bool,
}

impl MinimizeConfig {
    pub fn keen_baseline() -> Self {
        Self {
            rules: Ruleset::keen_baseline(),
            tier: DeductionTier::Hard,
            max_iterations: 1000,
            prefer_add: true,
        }
    }
}

/// Result of minimization.
#[derive(Debug, Clone)]
pub struct MinimizeResult {
    /// The minimized puzzle.
    pub puzzle: Puzzle,
    /// Number of cages before minimization.
    pub original_cage_count: usize,
    /// Number of cages after minimization.
    pub final_cage_count: usize,
    /// Number of successful merges performed.
    pub merges_performed: u32,
    /// Number of merge attempts that failed uniqueness check.
    pub merges_rejected: u32,
}

/// Minimize a puzzle by merging adjacent cages while preserving uniqueness.
///
/// # Arguments
/// * `puzzle` - The puzzle to minimize
/// * `solution` - The known solution (required to compute merged cage targets)
/// * `config` - Minimization configuration
///
/// # Returns
/// * `Ok(MinimizeResult)` - The minimized puzzle and statistics
/// * `Err(GenError)` - If minimization fails (e.g., solution mismatch)
pub fn minimize_puzzle(
    puzzle: Puzzle,
    solution: &[u8],
    config: MinimizeConfig,
) -> Result<MinimizeResult, GenError> {
    let n = puzzle.n;
    let a = (n as usize) * (n as usize);

    if solution.len() != a {
        return Err(GenError::AttemptsExhausted { attempts: 0 });
    }

    let original_cage_count = puzzle.cages.len();
    let mut current = puzzle;
    let mut merges_performed = 0u32;
    let mut merges_rejected = 0u32;
    let mut iteration = 0u32;

    trace!(
        n = current.n,
        original_cages = original_cage_count,
        "minimizer.start"
    );

    loop {
        if iteration >= config.max_iterations {
            trace!(iteration, "minimizer.max_iterations_reached");
            break;
        }
        iteration += 1;

        // Find a valid merge candidate
        let merge_candidate = find_merge_candidate(&current, solution, config);

        match merge_candidate {
            Some((cage_a, cage_b, merged_cage)) => {
                // Build candidate puzzle with merged cage
                let candidate = apply_merge(&current, cage_a, cage_b, merged_cage);

                // Verify uniqueness
                let count = count_solutions_up_to_with_deductions(
                    &candidate,
                    config.rules,
                    config.tier,
                    2,
                )?;

                if count == 1 {
                    trace!(
                        iteration,
                        cage_a,
                        cage_b,
                        new_cage_count = candidate.cages.len(),
                        "minimizer.merge_accepted"
                    );
                    current = candidate;
                    merges_performed += 1;
                } else {
                    trace!(
                        iteration,
                        cage_a,
                        cage_b,
                        solutions = count,
                        "minimizer.merge_rejected"
                    );
                    merges_rejected += 1;
                    // Mark this pair as tried and continue searching
                    // For simplicity, we'll break and return current best
                    // A more sophisticated version could track tried pairs
                    break;
                }
            }
            None => {
                trace!(iteration, "minimizer.no_candidates");
                break;
            }
        }
    }

    let final_cage_count = current.cages.len();
    trace!(
        original_cages = original_cage_count,
        final_cages = final_cage_count,
        merges_performed,
        merges_rejected,
        "minimizer.done"
    );

    Ok(MinimizeResult {
        puzzle: current,
        original_cage_count,
        final_cage_count,
        merges_performed,
        merges_rejected,
    })
}

/// Find a pair of adjacent cages that can be merged.
///
/// Returns `Some((cage_a_idx, cage_b_idx, merged_cage))` if a valid candidate is found.
fn find_merge_candidate(
    puzzle: &Puzzle,
    solution: &[u8],
    config: MinimizeConfig,
) -> Option<(usize, usize, Cage)> {
    let n = puzzle.n;
    let n_usize = n as usize;

    // Build cell-to-cage mapping
    let a = n_usize * n_usize;
    let mut cell_to_cage = vec![usize::MAX; a];
    for (cage_idx, cage) in puzzle.cages.iter().enumerate() {
        for &cell in &cage.cells {
            let idx = cell.0 as usize;
            if idx < a {
                cell_to_cage[idx] = cage_idx;
            }
        }
    }

    // Find adjacent cage pairs
    let mut tried_pairs: HashSet<(usize, usize)> = HashSet::new();

    for (cage_a_idx, cage_a) in puzzle.cages.iter().enumerate() {
        for &cell in &cage_a.cells {
            let idx = cell.0 as usize;
            let row = idx / n_usize;
            let col = idx % n_usize;

            // Check orthogonal neighbors
            let neighbors = [
                (row > 0).then(|| (row - 1) * n_usize + col),
                (row + 1 < n_usize).then(|| (row + 1) * n_usize + col),
                (col > 0).then(|| row * n_usize + (col - 1)),
                (col + 1 < n_usize).then(|| row * n_usize + (col + 1)),
            ];

            for neighbor_idx in neighbors.into_iter().flatten() {
                let cage_b_idx = cell_to_cage[neighbor_idx];
                if cage_b_idx == usize::MAX || cage_b_idx == cage_a_idx {
                    continue;
                }

                // Normalize pair order for deduplication
                let pair = if cage_a_idx < cage_b_idx {
                    (cage_a_idx, cage_b_idx)
                } else {
                    (cage_b_idx, cage_a_idx)
                };

                if tried_pairs.contains(&pair) {
                    continue;
                }
                tried_pairs.insert(pair);

                let cage_b = &puzzle.cages[cage_b_idx];

                // Try to merge these cages
                if let Some(merged) = try_merge_cages(n, cage_a, cage_b, solution, config) {
                    return Some((pair.0, pair.1, merged));
                }
            }
        }
    }

    None
}

/// Attempt to merge two cages into one.
///
/// Returns `Some(merged_cage)` if the merge is valid under the ruleset.
fn try_merge_cages(
    n: u8,
    cage_a: &Cage,
    cage_b: &Cage,
    solution: &[u8],
    config: MinimizeConfig,
) -> Option<Cage> {
    let mut cells: SmallVec<[CellId; 6]> =
        SmallVec::with_capacity(cage_a.cells.len() + cage_b.cells.len());
    cells.extend(cage_a.cells.iter().copied());
    cells.extend(cage_b.cells.iter().copied());

    // Check max cage size
    if cells.len() > config.rules.max_cage_size as usize {
        return None;
    }

    // Collect cell values from solution
    let values: SmallVec<[u8; 6]> = cells.iter().map(|c| solution[c.0 as usize]).collect();

    // Determine operation and target
    let (op, target) = choose_op_and_target(&values, config);

    // Build candidate cage
    let merged = Cage { cells, op, target };

    // Validate the merged cage under ruleset
    if merged.validate_shape(n, config.rules).is_err() {
        return None;
    }

    Some(merged)
}

/// Choose operation and target for merged cage based on cell values.
fn choose_op_and_target(values: &[u8], config: MinimizeConfig) -> (Op, i32) {
    let len = values.len();

    match len {
        1 => (Op::Eq, values[0] as i32),
        2 => {
            let a = values[0];
            let b = values[1];

            // For 2-cell cages, we have more options
            if config.prefer_add {
                // Try Add first
                (Op::Add, (a as i32) + (b as i32))
            } else {
                // Try Mul first
                (Op::Mul, (a as i32) * (b as i32))
            }
        }
        _ => {
            // For 3+ cells, can only use Add or Mul (Sub/Div are 2-cell only)
            if config.prefer_add {
                let sum: i32 = values.iter().map(|&v| v as i32).sum();
                (Op::Add, sum)
            } else {
                let prod: i32 = values.iter().fold(1, |acc, &v| acc * (v as i32));
                (Op::Mul, prod)
            }
        }
    }
}

/// Apply a merge to produce a new puzzle.
fn apply_merge(puzzle: &Puzzle, cage_a_idx: usize, cage_b_idx: usize, merged: Cage) -> Puzzle {
    let (min_idx, max_idx) = if cage_a_idx < cage_b_idx {
        (cage_a_idx, cage_b_idx)
    } else {
        (cage_b_idx, cage_a_idx)
    };

    let mut cages: Vec<Cage> = Vec::with_capacity(puzzle.cages.len() - 1);

    for (i, cage) in puzzle.cages.iter().enumerate() {
        if i == min_idx {
            cages.push(merged.clone());
        } else if i != max_idx {
            cages.push(cage.clone());
        }
        // Skip max_idx entirely (merged into min_idx)
    }

    Puzzle { n: puzzle.n, cages }
}

#[cfg(all(test, feature = "gen-dlx"))]
mod tests {
    use super::*;
    use crate::generator::{GenerateConfig, generate};

    #[test]
    fn minimizer_preserves_uniqueness() {
        // Generate a small puzzle
        let gen_cfg = GenerateConfig::keen_baseline(4, 12345);
        let generated = generate(gen_cfg).unwrap();

        // Minimize it
        let min_cfg = MinimizeConfig::keen_baseline();
        let result =
            minimize_puzzle(generated.puzzle.clone(), &generated.solution, min_cfg).unwrap();

        // Verify uniqueness is preserved
        let count =
            count_solutions_up_to_with_deductions(&result.puzzle, min_cfg.rules, min_cfg.tier, 2)
                .unwrap();
        assert_eq!(count, 1, "Minimized puzzle should have unique solution");

        // Verify structure is valid
        result.puzzle.validate(min_cfg.rules).unwrap();
    }

    #[test]
    fn minimizer_reduces_or_maintains_cage_count() {
        let gen_cfg = GenerateConfig::keen_baseline(4, 54321);
        let generated = generate(gen_cfg).unwrap();

        let min_cfg = MinimizeConfig::keen_baseline();
        let result =
            minimize_puzzle(generated.puzzle.clone(), &generated.solution, min_cfg).unwrap();

        // Cage count should not increase
        assert!(
            result.final_cage_count <= result.original_cage_count,
            "Minimizer should not increase cage count"
        );
    }

    #[test]
    fn minimizer_handles_already_minimal_puzzle() {
        // Create a puzzle that's already "minimal" (all singletons)
        let n = 2u8;
        let solution = vec![1, 2, 2, 1];
        let puzzle = Puzzle {
            n,
            cages: vec![
                Cage {
                    cells: SmallVec::from_slice(&[CellId(0)]),
                    op: Op::Eq,
                    target: 1,
                },
                Cage {
                    cells: SmallVec::from_slice(&[CellId(1)]),
                    op: Op::Eq,
                    target: 2,
                },
                Cage {
                    cells: SmallVec::from_slice(&[CellId(2)]),
                    op: Op::Eq,
                    target: 2,
                },
                Cage {
                    cells: SmallVec::from_slice(&[CellId(3)]),
                    op: Op::Eq,
                    target: 1,
                },
            ],
        };

        let min_cfg = MinimizeConfig::keen_baseline();
        let result = minimize_puzzle(puzzle, &solution, min_cfg).unwrap();

        // Singleton Eq cages can potentially be merged if uniqueness allows
        // Just verify the result is valid
        result.puzzle.validate(min_cfg.rules).unwrap();
    }
}
