//! Z3-based formal verification of puzzle uniqueness.
//!
//! This module provides verification that a KenKen solution is unique
//! by encoding the full puzzle constraints in Z3 and checking if
//! any other solutions exist.
//!
//! Internal implementation module. Not part of the public API.

use kenken_core::Puzzle;

#[cfg(feature = "verify")]
use z3::{
    Config, SatResult, Solver,
    ast::{Ast, Bool, Int},
    with_z3_config,
};

#[cfg(feature = "verify")]
fn build_solver_for_puzzle(puzzle: &Puzzle) -> Result<(Solver, Vec<Int>), String> {
    use kenken_core::rules::Op;

    let width = puzzle.n as usize;
    let solver = Solver::new();

    let cells: Vec<Int> = (0..width * width)
        .map(|i| Int::new_const(format!("cell_{}", i)))
        .collect();

    let n_z3 = Int::from_i64(puzzle.n as i64);
    for cell in &cells {
        solver.assert(cell.ge(Int::from_i64(1)));
        solver.assert(cell.le(&n_z3));
    }

    for row in 0..width {
        let row_cells: Vec<&Int> = (0..width).map(|col| &cells[row * width + col]).collect();
        solver.assert(Int::distinct(&row_cells));
    }

    for col in 0..width {
        let col_cells: Vec<&Int> = (0..width).map(|row| &cells[row * width + col]).collect();
        solver.assert(Int::distinct(&col_cells));
    }

    for (cage_idx, cage) in puzzle.cages.iter().enumerate() {
        let mut cage_cells = Vec::with_capacity(cage.cells.len());
        for cell in &cage.cells {
            let idx = cell.0 as usize;
            let cell_ast = cells.get(idx).cloned().ok_or_else(|| {
                format!("Cage {cage_idx} references out-of-range cell index {idx}")
            })?;
            cage_cells.push(cell_ast);
        }

        let target = Int::from_i64(cage.target as i64);
        match cage.op {
            Op::Eq => {
                if cage_cells.len() != 1 {
                    return Err(format!(
                        "Eq cage {cage_idx} must have exactly one cell, found {}",
                        cage_cells.len()
                    ));
                }
                solver.assert(cage_cells[0].eq(&target));
            }
            Op::Add => {
                let sum = Int::add(&cage_cells);
                solver.assert(sum.eq(&target));
            }
            Op::Mul => {
                let product = Int::mul(&cage_cells);
                solver.assert(product.eq(&target));
            }
            Op::Sub => {
                if cage_cells.len() != 2 {
                    return Err(format!(
                        "Sub cage {cage_idx} must have exactly two cells, found {}",
                        cage_cells.len()
                    ));
                }
                if cage.target <= 0 {
                    solver.assert(Bool::from_bool(false));
                    continue;
                }
                let a = cage_cells[0].clone();
                let b = cage_cells[1].clone();
                let diff_ab = Int::sub(&[a.clone(), b.clone()]);
                let diff_ba = Int::sub(&[b, a]);
                let sub_ok = Bool::or(&[diff_ab.eq(&target), diff_ba.eq(&target)]);
                solver.assert(&sub_ok);
            }
            Op::Div => {
                if cage_cells.len() != 2 {
                    return Err(format!(
                        "Div cage {cage_idx} must have exactly two cells, found {}",
                        cage_cells.len()
                    ));
                }
                if cage.target <= 0 {
                    solver.assert(Bool::from_bool(false));
                    continue;
                }
                let a = cage_cells[0].clone();
                let b = cage_cells[1].clone();
                let ratio_ab = a.eq(Int::mul(&[b.clone(), target.clone()]));
                let ratio_ba = b.eq(Int::mul(&[a, target.clone()]));
                let div_ok = Bool::or(&[ratio_ab, ratio_ba]);
                solver.assert(&div_ok);
            }
        }
    }

    Ok((solver, cells))
}

#[cfg(feature = "verify")]
pub fn verify_solution_is_unique(puzzle: &Puzzle, solution: &[u8]) -> Result<(), String> {
    use kenken_core::rules::Ruleset;

    let n = puzzle.n;
    if solution.len() != (n as usize) * (n as usize) {
        return Err("Solution length mismatch".to_string());
    }

    puzzle
        .validate(Ruleset::keen_baseline())
        .map_err(|e| format!("Puzzle validation failed for Z3 verification: {e}"))?;

    let cfg = Config::new();
    with_z3_config(&cfg, || {
        let (solver, cells) = build_solver_for_puzzle(puzzle)?;

        // Check that the provided solution itself satisfies all constraints.
        solver.push();
        for (i, &cell_value) in solution.iter().enumerate() {
            let known = Int::from_i64(cell_value as i64);
            solver.assert(cells[i].eq(&known));
        }
        match solver.check() {
            SatResult::Sat => {
                solver.pop(1);
            }
            SatResult::Unsat => {
                solver.pop(1);
                return Err("Provided solution does not satisfy Latin+cage constraints".to_string());
            }
            SatResult::Unknown => {
                solver.pop(1);
                return Err(
                    "Z3 returned UNKNOWN while validating the provided solution".to_string()
                );
            }
        }

        // Try to find a solution different from the known one
        // If no such solution exists, the provided solution is unique.
        let mut different: Vec<Bool> = Vec::with_capacity(solution.len());
        for (i, &cell_value) in solution.iter().enumerate() {
            let known = Int::from_i64(cell_value as i64);
            different.push(cells[i].eq(&known).not());
        }
        let any_different = Bool::or(&different);
        solver.assert(&any_different);

        // Check: UNSAT = unique, SAT = not unique
        match solver.check() {
            SatResult::Unsat => Ok(()),
            SatResult::Unknown => Err("Z3 returned UNKNOWN (timeout or incomplete)".to_string()),
            SatResult::Sat => {
                Err("Found alternative solution satisfying full puzzle constraints".to_string())
            }
        }
    })
}

