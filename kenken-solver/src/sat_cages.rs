//! Cage constraint SAT encoding utilities (Varisat).
//!
//! This module is a staging area for extending SAT support from Latin-only
//! (`sat_latin`) to full KenKen cage arithmetic. See `docs/sat_cage_encoding.md`.

use kenken_core::rules::{Op, Ruleset};
use kenken_core::{Cage, Puzzle};
use smallvec::SmallVec;
use varisat::{ExtendFormula, Lit, Solver, Var};

use crate::sat_common::LatinVarMap;
use crate::sat_latin::SatUniqueness;
use crate::{DeductionTier, count_solutions_up_to_with_deductions};

#[cfg(feature = "tracing")]
use tracing::trace;

#[cfg(not(feature = "tracing"))]
macro_rules! trace {
    ($($tt:tt)*) => {};
}

/// Upper bound on enumerated satisfying tuples per cage for SAT allowlist encoding.
///
/// If a cage exceeds this threshold, SAT encoding is considered too large for the current strategy
/// and callers should fall back to non-SAT verification paths (or future encodings).
pub const SAT_TUPLE_THRESHOLD: usize = 512;

fn add_eq_cage_clauses(solver: &mut Solver, map: &LatinVarMap, cage: &Cage) -> bool {
    if cage.cells.len() != 1 {
        return false;
    }
    let n = map.n();
    let idx = cage.cells[0].0 as usize;
    let row = idx / n;
    let col = idx % n;
    if cage.target <= 0 || cage.target > n as i32 {
        return false;
    }
    solver.add_clause(&[map.lit(row, col, cage.target as usize - 1)]);
    true
}

fn allowed_sub_pair(a: u8, b: u8, target: i32) -> bool {
    (a as i32 - b as i32).abs() == target
}

fn allowed_div_pair(a: u8, b: u8, target: i32) -> bool {
    let (num, den) = if a >= b { (a, b) } else { (b, a) };
    den != 0 && (num as i32) == (den as i32).saturating_mul(target)
}

fn add_two_cell_sub_div_cage_clauses(solver: &mut Solver, map: &LatinVarMap, cage: &Cage) -> bool {
    if cage.cells.len() != 2 {
        return false;
    }
    let n = map.n();
    let a_idx = cage.cells[0].0 as usize;
    let b_idx = cage.cells[1].0 as usize;
    let (ar, ac) = (a_idx / n, a_idx % n);
    let (br, bc) = (b_idx / n, b_idx % n);

    let mut selectors: Vec<(Var, u8, u8)> = Vec::new();
    for av in 1..=n as u8 {
        for bv in 1..=n as u8 {
            let ok = match cage.op {
                Op::Sub => allowed_sub_pair(av, bv, cage.target),
                Op::Div => allowed_div_pair(av, bv, cage.target),
                _ => false,
            };
            if !ok {
                continue;
            }
            let s = solver.new_var();
            selectors.push((s, av, bv));
        }
    }
    if selectors.is_empty() {
        return false;
    }

    trace!(
        op = ?cage.op,
        cells = 2,
        selectors = selectors.len(),
        "sat.encode.subdiv.selectors"
    );

    // At least one selector.
    solver.add_clause(
        &selectors
            .iter()
            .map(|(s, _, _)| Lit::from_var(*s, true))
            .collect::<Vec<_>>(),
    );
    // At most one selector (pairwise).
    for i in 0..selectors.len() {
        for j in (i + 1)..selectors.len() {
            solver.add_clause(&[
                Lit::from_var(selectors[i].0, false),
                Lit::from_var(selectors[j].0, false),
            ]);
        }
    }
    // Selector implies assignments.
    for (s, av, bv) in selectors {
        solver.add_clause(&[Lit::from_var(s, false), map.lit(ar, ac, av as usize - 1)]);
        solver.add_clause(&[Lit::from_var(s, false), map.lit(br, bc, bv as usize - 1)]);
    }

    true
}

