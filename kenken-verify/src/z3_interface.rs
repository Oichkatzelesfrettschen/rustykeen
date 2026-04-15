//! Z3 SMT Solver Interface for Verification
//!
//! This module provides axiomatized Z3 verification for agreement checks.
//! Rather than using Z3 as a solver for large instances, we use it for:
//! - Small puzzle verification (n ≤ 12)
//! - Agreement checking between solver and Z3
//! - Formal axiomatization in Rocq

use kenken_core::{Puzzle, rules::Ruleset};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum Z3VerificationError {
    #[error("Puzzle validation failed: {0}")]
    InvalidPuzzle(String),
    #[error("Solution rejected by verified checker: {0}")]
    InvalidSolution(String),
    #[error("Z3 backend verification failed: {0}")]
    BackendFailure(String),
    #[error("SMT2 export failed: {0}")]
    Smt2ExportFailed(String),
}

/// Verify a solution using Z3 SMT solver (small puzzles only)
///
/// Z3 can effectively handle puzzles up to n ≈ 12 due to constraint explosion.
/// For larger puzzles, use native solver with Rocq proofs instead.
///
/// # Rocq Axiom
/// `axiom z3_verify_agrees: ∀ puzzle solution,
///   z3_verify puzzle solution = true → verify_solution puzzle solution = true`
pub fn verify_with_z3(puzzle: &Puzzle, solution: &[u8]) -> Result<bool, Z3VerificationError> {
    let rules = Ruleset::keen_baseline();
    puzzle
        .validate(rules)
        .map_err(|e| Z3VerificationError::InvalidPuzzle(e.to_string()))?;

    crate::verified_solver::verify_solution(puzzle, solution)
        .map_err(Z3VerificationError::InvalidSolution)?;

    match kenken_solver::z3_verify::verify_solution_is_unique(puzzle, solution) {
        Ok(()) => Ok(true),
        Err(msg) if msg.contains("Found alternative solution") => Ok(false),
        Err(msg) => Err(Z3VerificationError::BackendFailure(msg)),
    }
}

/// Generate Z3 SMT2 encoding of a puzzle (for external verification)
///
/// Output format is Z3 SMT2, suitable for external verification tools.
pub fn generate_z3_smt2(_puzzle: &Puzzle) -> Result<String, Z3VerificationError> {
    kenken_solver::z3_verify::export_puzzle_smt2(_puzzle)
        .map_err(Z3VerificationError::Smt2ExportFailed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kenken_core::{Cage, CellId, rules::Op};

    fn mk_cage(cells: &[u16], op: Op, target: i32) -> Cage {
        Cage {
            cells: cells.iter().copied().map(CellId).collect(),
            op,
            target,
        }
    }

    #[test]
    fn z3_reports_unique_for_fully_pinned_puzzle() {
        let puzzle = Puzzle {
            n: 2,
            cages: vec![
                mk_cage(&[0], Op::Eq, 1),
                mk_cage(&[1], Op::Eq, 2),
                mk_cage(&[2], Op::Eq, 2),
                mk_cage(&[3], Op::Eq, 1),
            ],
        };
        assert!(verify_with_z3(&puzzle, &[1, 2, 2, 1]).unwrap());
    }

    #[test]
    fn z3_reports_non_unique_for_underconstrained_puzzle() {
        let puzzle = Puzzle {
            n: 2,
            cages: vec![mk_cage(&[0, 1], Op::Add, 3), mk_cage(&[2, 3], Op::Add, 3)],
        };
        assert!(!verify_with_z3(&puzzle, &[1, 2, 2, 1]).unwrap());
    }

    #[test]
    fn z3_rejects_invalid_solution_before_backend_check() {
        let puzzle = Puzzle {
            n: 2,
            cages: vec![
                mk_cage(&[0], Op::Eq, 1),
                mk_cage(&[1], Op::Eq, 2),
                mk_cage(&[2], Op::Eq, 2),
                mk_cage(&[3], Op::Eq, 1),
            ],
        };
        let err = verify_with_z3(&puzzle, &[1, 1, 2, 2]).unwrap_err();
        assert!(matches!(err, Z3VerificationError::InvalidSolution(_)));
    }

    #[test]
    fn smt2_export_returns_solver_constraints() {
        let puzzle = Puzzle {
            n: 2,
            cages: vec![
                mk_cage(&[0], Op::Eq, 1),
                mk_cage(&[1], Op::Eq, 2),
                mk_cage(&[2], Op::Eq, 2),
                mk_cage(&[3], Op::Eq, 1),
            ],
        };
        let smt2 = generate_z3_smt2(&puzzle).unwrap();
        assert!(smt2.contains("(declare-fun cell_0"));
    }
}
