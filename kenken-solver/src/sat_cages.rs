//! Cage constraint SAT encoding utilities (Varisat).
//!
//! This module is a staging area for extending SAT support from Latin-only
//! (`sat_latin`) to full KenKen cage arithmetic. See `docs/sat_cage_encoding.md`.
//!
//! Internal implementation module. Not part of the public API.

#![allow(dead_code)]

use kenken_core::rules::{Op, Ruleset};
use kenken_core::{Cage, Puzzle};
use smallvec::SmallVec;
use varisat::{CnfFormula, ExtendFormula, Lit, Solver, Var, dimacs};

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
/// Chosen as 512 based on the following considerations:
///
/// - **Clause count scaling**: For T tuples in a k-cell cage, we need T selector vars,
///   2*k*T implication clauses, and T*(T-1)/2 at-most-one clauses. At T=512 with k=4,
///   this is ~135k clauses per cage.
///
/// - **Practical bounds**: For N<=9 and cages<=6 cells, typical tuple counts are:
///   - 2-cell Add: max 8 tuples
///   - 3-cell Add: max ~80 tuples
///   - 4-cell Add: hundreds (near threshold)
///   - 5+ cell Mul: may exceed threshold
///
/// - **Fallback cost**: When exceeded, `count_solutions_up_to(..., limit=2)` is used,
///   which is fast for small puzzles where tuple explosion is unlikely anyway.
///
/// See `docs/sat_cage_encoding.md` section 3.4 for detailed justification.
pub const SAT_TUPLE_THRESHOLD: usize = 512;

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum SatCertificationError {
    #[error("SAT cage encoding requires rules.sub_div_two_cell_only=true")]
    UnsupportedRuleset,
    #[error(
        "SAT strict certification rejected tuple overflow in cage {cage_index} ({op:?}, {cells} cells), threshold={threshold}"
    )]
    TupleOverflow {
        cage_index: usize,
        op: Op,
        cells: usize,
        threshold: usize,
    },
    #[error("SAT cage encoding failed for cage {cage_index} ({op:?}, {cells} cells): {reason}")]
    InvalidEncoding {
        cage_index: usize,
        op: Op,
        cells: usize,
        reason: &'static str,
    },
    #[error("DIMACS export failed: {0}")]
    DimacsExport(String),
}

fn add_eq_cage_clauses(target: &mut impl ExtendFormula, map: &LatinVarMap, cage: &Cage) -> bool {
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
    target.add_clause(&[map.lit(row, col, cage.target as usize - 1)]);
    true
}

fn allowed_sub_pair(a: u8, b: u8, target: i32) -> bool {
    (a as i32 - b as i32).abs() == target
}

fn allowed_div_pair(a: u8, b: u8, target: i32) -> bool {
    let (num, den) = if a >= b { (a, b) } else { (b, a) };
    den != 0 && (num as i32) == (den as i32).saturating_mul(target)
}

fn add_two_cell_sub_div_cage_clauses(
    target: &mut impl ExtendFormula,
    map: &LatinVarMap,
    cage: &Cage,
) -> bool {
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
            let s = target.new_var();
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
    target.add_clause(
        &selectors
            .iter()
            .map(|(s, _, _)| Lit::from_var(*s, true))
            .collect::<Vec<_>>(),
    );
    // At most one selector (pairwise).
    for i in 0..selectors.len() {
        for j in (i + 1)..selectors.len() {
            target.add_clause(&[
                Lit::from_var(selectors[i].0, false),
                Lit::from_var(selectors[j].0, false),
            ]);
        }
    }
    // Selector implies assignments.
    for (s, av, bv) in selectors {
        target.add_clause(&[Lit::from_var(s, false), map.lit(ar, ac, av as usize - 1)]);
        target.add_clause(&[Lit::from_var(s, false), map.lit(br, bc, bv as usize - 1)]);
    }

    true
}