fn add_tuple_allowlist(
    solver: &mut Solver,
    map: &LatinVarMap,
    cage: &Cage,
    tuples: &[SmallVec<[u8; 6]>],
) -> bool {
    if tuples.is_empty() {
        return false;
    }

    let n = map.n();
    let cells: Vec<usize> = cage.cells.iter().map(|c| c.0 as usize).collect();
    if tuples.iter().any(|t| t.len() != cells.len()) {
        return false;
    }

    // One selector per tuple, exactly one selector, selector implies assignments.
    let mut selectors: Vec<Var> = Vec::with_capacity(tuples.len());
    for _ in tuples {
        selectors.push(solver.new_var());
    }

    // At least one selector.
    solver.add_clause(
        &selectors
            .iter()
            .map(|s| Lit::from_var(*s, true))
            .collect::<Vec<_>>(),
    );
    // At most one selector (pairwise).
    for i in 0..selectors.len() {
        for j in (i + 1)..selectors.len() {
            solver.add_clause(&[
                Lit::from_var(selectors[i], false),
                Lit::from_var(selectors[j], false),
            ]);
        }
    }

    // Selector implies each cell's chosen value.
    for (sel, tup) in selectors.into_iter().zip(tuples.iter()) {
        for (pos, &v) in tup.iter().enumerate() {
            let idx = cells[pos];
            let row = idx / n;
            let col = idx % n;
            if v == 0 || (v as usize) > n {
                return false;
            }
            solver.add_clause(&[Lit::from_var(sel, false), map.lit(row, col, v as usize - 1)]);
        }
    }

    true
}

