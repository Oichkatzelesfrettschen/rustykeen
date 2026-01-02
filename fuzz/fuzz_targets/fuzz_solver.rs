#![no_main]

//! Fuzz target for KenKen solver.
//!
//! Tests that the solver handles arbitrary (potentially invalid) puzzles
//! without panicking or hitting undefined behavior.

use libfuzzer_sys::fuzz_target;

use arbitrary::Arbitrary;
use kenken_core::puzzle::{Cage, CellId, Puzzle};
use kenken_core::rules::{Op, Ruleset};
use kenken_solver::{count_solutions_up_to_with_deductions, solve_one_with_deductions, DeductionTier};
use smallvec::SmallVec;

/// Arbitrary input for generating puzzle-like structures.
#[derive(Arbitrary, Debug)]
struct FuzzPuzzle {
    n: u8,
    cages: Vec<FuzzCage>,
}

#[derive(Arbitrary, Debug)]
struct FuzzCage {
    cells: Vec<u16>,
    op: u8,
    target: i32,
}

impl FuzzPuzzle {
    fn to_puzzle(&self) -> Option<Puzzle> {
        // Constrain grid size to reasonable range
        let n = self.n.clamp(2, 9);

        // Convert cages
        let cages: Vec<Cage> = self
            .cages
            .iter()
            .filter_map(|fc| {
                // Skip empty cages
                if fc.cells.is_empty() {
                    return None;
                }

                // Convert cells, clamping to valid range
                let max_cell = (n as u16) * (n as u16);
                let cells: SmallVec<[CellId; 6]> = fc
                    .cells
                    .iter()
                    .take(6) // Max 6 cells per cage
                    .map(|&c| CellId(c % max_cell))
                    .collect();

                if cells.is_empty() {
                    return None;
                }

                // Map op byte to Op enum
                let op = match fc.op % 5 {
                    0 => Op::Add,
                    1 => Op::Mul,
                    2 => Op::Sub,
                    3 => Op::Div,
                    _ => Op::Eq,
                };

                Some(Cage {
                    cells,
                    op,
                    target: fc.target,
                })
            })
            .collect();

        if cages.is_empty() {
            return None;
        }

        Some(Puzzle { n, cages })
    }
}

fuzz_target!(|data: FuzzPuzzle| {
    let Some(puzzle) = data.to_puzzle() else {
        return;
    };

    let rules = Ruleset::keen_baseline();

    // Validation should not panic
    let valid = puzzle.validate(rules).is_ok();

    // Only run solver on valid puzzles to avoid wasting cycles
    if valid {
        // Solve should not panic
        let _ = solve_one_with_deductions(&puzzle, rules, DeductionTier::Normal);

        // Count should not panic (with small limit to bound runtime)
        let _ = count_solutions_up_to_with_deductions(&puzzle, rules, DeductionTier::Normal, 2);
    }
});