fn add_tuple_allowlist(
    target: &mut impl ExtendFormula,
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
        selectors.push(target.new_var());
    }

    // At least one selector.
    target.add_clause(
        &selectors
            .iter()
            .map(|s| Lit::from_var(*s, true))
            .collect::<Vec<_>>(),
    );
    // At most one selector (pairwise).
    for i in 0..selectors.len() {
        for j in (i + 1)..selectors.len() {
            target.add_clause(&[
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
            target.add_clause(&[Lit::from_var(sel, false), map.lit(row, col, v as usize - 1)]);
        }
    }

    true
}

fn native_uniqueness_fallback(puzzle: &Puzzle, rules: Ruleset) -> SatUniqueness {
    match count_solutions_up_to_with_deductions(puzzle, rules, DeductionTier::Hard, 2) {
        Ok(0) => SatUniqueness::Unsat,
        Ok(1) => SatUniqueness::Unique,
        Ok(_) => SatUniqueness::Multiple,
        Err(_) => SatUniqueness::Multiple,
    }
}

fn encode_puzzle_constraints(
    target: &mut impl ExtendFormula,
    puzzle: &Puzzle,
    rules: Ruleset,
) -> Result<LatinVarMap, SatCertificationError> {
    if !rules.sub_div_two_cell_only {
        return Err(SatCertificationError::UnsupportedRuleset);
    }

    let n = puzzle.n as usize;
    trace!(n, cages = puzzle.cages.len(), "sat.encode.start");
    let map = LatinVarMap::new(target, n);
    map.add_latin_constraints(target);

    for (cage_index, cage) in puzzle.cages.iter().enumerate() {
        match cage.op {
            Op::Eq => {
                if !add_eq_cage_clauses(target, &map, cage) {
                    return Err(SatCertificationError::InvalidEncoding {
                        cage_index,
                        op: cage.op,
                        cells: cage.cells.len(),
                        reason: "invalid Eq cage",
                    });
                }
            }
            Op::Sub | Op::Div => {
                if rules.sub_div_two_cell_only && cage.cells.len() != 2 {
                    return Err(SatCertificationError::InvalidEncoding {
                        cage_index,
                        op: cage.op,
                        cells: cage.cells.len(),
                        reason: "Sub/Div cage must have exactly two cells",
                    });
                }
                if !add_two_cell_sub_div_cage_clauses(target, &map, cage) {
                    return Err(SatCertificationError::InvalidEncoding {
                        cage_index,
                        op: cage.op,
                        cells: cage.cells.len(),
                        reason: "invalid Sub/Div cage clauses",
                    });
                }
            }
            Op::Add | Op::Mul => {
                let maybe = cage
                    .valid_permutations(puzzle.n, rules, SAT_TUPLE_THRESHOLD)
                    .map_err(|_| SatCertificationError::InvalidEncoding {
                        cage_index,
                        op: cage.op,
                        cells: cage.cells.len(),
                        reason: "tuple generation failed",
                    })?;

                let Some(tuples) = maybe else {
                    trace!(
                        op = ?cage.op,
                        cells = cage.cells.len(),
                        threshold = SAT_TUPLE_THRESHOLD,
                        "sat.encode.tuple_overflow"
                    );
                    return Err(SatCertificationError::TupleOverflow {
                        cage_index,
                        op: cage.op,
                        cells: cage.cells.len(),
                        threshold: SAT_TUPLE_THRESHOLD,
                    });
                };
                trace!(
                    op = ?cage.op,
                    cells = cage.cells.len(),
                    tuples = tuples.len(),
                    "sat.encode.tuples"
                );
                if !add_tuple_allowlist(target, &map, cage, &tuples) {
                    return Err(SatCertificationError::InvalidEncoding {
                        cage_index,
                        op: cage.op,
                        cells: cage.cells.len(),
                        reason: "tuple allowlist encoding failed",
                    });
                }
            }
        }
    }

    Ok(map)
}

fn solve_uniqueness(mut solver: Solver, map: &LatinVarMap) -> SatUniqueness {
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

/// Strict SAT-based uniqueness check.
///
/// Unlike `puzzle_uniqueness_via_sat`, this function fails closed if tuple
/// overflow would require a native fallback, preserving independent SAT proof
/// semantics for certification use-cases.
pub fn puzzle_uniqueness_via_sat_strict(
    puzzle: &Puzzle,
    rules: Ruleset,
) -> Result<SatUniqueness, SatCertificationError> {
    let mut solver = Solver::new();
    let map = encode_puzzle_constraints(&mut solver, puzzle, rules)?;
    Ok(solve_uniqueness(solver, &map))
}

/// SAT-based uniqueness check for a full puzzle, currently supporting:
/// - Latin constraints
/// - Eq cages
/// - 2-cell Sub/Div cages (ruleset baseline)
///
/// Add/Mul cage encoding is intentionally staged; see `docs/sat_cage_encoding.md`.
///
/// This permissive variant may fall back to the native solver if tuple
/// explosion exceeds `SAT_TUPLE_THRESHOLD`.
pub fn puzzle_uniqueness_via_sat(puzzle: &Puzzle, rules: Ruleset) -> SatUniqueness {
    match puzzle_uniqueness_via_sat_strict(puzzle, rules) {
        Ok(result) => result,
        Err(SatCertificationError::TupleOverflow { .. }) => {
            native_uniqueness_fallback(puzzle, rules)
        }
        Err(SatCertificationError::UnsupportedRuleset) => SatUniqueness::Multiple,
        Err(SatCertificationError::InvalidEncoding { .. }) => SatUniqueness::Unsat,
        Err(SatCertificationError::DimacsExport(_)) => SatUniqueness::Unsat,
    }
}

/// Build a strict SAT CNF formula for external solvers.
///
/// This fails closed on tuple overflow and does not apply native fallbacks.
pub fn export_puzzle_cnf_strict(
    puzzle: &Puzzle,
    rules: Ruleset,
) -> Result<CnfFormula, SatCertificationError> {
    let mut formula = CnfFormula::new();
    let _ = encode_puzzle_constraints(&mut formula, puzzle, rules)?;
    Ok(formula)
}

/// Export a puzzle as DIMACS CNF text for external SAT solvers.
pub fn export_puzzle_dimacs_strict(
    puzzle: &Puzzle,
    rules: Ruleset,
) -> Result<String, SatCertificationError> {
    let formula = export_puzzle_cnf_strict(puzzle, rules)?;
    let mut out = Vec::new();
    dimacs::write_dimacs(&mut out, &formula)
        .map_err(|e| SatCertificationError::DimacsExport(e.to_string()))?;
    String::from_utf8(out).map_err(|e| SatCertificationError::DimacsExport(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DeductionTier;
    use crate::count_solutions_up_to_with_deductions;
    use kenken_core::format::sgt_desc::parse_keen_desc;
    use kenken_core::rules::Op;
    use kenken_core::{Cage, CellId, Puzzle};
    use varisat::{ExtendFormula, Lit, Solver, Var};

    fn classify_dimacs_uniqueness(dimacs_text: &str, n: u8) -> SatUniqueness {
        let mut solver = Solver::new();
        solver.add_dimacs_cnf(dimacs_text.as_bytes()).unwrap();
        match solver.solve() {
            Ok(true) => {}
            Ok(false) => return SatUniqueness::Unsat,
            Err(_) => return SatUniqueness::Unsat,
        }

        let n_usize = n as usize;
        let latin_var_count = n_usize * n_usize * n_usize;
        let model = solver.model().unwrap();
        let mut assignment = vec![false; latin_var_count];
        for lit in model {
            let idx = lit.var().index();
            if idx < latin_var_count {
                assignment[idx] = lit.is_positive();
            }
        }

        let mut blocking = Vec::with_capacity(n_usize * n_usize);
        for row in 0..n_usize {
            for col in 0..n_usize {
                let mut chosen_var_idx = None;
                for val0 in 0..n_usize {
                    let idx = (row * n_usize + col) * n_usize + val0;
                    if assignment[idx] {
                        chosen_var_idx = Some(idx);
                        break;
                    }
                }
                let Some(var_idx) = chosen_var_idx else {
                    return SatUniqueness::Unsat;
                };
                blocking.push(Lit::from_var(Var::from_index(var_idx), false));
            }
        }

        solver.add_clause(&blocking);
        match solver.solve() {
            Ok(true) => SatUniqueness::Multiple,
            Ok(false) => SatUniqueness::Unique,
            Err(_) => SatUniqueness::Unique,
        }
    }

    fn mk_tuple_overflow_puzzle() -> Puzzle {
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

        Puzzle { n, cages }
    }

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

    #[test]
    fn sat_strict_mode_rejects_tuple_overflow() {
        let puzzle = mk_tuple_overflow_puzzle();
        let rules = Ruleset::keen_baseline();
        puzzle.validate(rules).unwrap();
        let err = puzzle_uniqueness_via_sat_strict(&puzzle, rules).unwrap_err();
        assert!(matches!(err, SatCertificationError::TupleOverflow { .. }));
    }

    #[test]
    fn sat_permissive_mode_falls_back_when_tuple_overflows() {
        let puzzle = mk_tuple_overflow_puzzle();
        let rules = Ruleset::keen_baseline();
        puzzle.validate(rules).unwrap();
        assert_eq!(
            puzzle_uniqueness_via_sat(&puzzle, rules),
            SatUniqueness::Unique
        );
    }

    #[test]
    fn dimacs_export_matches_strict_solver_uniqueness() {
        let puzzle = parse_keen_desc(2, "b__,a3a3").unwrap();
        let rules = Ruleset::keen_baseline();
        let dimacs = export_puzzle_dimacs_strict(&puzzle, rules).unwrap();
        assert!(dimacs.starts_with("p cnf "));
        let dimacs_result = classify_dimacs_uniqueness(&dimacs, puzzle.n);
        let strict_result = puzzle_uniqueness_via_sat_strict(&puzzle, rules).unwrap();
        assert_eq!(dimacs_result, strict_result);
    }
}