/// SAT-based uniqueness check for a full puzzle, currently supporting:
/// - Latin constraints
/// - Eq cages
/// - 2-cell Sub/Div cages (ruleset baseline)
///
/// Add/Mul cage encoding is intentionally staged; see `docs/sat_cage_encoding.md`.
pub fn puzzle_uniqueness_via_sat(puzzle: &Puzzle, rules: Ruleset) -> SatUniqueness {
    if !rules.sub_div_two_cell_only {
        return SatUniqueness::Multiple;
    }

    let n = puzzle.n as usize;
    trace!(n, cages = puzzle.cages.len(), "sat.encode.start");

    // If SAT encoding would be too large (tuple explosion), fall back to the native solver
    // which can still count solutions up to 2 with early exit.
    let native_fallback =
        || match count_solutions_up_to_with_deductions(puzzle, rules, DeductionTier::Hard, 2) {
            Ok(0) => SatUniqueness::Unsat,
            Ok(1) => SatUniqueness::Unique,
            Ok(_) => SatUniqueness::Multiple,
            Err(_) => SatUniqueness::Multiple,
        };

    // Start from a fresh solver and build the full encoding in one place.
    let mut solver = Solver::new();

    let map = LatinVarMap::new(&mut solver, n);
    map.add_latin_constraints(&mut solver);

    // Cage constraints (partial).
    for cage in &puzzle.cages {
        match cage.op {
            Op::Eq => {
                if !add_eq_cage_clauses(&mut solver, &map, cage) {
                    return SatUniqueness::Unsat;
                }
            }
            Op::Sub | Op::Div => {
                if rules.sub_div_two_cell_only && cage.cells.len() != 2 {
                    return SatUniqueness::Unsat;
                }
                if !add_two_cell_sub_div_cage_clauses(&mut solver, &map, cage) {
                    return SatUniqueness::Unsat;
                }
            }
            Op::Add | Op::Mul => {
                let Ok(maybe) = cage.valid_permutations(puzzle.n, rules, SAT_TUPLE_THRESHOLD)
                else {
                    return SatUniqueness::Unsat;
                };
                let Some(tuples) = maybe else {
                    trace!(
                        op = ?cage.op,
                        cells = cage.cells.len(),
                        threshold = SAT_TUPLE_THRESHOLD,
                        "sat.encode.tuple_overflow"
                    );
                    return native_fallback();
                };
                trace!(
                    op = ?cage.op,
                    cells = cage.cells.len(),
                    tuples = tuples.len(),
                    "sat.encode.tuples"
                );
                if !add_tuple_allowlist(&mut solver, &map, cage, &tuples) {
                    return SatUniqueness::Unsat;
                }
            }
        }
    }

    match solver.solve() {
        Ok(true) => {}
        Ok(false) => return SatUniqueness::Unsat,
        Err(_) => return SatUniqueness::Unsat,
    }

    let model = match solver.model() {
        Some(m) => m,
        None => return SatUniqueness::Unsat,
    };
    let blocking = match map.model_to_blocking_clause(&model) {
        Some(b) => b,
        None => return SatUniqueness::Unsat,
    };
    solver.add_clause(&blocking);
    match solver.solve() {
        Ok(true) => SatUniqueness::Multiple,
        Ok(false) => SatUniqueness::Unique,
        Err(_) => SatUniqueness::Unique,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DeductionTier;
    use crate::count_solutions_up_to_with_deductions;
    use kenken_core::format::sgt_desc::parse_keen_desc;
    use kenken_core::rules::Op;
    use kenken_core::{Cage, CellId, Puzzle};

    #[test]
    fn sat_cages_matches_solver_for_small_example() {
        let puzzle = parse_keen_desc(2, "b__,a3a3").unwrap();
        let rules = Ruleset::keen_baseline();
        assert_eq!(
            puzzle_uniqueness_via_sat(&puzzle, rules),
            SatUniqueness::Multiple
        );
    }

    #[test]
    fn sat_cages_reports_unique_for_fully_pinned_grid() {
        // 2x2 Latin square:
        // 1 2
        // 2 1
        let puzzle = Puzzle {
            n: 2,
            cages: vec![
                Cage {
                    cells: [CellId(0)].into_iter().collect(),
                    op: Op::Eq,
                    target: 1,
                },
                Cage {
                    cells: [CellId(1)].into_iter().collect(),
                    op: Op::Eq,
                    target: 2,
                },
                Cage {
                    cells: [CellId(2)].into_iter().collect(),
                    op: Op::Eq,
                    target: 2,
                },
                Cage {
                    cells: [CellId(3)].into_iter().collect(),
                    op: Op::Eq,
                    target: 1,
                },
            ],
        };
        let rules = Ruleset::keen_baseline();
        assert_eq!(
            puzzle_uniqueness_via_sat(&puzzle, rules),
            SatUniqueness::Unique
        );
    }

    #[test]
    fn sat_cages_reports_unsat_for_contradictory_eqs() {
        // Contradiction: row 0 has two 1s.
        let puzzle = Puzzle {
            n: 2,
            cages: vec![
                Cage {
                    cells: [CellId(0)].into_iter().collect(),
                    op: Op::Eq,
                    target: 1,
                },
                Cage {
                    cells: [CellId(1)].into_iter().collect(),
                    op: Op::Eq,
                    target: 1,
                },
                Cage {
                    cells: [CellId(2)].into_iter().collect(),
                    op: Op::Eq,
                    target: 2,
                },
                Cage {
                    cells: [CellId(3)].into_iter().collect(),
                    op: Op::Eq,
                    target: 2,
                },
            ],
        };
        let rules = Ruleset::keen_baseline();
        assert_eq!(
            puzzle_uniqueness_via_sat(&puzzle, rules),
            SatUniqueness::Unsat
        );
    }

    #[test]
    fn sat_cages_matches_solver_for_mixed_ops_unique_puzzle() {
        // A mostly pinned 4x4 puzzle with a few 2-cell cages (Add/Sub/Div).
        // The heavy pinning keeps the test fast and makes uniqueness unambiguous.
        let puzzle = Puzzle {
            n: 4,
            cages: vec![
                Cage {
                    cells: [CellId(0)].into_iter().collect(),
                    op: Op::Eq,
                    target: 1,
                },
                Cage {
                    cells: [CellId(1)].into_iter().collect(),
                    op: Op::Eq,
                    target: 2,
                },
                Cage {
                    cells: [CellId(2), CellId(3)].into_iter().collect(),
                    op: Op::Add,
                    target: 7,
                },
                Cage {
                    cells: [CellId(4), CellId(8)].into_iter().collect(),
                    op: Op::Sub,
                    target: 1,
                },
                Cage {
                    cells: [CellId(5)].into_iter().collect(),
                    op: Op::Eq,
                    target: 3,
                },
                Cage {
                    cells: [CellId(6)].into_iter().collect(),
                    op: Op::Eq,
                    target: 4,
                },
                Cage {
                    cells: [CellId(7), CellId(11)].into_iter().collect(),
                    op: Op::Div,
                    target: 2,
                },
                Cage {
                    cells: [CellId(9)].into_iter().collect(),
                    op: Op::Eq,
                    target: 4,
                },
                Cage {
                    cells: [CellId(10)].into_iter().collect(),
                    op: Op::Eq,
                    target: 1,
                },
                Cage {
                    cells: [CellId(12)].into_iter().collect(),
                    op: Op::Eq,
                    target: 4,
                },
                Cage {
                    cells: [CellId(13)].into_iter().collect(),
                    op: Op::Eq,
                    target: 1,
                },
                Cage {
                    cells: [CellId(14)].into_iter().collect(),
                    op: Op::Eq,
                    target: 2,
                },
                Cage {
                    cells: [CellId(15)].into_iter().collect(),
                    op: Op::Eq,
                    target: 3,
                },
            ],
        };
        let rules = Ruleset::keen_baseline();
        puzzle.validate(rules).unwrap();

        let native =
            count_solutions_up_to_with_deductions(&puzzle, rules, DeductionTier::Hard, 2).unwrap();
        assert_eq!(native, 1);
        assert_eq!(
            puzzle_uniqueness_via_sat(&puzzle, rules),
            SatUniqueness::Unique
        );
    }
}
