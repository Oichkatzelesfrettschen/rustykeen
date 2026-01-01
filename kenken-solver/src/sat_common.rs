//! Shared SAT helpers (Varisat) for Latin-square style encodings.
//!
//! This module centralizes:
//! - variable mapping `X(r,c,v)`
//! - Latin constraints
//! - model-to-blocking-clause extraction (ignoring auxiliary vars)
//!
//! It is `sat-varisat`-only by construction (module is only compiled when enabled).

use varisat::{ExtendFormula, Lit, Solver, Var};

#[derive(Debug, Clone)]
pub struct LatinVarMap {
    n: usize,
    vars: Vec<Var>,
}

impl LatinVarMap {
    pub fn new(solver: &mut Solver, n: usize) -> Self {
        let a = n * n;
        let mut vars = Vec::with_capacity(a * n);
        for _ in 0..(a * n) {
            vars.push(solver.new_var());
        }
        Self { n, vars }
    }

    pub fn n(&self) -> usize {
        self.n
    }

    pub fn vars(&self) -> &[Var] {
        &self.vars
    }

    fn var_idx(&self, row: usize, col: usize, val0: usize) -> usize {
        (row * self.n + col) * self.n + val0
    }

    pub fn lit(&self, row: usize, col: usize, val0: usize) -> Lit {
        Lit::from_var(self.vars[self.var_idx(row, col, val0)], true)
    }

    pub fn nlit(&self, row: usize, col: usize, val0: usize) -> Lit {
        Lit::from_var(self.vars[self.var_idx(row, col, val0)], false)
    }

    /// Add Latin constraints:
    /// - exactly one value per cell
    /// - row uniqueness
    /// - column uniqueness
    pub fn add_latin_constraints(&self, solver: &mut Solver) {
        let n = self.n;

        // Exactly one value per cell (pairwise at-most-one).
        for row in 0..n {
            for col in 0..n {
                let mut atleast = Vec::with_capacity(n);
                for val0 in 0..n {
                    atleast.push(self.lit(row, col, val0));
                }
                solver.add_clause(&atleast);
                for v1 in 0..n {
                    for v2 in (v1 + 1)..n {
                        solver.add_clause(&[self.nlit(row, col, v1), self.nlit(row, col, v2)]);
                    }
                }
            }
        }

        // Row uniqueness: no digit repeats in a row.
        for row in 0..n {
            for val0 in 0..n {
                for c1 in 0..n {
                    for c2 in (c1 + 1)..n {
                        solver.add_clause(&[self.nlit(row, c1, val0), self.nlit(row, c2, val0)]);
                    }
                }
            }
        }

        // Col uniqueness: no digit repeats in a column.
        for col in 0..n {
            for val0 in 0..n {
                for r1 in 0..n {
                    for r2 in (r1 + 1)..n {
                        solver.add_clause(&[self.nlit(r1, col, val0), self.nlit(r2, col, val0)]);
                    }
                }
            }
        }
    }

    pub fn add_givens_or_unsat(&self, solver: &mut Solver, givens: &[u8]) -> bool {
        let n = self.n;
        let a = n * n;
        if givens.len() != a {
            return false;
        }
        for row in 0..n {
            for col in 0..n {
                let given = givens[row * n + col];
                if given == 0 {
                    continue;
                }
                if given as usize > n {
                    return false;
                }
                solver.add_clause(&[self.lit(row, col, given as usize - 1)]);
            }
        }
        true
    }

    /// Build a clause that blocks the current Latin assignment, ignoring auxiliary vars.
    pub fn model_to_blocking_clause(&self, model: &[Lit]) -> Option<Vec<Lit>> {
        let n = self.n;
        let a = n * n;

        let mut assignment = vec![false; self.vars.len()];
        for lit in model {
            let idx = lit.var().index();
            if idx < assignment.len() {
                assignment[idx] = lit.is_positive();
            }
        }

        let mut blocking = Vec::with_capacity(a);
        for row in 0..n {
            for col in 0..n {
                let mut chosen = None;
                for val0 in 0..n {
                    let v = self.vars[self.var_idx(row, col, val0)];
                    if assignment[v.index()] {
                        chosen = Some(val0);
                        break;
                    }
                }
                let val0 = chosen?;
                blocking.push(self.nlit(row, col, val0));
            }
        }
        Some(blocking)
    }
}
