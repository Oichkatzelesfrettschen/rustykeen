//! Latin-square SAT encoding utilities using Varisat.
//!
//! Current scope is Latin constraints only; cage arithmetic constraints are a follow-up.
//!
use varisat::{ExtendFormula, Solver};

use crate::sat_common::LatinVarMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SatUniqueness {
    Unsat,
    Unique,
    Multiple,
}

pub fn latin_uniqueness_via_sat(n: u8, givens: &[u8]) -> SatUniqueness {
    let n_usize = n as usize;
    let a = n_usize * n_usize;
    assert_eq!(givens.len(), a);

    let mut solver = Solver::new();
    let map = LatinVarMap::new(&mut solver, n_usize);
    map.add_latin_constraints(&mut solver);

    // Givens.
    if !map.add_givens_or_unsat(&mut solver, givens) {
        return SatUniqueness::Unsat;
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

    #[test]
    fn sat_latin_2x2_unique_under_given() {
        let mut givens = [0u8; 4];
        givens[0] = 1;
        assert_eq!(latin_uniqueness_via_sat(2, &givens), SatUniqueness::Unique);
    }
}
