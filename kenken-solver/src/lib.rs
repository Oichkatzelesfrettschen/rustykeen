#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

#[cfg(feature = "solver-dlx")]
pub mod dlx_latin;
pub mod error;
#[cfg(feature = "sat-varisat")]
pub mod sat_cages;
#[cfg(feature = "sat-varisat")]
pub mod sat_common;
#[cfg(feature = "sat-varisat")]
pub mod sat_latin;
pub mod solver;

pub use crate::solver::{
    DeductionTier, DifficultyTier, Solution, SolveStats, count_solutions_up_to,
    count_solutions_up_to_with_deductions, solve_one, solve_one_with_deductions,
    solve_one_with_stats,
};
