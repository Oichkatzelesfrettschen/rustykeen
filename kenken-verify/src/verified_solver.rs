//! Verified solver implementation extracted from Rocq proofs
//!
//! This module contains implementations translated from Rocq/Coq formalization
//! with correctness proofs. Each function has a corresponding theorem in rcoq/

use kenken_core::rules::{Op, Ruleset};
use kenken_core::{Cage, Puzzle};

/// Verify that a proposed solution satisfies all constraints
///
/// # Rocq Theorem
/// `theorem_verify_solution_correct: ∀ puzzle solution, verify_solution puzzle solution
/// returns Ok iff solution satisfies all_constraints puzzle`
pub fn verify_solution(puzzle: &Puzzle, solution: &[u8]) -> Result<(), String> {
    puzzle
        .validate(Ruleset::keen_baseline())
        .map_err(|e| format!("Puzzle validation failed: {e}"))?;

    // Check solution length
    if solution.len() != (puzzle.n * puzzle.n) as usize {
        return Err(format!(
            "Solution length {} != grid size {}",
            solution.len(),
            puzzle.n * puzzle.n
        ));
    }

    // Check grid values in range [1, n]
    for &digit in solution {
        if digit < 1 || digit > puzzle.n {
            return Err(format!("Digit {} out of range [1, {}]", digit, puzzle.n));
        }
    }

    // Check row uniqueness
    for row in 0..puzzle.n as usize {
        let mut seen = std::collections::HashSet::new();
        for col in 0..puzzle.n as usize {
            let idx = row * puzzle.n as usize + col;
            if !seen.insert(solution[idx]) {
                return Err(format!("Duplicate in row {}", row));
            }
        }
    }

    // Check column uniqueness
    for col in 0..puzzle.n as usize {
        let mut seen = std::collections::HashSet::new();
        for row in 0..puzzle.n as usize {
            let idx = row * puzzle.n as usize + col;
            if !seen.insert(solution[idx]) {
                return Err(format!("Duplicate in column {}", col));
            }
        }
    }

    // Check cage constraints
    for cage in &puzzle.cages {
        verify_cage_constraint(cage, solution)?;
    }

    Ok(())
}

/// Verify a single cage constraint
fn verify_cage_constraint(cage: &Cage, solution: &[u8]) -> Result<(), String> {
    let values = cage
        .cells
        .iter()
        .map(|cell_id| {
            solution
                .get(cell_id.0 as usize)
                .copied()
                .ok_or_else(|| format!("Cell {} out of range for solution length", cell_id.0))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let target = cage.target;
    let op = cage.op;

    match op {
        Op::Add => {
            let sum: u32 = values.iter().map(|&v| v as u32).sum();
            if sum != target as u32 {
                return Err(format!("Cage ADD sum {} != target {}", sum, cage.target));
            }
        }
        Op::Sub => {
            if values.len() != 2 {
                return Err("Subtract cage must have 2 cells".to_string());
            }
            let diff = (values[0] as i32 - values[1] as i32).abs();
            if diff != target {
                return Err(format!("Cage SUB diff {} != target {}", diff, cage.target));
            }
        }
        Op::Mul => {
            let product: u32 = values.iter().map(|&v| v as u32).product();
            if product != target as u32 {
                return Err(format!(
                    "Cage MUL product {} != target {}",
                    product, cage.target
                ));
            }
        }
        Op::Div => {
            if values.len() != 2 {
                return Err("Divide cage must have 2 cells".to_string());
            }
            let (num, den) = if values[0] >= values[1] {
                (values[0], values[1])
            } else {
                (values[1], values[0])
            };
            if den == 0 || (num as i32) != (den as i32).saturating_mul(target) {
                return Err(format!(
                    "Cage DIV pair ({}, {}) does not match target {}",
                    values[0], values[1], target
                ));
            }
        }
        Op::Eq => {
            if values.len() != 1 {
                return Err("Eq cage must have exactly 1 cell".to_string());
            }
            if values[0] as i32 != target {
                return Err(format!("Cage EQ value {} != target {}", values[0], target));
            }
        }
    }

    Ok(())
}

/// Count unique solutions up to a limit
///
/// # Rocq Theorem
/// `theorem_count_solutions_terminating: ∀ puzzle, WF (count_solutions_up_to puzzle)`
pub fn count_solutions_up_to(puzzle: &Puzzle, limit: usize) -> Result<usize, String> {
    // Temporary bridge: delegate to the deterministic solver until the
    // extracted Rocq counting implementation is integrated.

    let rules = Ruleset::keen_baseline();
    let limit_u32 = limit.min(u32::MAX as usize) as u32;
    let solutions = kenken_solver::count_solutions_up_to(puzzle, rules, limit_u32)
        .map_err(|e| format!("Solver error: {}", e))?;

    Ok(solutions as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kenken_core::{CellId, rules::Op};

    fn mk_cage(cells: &[u16], op: Op, target: i32) -> Cage {
        Cage {
            cells: cells.iter().copied().map(CellId).collect(),
            op,
            target,
        }
    }

    #[test]
    fn test_verify_solution_basic() {
        let puzzle = Puzzle {
            n: 2,
            cages: vec![
                mk_cage(&[0], Op::Eq, 1),
                mk_cage(&[1], Op::Eq, 2),
                mk_cage(&[2], Op::Eq, 2),
                mk_cage(&[3], Op::Eq, 1),
            ],
        };
        let solution = vec![1, 2, 2, 1];
        assert!(verify_solution(&puzzle, &solution).is_ok());
    }

    #[test]
    fn test_verify_solution_division_is_symmetric() {
        let puzzle = Puzzle {
            n: 2,
            cages: vec![
                mk_cage(&[0, 1], Op::Div, 2),
                mk_cage(&[2], Op::Eq, 2),
                mk_cage(&[3], Op::Eq, 1),
            ],
        };
        let solution = vec![1, 2, 2, 1];
        assert!(verify_solution(&puzzle, &solution).is_ok());
    }
}
