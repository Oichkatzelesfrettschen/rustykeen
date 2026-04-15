//! SAT Solver Interface for Verification
//!
//! This module provides axiomatized SAT verification using Varisat
//! for puzzles where SAT encoding is more efficient than Z3.

use kenken_core::{Puzzle, rules::Ruleset};
use kenken_solver::sat_cages::{export_puzzle_dimacs_strict, puzzle_uniqueness_via_sat_strict};
use kenken_solver::sat_latin::SatUniqueness;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SatVerificationError {
    #[error("Puzzle validation failed: {0}")]
    InvalidPuzzle(String),
    #[error("Solution rejected by verified checker: {0}")]
    InvalidSolution(String),
    #[error("SAT backend disagrees with native verification: {0}")]
    BackendDisagreement(String),
    #[error("SAT strict certification failed: {0}")]
    StrictCertificationFailed(String),
    #[error("CNF export failed: {0}")]
    CnfExportFailed(String),
}

/// Verify a solution using SAT solver
///
/// # Rocq Axiom
/// `axiom sat_verify_agrees: ∀ puzzle solution,
///   sat_verify puzzle solution = true → verify_solution puzzle solution = true`
pub fn verify_with_sat(puzzle: &Puzzle, solution: &[u8]) -> Result<bool, SatVerificationError> {
    let rules = Ruleset::keen_baseline();
    puzzle
        .validate(rules)
        .map_err(|e| SatVerificationError::InvalidPuzzle(e.to_string()))?;

    crate::verified_solver::verify_solution(puzzle, solution)
        .map_err(SatVerificationError::InvalidSolution)?;

    match puzzle_uniqueness_via_sat_strict(puzzle, rules) {
        Ok(SatUniqueness::Unique) => Ok(true),
        Ok(SatUniqueness::Multiple) => Ok(false),
        Ok(SatUniqueness::Unsat) => Err(SatVerificationError::BackendDisagreement(
            "SAT encoding reported UNSAT for a solver-validated candidate".to_string(),
        )),
        Err(e) => Err(SatVerificationError::StrictCertificationFailed(
            e.to_string(),
        )),
    }
}

/// Generate CNF formula for puzzle (for external SAT solvers)
pub fn generate_cnf(puzzle: &Puzzle) -> Result<String, SatVerificationError> {
    let rules = Ruleset::keen_baseline();
    puzzle
        .validate(rules)
        .map_err(|e| SatVerificationError::InvalidPuzzle(e.to_string()))?;
    export_puzzle_dimacs_strict(puzzle, rules)
        .map_err(|e| SatVerificationError::CnfExportFailed(e.to_string()))
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
    fn sat_reports_unique_for_fully_pinned_puzzle() {
        let puzzle = Puzzle {
            n: 2,
            cages: vec![
                mk_cage(&[0], Op::Eq, 1),
                mk_cage(&[1], Op::Eq, 2),
                mk_cage(&[2], Op::Eq, 2),
                mk_cage(&[3], Op::Eq, 1),
            ],
        };
        assert!(verify_with_sat(&puzzle, &[1, 2, 2, 1]).unwrap());
    }

    #[test]
    fn sat_reports_multiple_for_underconstrained_puzzle() {
        let puzzle = Puzzle {
            n: 2,
            cages: vec![mk_cage(&[0, 1], Op::Add, 3), mk_cage(&[2, 3], Op::Add, 3)],
        };
        assert!(!verify_with_sat(&puzzle, &[1, 2, 2, 1]).unwrap());
    }

    #[test]
    fn sat_rejects_invalid_solution_before_backend_check() {
        let puzzle = Puzzle {
            n: 2,
            cages: vec![
                mk_cage(&[0], Op::Eq, 1),
                mk_cage(&[1], Op::Eq, 2),
                mk_cage(&[2], Op::Eq, 2),
                mk_cage(&[3], Op::Eq, 1),
            ],
        };
        let err = verify_with_sat(&puzzle, &[1, 1, 2, 2]).unwrap_err();
        assert!(matches!(err, SatVerificationError::InvalidSolution(_)));
    }

    #[test]
    fn cnf_export_returns_dimacs() {
        let puzzle = Puzzle {
            n: 2,
            cages: vec![
                mk_cage(&[0], Op::Eq, 1),
                mk_cage(&[1], Op::Eq, 2),
                mk_cage(&[2], Op::Eq, 2),
                mk_cage(&[3], Op::Eq, 1),
            ],
        };
        let cnf = generate_cnf(&puzzle).unwrap();
        assert!(cnf.starts_with("p cnf "));
    }

    #[test]
    fn sat_strict_mode_reports_tuple_overflow() {
        let n = 6u8;
        let n_usize = n as usize;
        let mut cages = Vec::new();
        cages.push(Cage {
            cells: (0u16..6u16).map(CellId).collect(),
            op: Op::Add,
            target: 21,
        });
        for row in 1..n_usize {
            for col in 0..n_usize {
                let idx = (row * n_usize + col) as u16;
                let value = ((row + col) % n_usize + 1) as i32;
                cages.push(Cage {
                    cells: [CellId(idx)].into_iter().collect(),
                    op: Op::Eq,
                    target: value,
                });
            }
        }
        let puzzle = Puzzle { n, cages };
        let err = verify_with_sat(
            &puzzle,
            &[
                1, 2, 3, 4, 5, 6, 2, 3, 4, 5, 6, 1, 3, 4, 5, 6, 1, 2, 4, 5, 6, 1, 2, 3, 5, 6, 1, 2,
                3, 4, 6, 1, 2, 3, 4, 5,
            ],
        )
        .unwrap_err();
        assert!(matches!(
            err,
            SatVerificationError::StrictCertificationFailed(_)
        ));
    }
}
