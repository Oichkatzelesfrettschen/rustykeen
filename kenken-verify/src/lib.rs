#![forbid(unsafe_code)]
#![deny(warnings)]

//! Formal Verification Crate for KenKen Solver
//!
//! This crate provides verified implementations of KenKen solver algorithms
//! using Rocq (Coq 9.1.0) formal proofs with extraction to Rust.
//!
//! # Architecture
//!
//! - `rcoq/` directory contains Rocq formalization (porting approach)
//! - `verified_solver.rs` contains extracted and manually verified implementations
//! - `z3_interface.rs` provides axiomatized Z3 verification
//! - `sat_interface.rs` provides SAT solver agreement verification

pub mod verified_solver;

#[cfg(feature = "verify-z3")]
pub mod z3_interface;

#[cfg(feature = "verify-sat")]
pub mod sat_interface;

#[cfg(not(feature = "verify-z3"))]
mod z3_interface {
    //! Stub for when verify-z3 feature is disabled
}

#[cfg(not(feature = "verify-sat"))]
mod sat_interface {
    //! Stub for when verify-sat feature is disabled
}

/// Public API: Verify a solution against a puzzle using extracted proofs
///
/// Returns Ok(()) if solution is valid, Err with description if invalid.
pub fn verify_solution(puzzle: &kenken_core::Puzzle, solution: &[u8]) -> Result<(), String> {
    verified_solver::verify_solution(puzzle, solution)
}

/// Public API: Count solutions up to a limit using verified counting
pub fn count_solutions_up_to(puzzle: &kenken_core::Puzzle, limit: usize) -> Result<usize, String> {
    verified_solver::count_solutions_up_to(puzzle, limit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_solution_stub() {
        // Placeholder: Will be expanded with corpus tests
        let result = verify_solution;
        let _r = result;
    }
}