#[cfg(feature = "verify")]
pub fn export_puzzle_smt2(puzzle: &Puzzle) -> Result<String, String> {
    use kenken_core::rules::Ruleset;

    puzzle
        .validate(Ruleset::keen_baseline())
        .map_err(|e| format!("Puzzle validation failed for SMT2 export: {e}"))?;

    let cfg = Config::new();
    with_z3_config(&cfg, || {
        let (solver, _) = build_solver_for_puzzle(puzzle)?;
        Ok(solver.to_smt2())
    })
}

#[cfg(not(feature = "verify"))]
pub fn verify_solution_is_unique(_puzzle: &Puzzle, _solution: &[u8]) -> Result<(), String> {
    Err("Z3 verification requires 'verify' feature".to_string())
}

#[cfg(not(feature = "verify"))]
pub fn export_puzzle_smt2(_puzzle: &Puzzle) -> Result<String, String> {
    Err("Z3 verification requires 'verify' feature".to_string())
}

#[cfg(all(test, feature = "verify"))]
mod tests {
    use super::{export_puzzle_smt2, verify_solution_is_unique};
    use kenken_core::{Cage, CellId, Puzzle, rules::Op};
    use z3::{
        Config, SatResult, Solver,
        ast::{Bool, Int},
        with_z3_config,
    };

    fn mk_cage(cells: &[u16], op: Op, target: i32) -> Cage {
        Cage {
            cells: cells.iter().copied().map(CellId).collect(),
            op,
            target,
        }
    }

    #[test]
    fn z3_verifies_unique_solution_with_cage_constraints() {
        let puzzle = Puzzle {
            n: 2,
            cages: vec![mk_cage(&[0], Op::Eq, 1), mk_cage(&[1, 2, 3], Op::Add, 5)],
        };
        let solution = [1, 2, 2, 1];
        assert!(verify_solution_is_unique(&puzzle, &solution).is_ok());
    }

    #[test]
    fn z3_detects_non_unique_add_puzzle() {
        let puzzle = Puzzle {
            n: 2,
            cages: vec![mk_cage(&[0, 1], Op::Add, 3), mk_cage(&[2, 3], Op::Add, 3)],
        };
        let solution = [1, 2, 2, 1];
        assert!(verify_solution_is_unique(&puzzle, &solution).is_err());
    }

    #[test]
    fn z3_detects_non_unique_mul_puzzle() {
        let puzzle = Puzzle {
            n: 2,
            cages: vec![mk_cage(&[0, 1], Op::Mul, 2), mk_cage(&[2, 3], Op::Mul, 2)],
        };
        let solution = [1, 2, 2, 1];
        assert!(verify_solution_is_unique(&puzzle, &solution).is_err());
    }

    #[test]
    fn z3_detects_non_unique_sub_puzzle() {
        let puzzle = Puzzle {
            n: 2,
            cages: vec![mk_cage(&[0, 1], Op::Sub, 1), mk_cage(&[2, 3], Op::Sub, 1)],
        };
        let solution = [1, 2, 2, 1];
        assert!(verify_solution_is_unique(&puzzle, &solution).is_err());
    }

    #[test]
    fn z3_detects_non_unique_div_puzzle() {
        let puzzle = Puzzle {
            n: 2,
            cages: vec![mk_cage(&[0, 1], Op::Div, 2), mk_cage(&[2, 3], Op::Div, 2)],
        };
        let solution = [1, 2, 2, 1];
        assert!(verify_solution_is_unique(&puzzle, &solution).is_err());
    }

    #[test]
    fn z3_rejects_invalid_known_solution() {
        let puzzle = Puzzle {
            n: 2,
            cages: vec![mk_cage(&[0], Op::Eq, 1), mk_cage(&[1, 2, 3], Op::Add, 5)],
        };
        let wrong_solution = [2, 1, 1, 2];
        assert!(verify_solution_is_unique(&puzzle, &wrong_solution).is_err());
    }

    #[test]
    fn smt2_export_matches_uniqueness_check_for_unique_puzzle() {
        let puzzle = Puzzle {
            n: 2,
            cages: vec![
                mk_cage(&[0], Op::Eq, 1),
                mk_cage(&[1], Op::Eq, 2),
                mk_cage(&[2], Op::Eq, 2),
                mk_cage(&[3], Op::Eq, 1),
            ],
        };
        let solution = [1u8, 2, 2, 1];
        let smt2 = export_puzzle_smt2(&puzzle).unwrap();
        assert!(smt2.contains("cell_0"));

        let cfg = Config::new();
        with_z3_config(&cfg, || {
            let solver = Solver::new();
            solver.from_string(smt2.as_bytes());
            assert!(matches!(solver.check(), SatResult::Sat));

            solver.push();
            for (i, &value) in solution.iter().enumerate() {
                let var = Int::new_const(format!("cell_{i}"));
                solver.assert(var.eq(Int::from_i64(value as i64)));
            }
            assert!(matches!(solver.check(), SatResult::Sat));
            solver.pop(1);

            let mut any_different = Vec::with_capacity(solution.len());
            for (i, &value) in solution.iter().enumerate() {
                let var = Int::new_const(format!("cell_{i}"));
                any_different.push(var.eq(Int::from_i64(value as i64)).not());
            }
            solver.assert(Bool::or(&any_different));
            assert!(matches!(solver.check(), SatResult::Unsat));
        });

        assert!(verify_solution_is_unique(&puzzle, &solution).is_ok());
    }
}
