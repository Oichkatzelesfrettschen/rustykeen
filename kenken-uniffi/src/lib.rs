// UniFFI generates the FFI surface (including `#[no_mangle]` symbols), which
// requires allowing unsafe code in this adapter crate.
#![allow(unsafe_code)]
#![doc = include_str!("../README.md")]

use kenken_core::format::sgt_desc::encode_keen_desc;
use kenken_core::format::sgt_desc::parse_keen_desc;
use kenken_core::rules::Ruleset;
use kenken_solver::{count_solutions_up_to_with_deductions, solve_one_with_deductions};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeductionTier {
    None,
    Easy,
    Normal,
    Hard,
}

impl From<DeductionTier> for kenken_solver::DeductionTier {
    fn from(t: DeductionTier) -> Self {
        match t {
            DeductionTier::None => kenken_solver::DeductionTier::None,
            DeductionTier::Easy => kenken_solver::DeductionTier::Easy,
            DeductionTier::Normal => kenken_solver::DeductionTier::Normal,
            DeductionTier::Hard => kenken_solver::DeductionTier::Hard,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid {
    pub n: u8,
    pub cells: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Generated {
    pub desc: String,
    pub solution: Grid,
}

pub fn solve_sgt_desc(n: u8, desc: String, tier: DeductionTier) -> Option<Grid> {
    let puzzle = parse_keen_desc(n, &desc).ok()?;
    let solution =
        solve_one_with_deductions(&puzzle, Ruleset::keen_baseline(), tier.into()).ok()?;
    let solution = solution?;
    Some(Grid {
        n: solution.n,
        cells: solution.grid,
    })
}

pub fn generate_sgt_desc(n: u8, seed: u64, tier: DeductionTier) -> Option<Generated> {
    #[cfg(feature = "gen")]
    {
        let cfg = kenken_gen::generator::GenerateConfig {
            tier: tier.into(),
            ..kenken_gen::generator::GenerateConfig::keen_baseline(n, seed)
        };
        let g = kenken_gen::generator::generate(cfg).ok()?;
        let desc = encode_keen_desc(&g.puzzle, Ruleset::keen_baseline()).ok()?;
        Some(Generated {
            desc,
            solution: Grid {
                n: g.puzzle.n,
                cells: g.solution,
            },
        })
    }

    #[cfg(not(feature = "gen"))]
    {
        let _ = (n, seed, tier);
        None
    }
}

pub fn count_solutions_sgt_desc(n: u8, desc: String, tier: DeductionTier, limit: u32) -> u32 {
    let Ok(puzzle) = parse_keen_desc(n, &desc) else {
        return 0;
    };
    count_solutions_up_to_with_deductions(&puzzle, Ruleset::keen_baseline(), tier.into(), limit)
        .unwrap_or(0)
}

uniffi::include_scaffolding!("keen");
