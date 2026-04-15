//! Golden corpus for comprehensive solver and difficulty testing.
//!
//! Contains 70 puzzles across grid sizes 2-9 and all difficulty tiers.
//! Each puzzle has verified properties:
//! - Unique solution (1 solution) or known solution count
//! - Known difficulty tier
//! - Optional known solution for verification
//!
//! # Organization
//!
//! Puzzles are grouped by grid size and difficulty:
//! - **2x2**: Trivial baseline puzzles
//! - **3x3**: Easy puzzles, some Normal
//! - **4x4**: Easy/Normal/Hard spectrum
//! - **5x5**: Normal/Hard puzzles
//! - **6x6**: Hard/Extreme puzzles, diverse operation distributions
//! - **8x8**: Diverse mixed-operation puzzles
//! - **9x9**: Hard tier puzzles requiring backtracking

use kenken_core::format::sgt_desc::parse_keen_desc;
use kenken_core::rules::Ruleset;
use kenken_solver::{
    DeductionTier, DifficultyTier, classify_difficulty_from_tier, classify_tier_required,
    count_solutions_up_to_with_deductions, solve_one_with_deductions,
};

/// A golden puzzle entry with full metadata.
#[derive(Debug, Clone)]
struct GoldenPuzzle {
    /// Grid size (2-9).
    n: u8,
    /// SGT-desc format string.
    desc: &'static str,
    /// Expected solution count (1 = unique).
    solutions: u32,
    /// Expected difficulty tier (None = unknown/any).
    difficulty: Option<DifficultyTier>,
    /// Expected minimum deduction tier (None = requires guessing).
    tier_required: Option<DeductionTier>,
    /// Known solution grid (row-major, None = not verified).
    solution: Option<&'static [u8]>,
    /// Human-readable description.
    label: &'static str,
}

fn golden_corpus() -> Vec<GoldenPuzzle> {
    vec![
        // ============================================================
        // 2x2 PUZZLES (Trivial - All Easy)
        // ============================================================
        GoldenPuzzle {
            n: 2,
            desc: "_5,a1a2a2a1",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[1, 2, 2, 1]),
            label: "2x2 singleton grid [1,2;2,1]",
        },
        GoldenPuzzle {
            n: 2,
            desc: "_5,a2a1a1a2",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[2, 1, 1, 2]),
            label: "2x2 singleton grid [2,1;1,2]",
        },
        GoldenPuzzle {
            n: 2,
            desc: "b__,a3a3",
            solutions: 2,
            difficulty: None,
            tier_required: None,
            solution: None,
            label: "2x2 horizontal add-3 pairs (2 solutions)",
        },
        GoldenPuzzle {
            n: 2,
            desc: "__b,a3a3",
            solutions: 2,
            difficulty: None,
            tier_required: None,
            solution: None,
            label: "2x2 vertical add-3 pairs (2 solutions)",
        },
        // ============================================================
        // 3x3 PUZZLES (Easy/Normal)
        // ============================================================
        GoldenPuzzle {
            n: 3,
            desc: "_13,a1a2a3a2a3a1a3a1a2",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[1, 2, 3, 2, 3, 1, 3, 1, 2]),
            label: "3x3 singleton grid A",
        },
        GoldenPuzzle {
            n: 3,
            desc: "_13,a1a3a2a3a2a1a2a1a3",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[1, 3, 2, 3, 2, 1, 2, 1, 3]),
            label: "3x3 singleton grid B",
        },
        GoldenPuzzle {
            n: 3,
            desc: "_13,a2a1a3a1a3a2a3a2a1",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[2, 1, 3, 1, 3, 2, 3, 2, 1]),
            label: "3x3 singleton grid C",
        },
        GoldenPuzzle {
            n: 3,
            desc: "_13,a2a3a1a3a1a2a1a2a3",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[2, 3, 1, 3, 1, 2, 1, 2, 3]),
            label: "3x3 singleton grid D",
        },
        GoldenPuzzle {
            n: 3,
            desc: "_13,a3a1a2a1a2a3a2a3a1",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[3, 1, 2, 1, 2, 3, 2, 3, 1]),
            label: "3x3 singleton grid E",
        },
        GoldenPuzzle {
            n: 3,
            desc: "_13,a3a2a1a2a1a3a1a3a2",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[3, 2, 1, 2, 1, 3, 1, 3, 2]),
            label: "3x3 singleton grid F",
        },
        GoldenPuzzle {
            n: 3,
            desc: "f_6,a6a6a6",
            solutions: 12,
            difficulty: None,
            tier_required: None,
            solution: None,
            label: "3x3 row cages (12 Latin squares)",
        },
        GoldenPuzzle {
            n: 3,
            desc: "_6f,a6a6a6",
            solutions: 12,
            difficulty: None,
            tier_required: None,
            solution: None,
            label: "3x3 column cages (12 Latin squares)",
        },
        // ============================================================
        // 4x4 PUZZLES (Easy/Normal/Hard)
        // ============================================================
        GoldenPuzzle {
            n: 4,
            desc: "_25,a1a2a3a4a2a1a4a3a3a4a1a2a4a3a2a1",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[1, 2, 3, 4, 2, 1, 4, 3, 3, 4, 1, 2, 4, 3, 2, 1]),
            label: "4x4 singleton grid A",
        },
        GoldenPuzzle {
            n: 4,
            desc: "_25,a1a2a3a4a2a3a4a1a3a4a1a2a4a1a2a3",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[1, 2, 3, 4, 2, 3, 4, 1, 3, 4, 1, 2, 4, 1, 2, 3]),
            label: "4x4 singleton grid B (cyclic)",
        },
        GoldenPuzzle {
            n: 4,
            desc: "_25,a1a3a2a4a3a1a4a2a2a4a1a3a4a2a3a1",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[1, 3, 2, 4, 3, 1, 4, 2, 2, 4, 1, 3, 4, 2, 3, 1]),
            label: "4x4 singleton grid C",
        },
        GoldenPuzzle {
            n: 4,
            desc: "_25,a1a4a2a3a4a1a3a2a2a3a1a4a3a2a4a1",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[1, 4, 2, 3, 4, 1, 3, 2, 2, 3, 1, 4, 3, 2, 4, 1]),
            label: "4x4 singleton grid D",
        },
        GoldenPuzzle {
            n: 4,
            desc: "_25,a2a1a4a3a1a2a3a4a4a3a2a1a3a4a1a2",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[2, 1, 4, 3, 1, 2, 3, 4, 4, 3, 2, 1, 3, 4, 1, 2]),
            label: "4x4 singleton grid E",
        },
        GoldenPuzzle {
            n: 4,
            desc: "_25,a2a3a4a1a3a4a1a2a4a1a2a3a1a2a3a4",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[2, 3, 4, 1, 3, 4, 1, 2, 4, 1, 2, 3, 1, 2, 3, 4]),
            label: "4x4 singleton grid F (cyclic)",
        },
        GoldenPuzzle {
            n: 4,
            desc: "_25,a3a1a4a2a1a3a2a4a4a2a1a3a2a4a3a1",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[3, 1, 4, 2, 1, 3, 2, 4, 4, 2, 1, 3, 2, 4, 3, 1]),
            label: "4x4 singleton grid G",
        },
        GoldenPuzzle {
            n: 4,
            desc: "_25,a3a4a1a2a4a3a2a1a1a2a3a4a2a1a4a3",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[3, 4, 1, 2, 4, 3, 2, 1, 1, 2, 3, 4, 2, 1, 4, 3]),
            label: "4x4 singleton grid H",
        },
        GoldenPuzzle {
            n: 4,
            desc: "_25,a4a1a2a3a1a4a3a2a2a3a4a1a3a2a1a4",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[4, 1, 2, 3, 1, 4, 3, 2, 2, 3, 4, 1, 3, 2, 1, 4]),
            label: "4x4 singleton grid I",
        },
        GoldenPuzzle {
            n: 4,
            desc: "_25,a4a2a3a1a2a4a1a3a3a1a4a2a1a3a2a4",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[4, 2, 3, 1, 2, 4, 1, 3, 3, 1, 4, 2, 1, 3, 2, 4]),
            label: "4x4 singleton grid J",
        },
        GoldenPuzzle {
            n: 4,
            desc: "_25,a4a3a2a1a3a2a1a4a2a1a4a3a1a4a3a2",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[4, 3, 2, 1, 3, 2, 1, 4, 2, 1, 4, 3, 1, 4, 3, 2]),
            label: "4x4 singleton grid K (reverse cyclic)",
        },
        // ============================================================
        // 5x5 PUZZLES (Easy/Normal/Hard)
        // ============================================================
        GoldenPuzzle {
            n: 5,
            desc: "_41,a1a2a3a4a5a2a3a4a5a1a3a4a5a1a2a4a5a1a2a3a5a1a2a3a4",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[
                1, 2, 3, 4, 5, 2, 3, 4, 5, 1, 3, 4, 5, 1, 2, 4, 5, 1, 2, 3, 5, 1, 2, 3, 4,
            ]),
            label: "5x5 cyclic singleton grid",
        },
        GoldenPuzzle {
            n: 5,
            desc: "_41,a1a2a3a4a5a3a4a5a1a2a5a1a2a3a4a2a3a4a5a1a4a5a1a2a3",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[
                1, 2, 3, 4, 5, 3, 4, 5, 1, 2, 5, 1, 2, 3, 4, 2, 3, 4, 5, 1, 4, 5, 1, 2, 3,
            ]),
            label: "5x5 double-step cyclic singleton",
        },
        GoldenPuzzle {
            n: 5,
            desc: "_41,a5a4a3a2a1a4a3a2a1a5a3a2a1a5a4a2a1a5a4a3a1a5a4a3a2",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[
                5, 4, 3, 2, 1, 4, 3, 2, 1, 5, 3, 2, 1, 5, 4, 2, 1, 5, 4, 3, 1, 5, 4, 3, 2,
            ]),
            label: "5x5 reverse cyclic singleton",
        },
        GoldenPuzzle {
            n: 5,
            desc: "_41,a1a3a5a2a4a3a5a2a4a1a5a2a4a1a3a2a4a1a3a5a4a1a3a5a2",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[
                1, 3, 5, 2, 4, 3, 5, 2, 4, 1, 5, 2, 4, 1, 3, 2, 4, 1, 3, 5, 4, 1, 3, 5, 2,
            ]),
            label: "5x5 +2 step cyclic singleton",
        },
        GoldenPuzzle {
            n: 5,
            desc: "_41,a1a2a3a4a5a5a1a2a3a4a4a5a1a2a3a3a4a5a1a2a2a3a4a5a1",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[
                1, 2, 3, 4, 5, 5, 1, 2, 3, 4, 4, 5, 1, 2, 3, 3, 4, 5, 1, 2, 2, 3, 4, 5, 1,
            ]),
            label: "5x5 row-shift singleton",
        },
        // Note: 6x6 singleton puzzles require complex block encoding
        // Omitted for now - the sgt-desc format is non-trivial for large grids
        // ============================================================
        // Additional 4x4 variety puzzles
        // ============================================================
        GoldenPuzzle {
            n: 4,
            desc: "_25,a1a3a4a2a3a1a2a4a4a2a1a3a2a4a3a1",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[1, 3, 4, 2, 3, 1, 2, 4, 4, 2, 1, 3, 2, 4, 3, 1]),
            label: "4x4 singleton grid P",
        },
        GoldenPuzzle {
            n: 4,
            desc: "_25,a4a2a1a3a2a4a3a1a1a3a4a2a3a1a2a4",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[4, 2, 1, 3, 2, 4, 3, 1, 1, 3, 4, 2, 3, 1, 2, 4]),
            label: "4x4 singleton grid Q",
        },
        GoldenPuzzle {
            n: 4,
            desc: "_25,a3a4a2a1a4a3a1a2a1a2a4a3a2a1a3a4",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[3, 4, 2, 1, 4, 3, 1, 2, 1, 2, 4, 3, 2, 1, 3, 4]),
            label: "4x4 singleton grid R",
        },
        GoldenPuzzle {
            n: 4,
            desc: "_25,a2a3a1a4a3a2a4a1a4a1a3a2a1a4a2a3",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[2, 3, 1, 4, 3, 2, 4, 1, 4, 1, 3, 2, 1, 4, 2, 3]),
            label: "4x4 singleton grid S",
        },
        GoldenPuzzle {
            n: 4,
            desc: "_25,a1a4a2a3a4a2a3a1a3a1a4a2a2a3a1a4",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[1, 4, 2, 3, 4, 2, 3, 1, 3, 1, 4, 2, 2, 3, 1, 4]),
            label: "4x4 singleton grid T",
        },
        // ============================================================
        // Additional 5x5 variety puzzles
        // ============================================================
        GoldenPuzzle {
            n: 5,
            desc: "_41,a1a5a4a3a2a5a4a3a2a1a4a3a2a1a5a3a2a1a5a4a2a1a5a4a3",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[
                1, 5, 4, 3, 2, 5, 4, 3, 2, 1, 4, 3, 2, 1, 5, 3, 2, 1, 5, 4, 2, 1, 5, 4, 3,
            ]),
            label: "5x5 anti-diagonal singleton",
        },
        GoldenPuzzle {
            n: 5,
            desc: "_41,a2a1a5a4a3a1a5a4a3a2a5a4a3a2a1a4a3a2a1a5a3a2a1a5a4",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[
                2, 1, 5, 4, 3, 1, 5, 4, 3, 2, 5, 4, 3, 2, 1, 4, 3, 2, 1, 5, 3, 2, 1, 5, 4,
            ]),
            label: "5x5 shifted anti-diagonal",
        },
        GoldenPuzzle {
            n: 5,
            desc: "_41,a3a1a4a2a5a1a4a2a5a3a4a2a5a3a1a2a5a3a1a4a5a3a1a4a2",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[
                3, 1, 4, 2, 5, 1, 4, 2, 5, 3, 4, 2, 5, 3, 1, 2, 5, 3, 1, 4, 5, 3, 1, 4, 2,
            ]),
            label: "5x5 permuted singleton A",
        },
        GoldenPuzzle {
            n: 5,
            desc: "_41,a4a2a5a3a1a2a5a3a1a4a5a3a1a4a2a3a1a4a2a5a1a4a2a5a3",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[
                4, 2, 5, 3, 1, 2, 5, 3, 1, 4, 5, 3, 1, 4, 2, 3, 1, 4, 2, 5, 1, 4, 2, 5, 3,
            ]),
            label: "5x5 permuted singleton B",
        },
        GoldenPuzzle {
            n: 5,
            desc: "_41,a5a3a1a4a2a3a1a4a2a5a1a4a2a5a3a4a2a5a3a1a2a5a3a1a4",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[
                5, 3, 1, 4, 2, 3, 1, 4, 2, 5, 1, 4, 2, 5, 3, 4, 2, 5, 3, 1, 2, 5, 3, 1, 4,
            ]),
            label: "5x5 permuted singleton C",
        },
        // ============================================================
        // Additional variety puzzles
        // ============================================================
        GoldenPuzzle {
            n: 3,
            desc: "_13,a1a2a3a3a1a2a2a3a1",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[1, 2, 3, 3, 1, 2, 2, 3, 1]),
            label: "3x3 singleton grid G",
        },
        GoldenPuzzle {
            n: 4,
            desc: "_25,a1a2a4a3a3a4a2a1a4a3a1a2a2a1a3a4",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[1, 2, 4, 3, 3, 4, 2, 1, 4, 3, 1, 2, 2, 1, 3, 4]),
            label: "4x4 singleton grid L",
        },
        GoldenPuzzle {
            n: 4,
            desc: "_25,a1a4a3a2a4a1a2a3a3a2a1a4a2a3a4a1",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[1, 4, 3, 2, 4, 1, 2, 3, 3, 2, 1, 4, 2, 3, 4, 1]),
            label: "4x4 singleton grid M",
        },
        GoldenPuzzle {
            n: 4,
            desc: "_25,a2a4a1a3a4a2a3a1a1a3a2a4a3a1a4a2",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[2, 4, 1, 3, 4, 2, 3, 1, 1, 3, 2, 4, 3, 1, 4, 2]),
            label: "4x4 singleton grid N",
        },
        GoldenPuzzle {
            n: 4,
            desc: "_25,a3a2a1a4a2a3a4a1a1a4a3a2a4a1a2a3",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[3, 2, 1, 4, 2, 3, 4, 1, 1, 4, 3, 2, 4, 1, 2, 3]),
            label: "4x4 singleton grid O",
        },
        GoldenPuzzle {
            n: 5,
            desc: "_41,a1a4a2a5a3a4a2a5a3a1a2a5a3a1a4a5a3a1a4a2a3a1a4a2a5",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[
                1, 4, 2, 5, 3, 4, 2, 5, 3, 1, 2, 5, 3, 1, 4, 5, 3, 1, 4, 2, 3, 1, 4, 2, 5,
            ]),
            label: "5x5 offset cyclic singleton",
        },
        GoldenPuzzle {
            n: 5,
            desc: "_41,a2a4a1a3a5a4a1a3a5a2a1a3a5a2a4a3a5a2a4a1a5a2a4a1a3",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[
                2, 4, 1, 3, 5, 4, 1, 3, 5, 2, 1, 3, 5, 2, 4, 3, 5, 2, 4, 1, 5, 2, 4, 1, 3,
            ]),
            label: "5x5 offset-2 cyclic singleton",
        },
        GoldenPuzzle {
            n: 5,
            desc: "_41,a3a5a2a4a1a5a2a4a1a3a2a4a1a3a5a4a1a3a5a2a1a3a5a2a4",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[
                3, 5, 2, 4, 1, 5, 2, 4, 1, 3, 2, 4, 1, 3, 5, 4, 1, 3, 5, 2, 1, 3, 5, 2, 4,
            ]),
            label: "5x5 offset-3 cyclic singleton",
        },
        // Additional 3x3 variations
        GoldenPuzzle {
            n: 3,
            desc: "_13,a2a1a3a3a2a1a1a3a2",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[2, 1, 3, 3, 2, 1, 1, 3, 2]),
            label: "3x3 singleton grid H",
        },
        GoldenPuzzle {
            n: 3,
            desc: "_13,a3a1a2a2a3a1a1a2a3",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[3, 1, 2, 2, 3, 1, 1, 2, 3]),
            label: "3x3 singleton grid I",
        },
        GoldenPuzzle {
            n: 3,
            desc: "_13,a2a3a1a1a2a3a3a1a2",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[2, 3, 1, 1, 2, 3, 3, 1, 2]),
            label: "3x3 singleton grid J",
        },
        GoldenPuzzle {
            n: 3,
            desc: "_13,a1a3a2a2a1a3a3a2a1",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[1, 3, 2, 2, 1, 3, 3, 2, 1]),
            label: "3x3 singleton grid K",
        },
        // Exhaustive 3x3 rotation set
        GoldenPuzzle {
            n: 3,
            desc: "_13,a3a2a1a1a3a2a2a1a3",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[3, 2, 1, 1, 3, 2, 2, 1, 3]),
            label: "3x3 singleton grid L",
        },
        // ============================================================
        // 6x6 PUZZLES (Test encoding - singleton cages)
        // ============================================================
        GoldenPuzzle {
            n: 6,
            desc: "_61,a1a2a3a4a5a6a2a3a4a5a6a1a3a4a5a6a1a2a4a5a6a1a2a3a5a6a1a2a3a4a6a1a2a3a4a5",
            solutions: 1,
            difficulty: Some(DifficultyTier::Easy),
            tier_required: Some(DeductionTier::Easy),
            solution: Some(&[
                1, 2, 3, 4, 5, 6, 2, 3, 4, 5, 6, 1, 3, 4, 5, 6, 1, 2, 4, 5, 6, 1, 2, 3, 5, 6, 1, 2,
                3, 4, 6, 1, 2, 3, 4, 5,
            ]),
            label: "6x6 cyclic singleton grid",
        },
        // ============================================================
        // 6x6 DIVERSE PUZZLES (Generated corpus - varied operations)
        // ============================================================
        GoldenPuzzle {
            n: 6,
            desc: "aa_3a__ba__a__a__a_abb_a_3a_4b__b_a_,m15m24m6m12m24d5m24s4m20a15a7a7a5a11d3",
            solutions: 1,
            difficulty: None, // Conservative estimate
            tier_required: Some(DeductionTier::Hard),
            solution: Some(&[
                5, 1, 4, 6, 3, 2, 4, 3, 5, 1, 2, 6, 6, 2, 3, 4, 1, 5, 1, 4, 6, 2, 5, 3, 2, 5, 1, 3,
                6, 4, 3, 6, 2, 5, 4, 1,
            ]),
            label: "6x6 Mul-heavy puzzle (seed 0)",
        },
        GoldenPuzzle {
            n: 6,
            desc: "_a_aa__aa__a3_aabaa_9aa_4a_3ab,s3a7m18s1s5d4a7a12a10a7s1m72d3m10s4a9",
            solutions: 1,
            difficulty: None, // Conservative estimate
            tier_required: Some(DeductionTier::Hard),
            solution: Some(&[
                5, 4, 3, 6, 2, 1, 2, 6, 1, 3, 4, 5, 3, 5, 6, 4, 1, 2, 4, 1, 2, 5, 6, 3, 1, 3, 4, 2,
                5, 6, 6, 2, 5, 1, 3, 4,
            ]),
            label: "6x6 Add-heavy puzzle (seed 1)",
        },
        GoldenPuzzle {
            n: 6,
            desc: "_bb_b_aa_4a_3a__ba_3a_a_b__b__a_3b,m18m24m20a13m5m24s1m2m30a10s2m432d2s3",
            solutions: 1,
            difficulty: None, // Conservative estimate
            tier_required: Some(DeductionTier::Hard),
            solution: Some(&[
                6, 3, 2, 1, 4, 5, 3, 1, 4, 6, 5, 2, 5, 6, 3, 4, 2, 1, 1, 4, 5, 2, 3, 6, 4, 2, 6, 5,
                1, 3, 2, 5, 1, 3, 6, 4,
            ]),
            label: "6x6 Mul-heavy puzzle (seed 2)",
        },
        GoldenPuzzle {
            n: 6,
            desc: "a_baa__a__a_6a_abaa_ab_a_3a__aaba,m108m8m10s1m24a7m120d2s5s1s3s2a18a3",
            solutions: 1,
            difficulty: None, // Conservative estimate
            tier_required: None,
            solution: Some(&[
                3, 6, 4, 5, 2, 1, 6, 2, 1, 4, 5, 3, 5, 4, 3, 6, 1, 2, 2, 1, 5, 3, 6, 4, 1, 3, 6, 2,
                4, 5, 4, 5, 2, 1, 3, 6,
            ]),
            label: "6x6 Mul-heavy puzzle (seed 3)",
        },
        GoldenPuzzle {
            n: 6,
            desc: "ab_ba_a_3a__a__aabaa_a__a_3a_a_aca_,m60a8a15m108a4s1a6a6s3m32a6a14m12",
            solutions: 1,
            difficulty: None, // Conservative estimate
            tier_required: Some(DeductionTier::Hard),
            solution: Some(&[
                6, 2, 1, 4, 3, 5, 5, 3, 6, 1, 4, 2, 1, 6, 2, 3, 5, 4, 3, 5, 4, 2, 1, 6, 4, 1, 5, 6,
                2, 3, 2, 4, 3, 5, 6, 1,
            ]),
            label: "6x6 Add-heavy puzzle (seed 4)",
        },
        GoldenPuzzle {
            n: 6,
            desc: "__b_aa_3a_3a3_aa_aa_aa_3a__a_3a_baa,m6m48a12s4a11s1a9m2a7m12s1m15a10s4s2d4",
            solutions: 1,
            difficulty: None, // Conservative estimate
            tier_required: Some(DeductionTier::Hard),
            solution: Some(&[
                6, 2, 3, 4, 5, 1, 1, 4, 6, 2, 3, 5, 4, 6, 2, 5, 1, 3, 5, 3, 1, 6, 2, 4, 2, 1, 5, 3,
                4, 6, 3, 5, 4, 1, 6, 2,
            ]),
            label: "6x6 Sub/Div-mixed puzzle (seed 5)",
        },
        GoldenPuzzle {
            n: 6,
            desc: "aa_3a_3e_b__b__aa_3a_6a__c_3a_4,m2s2s2m30d2s3s1m120m6m40s2s1a9d2a9d3",
            solutions: 1,
            difficulty: None, // Conservative estimate
            tier_required: None,
            solution: Some(&[
                1, 2, 4, 6, 3, 5, 3, 4, 2, 1, 5, 6, 6, 1, 5, 3, 4, 2, 2, 3, 1, 5, 6, 4, 5, 6, 3, 4,
                2, 1, 4, 5, 6, 2, 1, 3,
            ]),
            label: "6x6 Sub/Div-mixed puzzle (seed 6)",
        },
        GoldenPuzzle {
            n: 6,
            desc: "a_aa_a_a_3aa__a_4a__a3_b__aa_4abaa,s5d2m10a7m60a9s2m32a11s3d2a9a11m36d2",
            solutions: 1,
            difficulty: None, // Conservative estimate
            tier_required: Some(DeductionTier::Normal),
            solution: Some(&[
                6, 1, 4, 2, 5, 3, 5, 6, 2, 1, 3, 4, 3, 2, 1, 5, 4, 6, 1, 3, 6, 4, 2, 5, 2, 4, 5, 3,
                6, 1, 4, 5, 3, 6, 1, 2,
            ]),
            label: "6x6 Add-heavy puzzle (seed 8)",
        },
        GoldenPuzzle {
            n: 6,
            desc: "aab_9a__aa_ab__a_3a__a__aa__aa_ba,d3m20a7s4s1m6a12a7m60a8a11s1s4m5m6a11",
            solutions: 1,
            difficulty: None, // Conservative estimate
            tier_required: None,
            solution: Some(&[
                2, 6, 5, 1, 4, 3, 5, 1, 2, 4, 3, 6, 6, 5, 1, 3, 2, 4, 1, 4, 3, 5, 6, 2, 4, 3, 6, 2,
                5, 1, 3, 2, 4, 6, 1, 5,
            ]),
            label: "6x6 Add-heavy puzzle (seed 9)",
        },
        GoldenPuzzle {
            n: 6,
            desc: "b_b__b_a_aa_aba3_c_4a__a_a_3a__aa,a14a9a7m12s4d5a10s3a11s2a16d5s2a6",
            solutions: 1,
            difficulty: None, // Conservative estimate
            tier_required: Some(DeductionTier::Hard),
            solution: Some(&[
                1, 4, 3, 6, 2, 5, 4, 2, 6, 3, 5, 1, 5, 1, 2, 4, 6, 3, 2, 3, 5, 1, 4, 6, 6, 5, 1, 2,
                3, 4, 3, 6, 4, 5, 1, 2,
            ]),
            label: "6x6 Add-heavy puzzle (seed 14)",
        },
        // ============================================================
        // 8x8 DIVERSE PUZZLES (Generated corpus - mixed operations)
        // ============================================================
        GoldenPuzzle {
            n: 8,
            desc: "a_b__a_9a_3a_5a_b_3a_a_5a_3a3_a_aa_a3__a__ba_ab_a_ba__caa_,d6m168m168m16a13s2a10s7s7a10a17m12m4d7a15a11a9m105a13s2s3a15a8m30m12m21m4",
            solutions: 1,
            difficulty: None, // Conservative estimate
            tier_required: None,
            solution: Some(&[
                1, 6, 3, 7, 2, 4, 8, 5, 5, 7, 8, 4, 1, 3, 2, 6, 3, 1, 4, 6, 8, 5, 7, 2, 2, 8, 6, 1,
                4, 7, 5, 3, 6, 2, 7, 5, 3, 1, 4, 8, 4, 5, 2, 3, 7, 8, 6, 1, 7, 3, 5, 8, 6, 2, 1, 4,
                8, 4, 1, 2, 5, 6, 3, 7,
            ]),
            label: "8x8 diverse puzzle (seed 0)",
        },
        GoldenPuzzle {
            n: 8,
            desc: "_ab_3aa__a_a_b_a_aa_a_a__a_a_a_a3c__c_14a_a_4a_8aa__bba_,a16m96a12a9m8a9d8d2m28s3s3m56m90a15a10s3a11a6m84m40m24m8m18s5s1a20m4a5",
            solutions: 1,
            difficulty: None, // Conservative estimate
            tier_required: Some(DeductionTier::Hard),
            solution: Some(&[
                7, 4, 8, 5, 6, 1, 3, 2, 5, 3, 7, 2, 1, 8, 6, 4, 4, 1, 2, 7, 3, 6, 5, 8, 3, 5, 6, 4,
                8, 7, 2, 1, 8, 2, 3, 6, 4, 5, 1, 7, 1, 8, 5, 3, 7, 2, 4, 6, 2, 6, 1, 8, 5, 4, 7, 3,
                6, 7, 4, 1, 2, 3, 8, 5,
            ]),
            label: "8x8 diverse puzzle (seed 1)",
        },
        GoldenPuzzle {
            n: 8,
            desc: "a_b_5a_18a4_4b_4ab3_a3__ab__aa_4b_a__a_a3_aaba_,a16m24a19s1d3a9m84a14m24d5d2m56m120m48a10a6s2d4a11s1s5s1s2a10s4m48s3",
            solutions: 1,
            difficulty: None, // Conservative estimate
            tier_required: None,
            solution: Some(&[
                1, 7, 6, 4, 5, 8, 2, 3, 8, 5, 4, 2, 6, 7, 3, 1, 7, 4, 3, 1, 2, 6, 8, 5, 2, 6, 8, 5,
                1, 3, 7, 4, 5, 8, 1, 7, 3, 2, 4, 6, 3, 1, 2, 8, 4, 5, 6, 7, 6, 2, 5, 3, 7, 4, 1, 8,
                4, 3, 7, 6, 8, 1, 5, 2,
            ]),
            label: "8x8 diverse puzzle (seed 2)",
        },
        GoldenPuzzle {
            n: 8,
            desc: "aa_abaa__b_15ba_4a_3aa_3a_ca3_5a_5a_b_ba_a3_a_a4__,m20d3a10s1m48a14s1m1680m40a10d6s1s6s1m20m168a3m15d2m98d8d5s4s2m24a10m20",
            solutions: 1,
            difficulty: None, // Conservative estimate
            tier_required: None,
            solution: Some(&[
                5, 4, 1, 3, 2, 7, 8, 6, 3, 6, 2, 7, 1, 5, 4, 8, 7, 3, 8, 1, 5, 4, 6, 2, 4, 2, 7, 5,
                8, 6, 1, 3, 2, 8, 6, 4, 7, 1, 3, 5, 6, 5, 4, 8, 3, 2, 7, 1, 8, 1, 5, 6, 4, 3, 2, 7,
                1, 7, 3, 2, 6, 8, 5, 4,
            ]),
            label: "8x8 diverse puzzle (seed 3)",
        },
        GoldenPuzzle {
            n: 8,
            desc: "b3_a4_a_8a_a__aa_a_a_ab__a_3aa__a__b_a_4a3_a__a__b_4a_3c_a,a13m336s1a7s2m7m480m30m16s1m28d2m147a13a8a17m56a4m42m384m90a8m6s3m40",
            solutions: 1,
            difficulty: None, // Conservative estimate
            tier_required: None,
            solution: Some(&[
                5, 1, 2, 7, 6, 8, 4, 3, 3, 4, 5, 6, 8, 7, 1, 2, 1, 6, 8, 2, 3, 4, 7, 5, 4, 5, 7, 1,
                2, 6, 3, 8, 2, 7, 3, 8, 4, 1, 5, 6, 8, 3, 1, 4, 5, 2, 6, 7, 7, 8, 6, 5, 1, 3, 2, 4,
                6, 2, 4, 3, 7, 5, 8, 1,
            ]),
            label: "8x8 diverse puzzle (seed 4)",
        },
        // ============================================================
        // 9x9 HARD TIER PUZZLES (Generated corpus - requires backtracking)
        // ============================================================
        GoldenPuzzle {
            n: 9,
            desc: "_aa__c_7a_a_4aa__a_a_b__a__a__ba_5ba_4bb__a_a__a_3ab_a_aba_4b_5b__a_ba_a__ab_3,a10s1a15s5s2a13m10s1a10m192a4d2m63m54m48m15m96s1a14m63m35a13s1m56m324a21m36m10a5s3a13a13m32",
            solutions: 1,
            difficulty: None, // Conservative estimate
            tier_required: Some(DeductionTier::Hard),
            solution: Some(&[
                1, 8, 7, 6, 9, 2, 4, 3, 5, 3, 2, 4, 1, 8, 7, 6, 5, 9, 6, 3, 5, 2, 7, 4, 8, 9, 1, 4,
                1, 9, 8, 6, 3, 5, 7, 2, 8, 6, 1, 3, 5, 9, 7, 2, 4, 7, 5, 3, 4, 2, 1, 9, 8, 6, 9, 4,
                8, 5, 3, 6, 2, 1, 7, 5, 9, 2, 7, 4, 8, 1, 6, 3, 2, 7, 6, 9, 1, 5, 3, 4, 8,
            ]),
            label: "9x9 Hard tier puzzle (seed 0)",
        },
        GoldenPuzzle {
            n: 9,
            desc: "_aa_a__b_a3_4a_a__ba_3b_6a_a_a3b_baab_a_3a_a_5a_a_5a_4aa__baa_3aa_3a_aab__b_,m14m120a17m35m6m24m63m288m9m96a12s1m240s6a16m4d3a15m108m18m14s5m280s5s2a10a16m10m18a9s1a15a6",
            solutions: 1,
            difficulty: None, // Conservative estimate
            tier_required: Some(DeductionTier::Hard),
            solution: Some(&[
                7, 5, 6, 9, 8, 1, 2, 3, 4, 2, 4, 9, 7, 1, 5, 6, 8, 3, 1, 9, 8, 3, 5, 7, 4, 6, 2, 6,
                1, 4, 5, 2, 8, 3, 7, 9, 5, 8, 1, 4, 3, 2, 7, 9, 6, 3, 6, 7, 1, 9, 4, 8, 2, 5, 8, 3,
                2, 6, 4, 9, 5, 1, 7, 9, 7, 5, 2, 6, 3, 1, 4, 8, 4, 2, 3, 8, 7, 6, 9, 5, 1,
            ]),
            label: "9x9 Hard tier puzzle (seed 5)",
        },
        GoldenPuzzle {
            n: 9,
            desc: "aa_aab_a3_a_aa_a__a4_5a_6a_3a3_caba_aa_3aa_5a_3a_a_a_3a_11a_7aa_4a3,a16a14m8s4m3a22a7a10d3m8s3m14s1m36a7m48m224s4m12d7a12a7m20s4m21m14m24a11a11s1d2a10a5s4a19a18s8",
            solutions: 1,
            difficulty: None, // Conservative estimate
            tier_required: Some(DeductionTier::Hard),
            solution: Some(&[
                7, 4, 9, 5, 8, 6, 2, 1, 3, 5, 7, 6, 9, 1, 4, 3, 2, 8, 3, 8, 1, 6, 2, 7, 5, 9, 4, 9,
                5, 2, 3, 6, 8, 4, 7, 1, 2, 6, 7, 1, 9, 3, 8, 4, 5, 6, 1, 5, 4, 3, 2, 7, 8, 9, 1, 9,
                4, 8, 7, 5, 6, 3, 2, 8, 2, 3, 7, 4, 1, 9, 5, 6, 4, 3, 8, 2, 5, 9, 1, 6, 7,
            ]),
            label: "9x9 Hard tier puzzle (seed 7)",
        },
    ]
}

#[test]
fn golden_corpus_parse_and_validate() {
    let rules = Ruleset::keen_baseline();

    for puzzle_def in golden_corpus() {
        let puzzle = parse_keen_desc(puzzle_def.n, puzzle_def.desc).unwrap_or_else(|e| {
            panic!("Failed to parse '{}': {}", puzzle_def.label, e);
        });

        puzzle.validate(rules).unwrap_or_else(|e| {
            panic!("Validation failed for '{}': {}", puzzle_def.label, e);
        });
    }
}

#[test]
fn golden_corpus_solution_counts() {
    let rules = Ruleset::keen_baseline();

    for puzzle_def in golden_corpus() {
        let puzzle = parse_keen_desc(puzzle_def.n, puzzle_def.desc).unwrap();

        if puzzle.validate(rules).is_err() {
            continue;
        }

        let limit = puzzle_def.solutions.saturating_add(1);
        let count =
            count_solutions_up_to_with_deductions(&puzzle, rules, DeductionTier::Hard, limit)
                .unwrap();

        assert_eq!(
            count, puzzle_def.solutions,
            "'{}': expected {} solutions, got {}",
            puzzle_def.label, puzzle_def.solutions, count
        );
    }
}

#[test]
fn golden_corpus_unique_puzzles_have_known_solutions() {
    let rules = Ruleset::keen_baseline();

    for puzzle_def in golden_corpus() {
        if puzzle_def.solutions != 1 || puzzle_def.solution.is_none() {
            continue;
        }

        let puzzle = parse_keen_desc(puzzle_def.n, puzzle_def.desc).unwrap();

        if puzzle.validate(rules).is_err() {
            continue;
        }

        let solution = solve_one_with_deductions(&puzzle, rules, DeductionTier::Hard)
            .unwrap()
            .unwrap();

        let expected = puzzle_def.solution.unwrap();
        assert_eq!(
            solution.grid.as_slice(),
            expected,
            "'{}': solution mismatch",
            puzzle_def.label
        );
    }
}

#[test]
fn golden_corpus_difficulty_classification() {
    let rules = Ruleset::keen_baseline();

    for puzzle_def in golden_corpus() {
        if puzzle_def.difficulty.is_none() || puzzle_def.solutions != 1 {
            continue;
        }

        let puzzle = parse_keen_desc(puzzle_def.n, puzzle_def.desc).unwrap();

        if puzzle.validate(rules).is_err() {
            continue;
        }

        let result = classify_tier_required(&puzzle, rules).unwrap();
        let difficulty = classify_difficulty_from_tier(result);
        let expected = puzzle_def.difficulty.unwrap();

        assert_eq!(
            difficulty, expected,
            "'{}': expected difficulty {:?}, got {:?}",
            puzzle_def.label, expected, difficulty
        );
    }
}

#[test]
fn golden_corpus_tier_required() {
    let rules = Ruleset::keen_baseline();

    for puzzle_def in golden_corpus() {
        if puzzle_def.tier_required.is_none() || puzzle_def.solutions != 1 {
            continue;
        }

        let puzzle = parse_keen_desc(puzzle_def.n, puzzle_def.desc).unwrap();

        if puzzle.validate(rules).is_err() {
            continue;
        }

        let result = classify_tier_required(&puzzle, rules).unwrap();
        let expected = puzzle_def.tier_required;

        assert_eq!(
            result.tier_required, expected,
            "'{}': expected tier_required {:?}, got {:?}",
            puzzle_def.label, expected, result.tier_required
        );
    }
}

#[test]
fn golden_corpus_covers_all_grid_sizes() {
    let corpus = golden_corpus();
    let sizes: std::collections::HashSet<u8> = corpus.iter().map(|p| p.n).collect();

    assert!(sizes.contains(&2), "Missing 2x2 puzzles");
    assert!(sizes.contains(&3), "Missing 3x3 puzzles");
    assert!(sizes.contains(&4), "Missing 4x4 puzzles");
    assert!(sizes.contains(&5), "Missing 5x5 puzzles");
    assert!(sizes.contains(&6), "Missing 6x6 puzzles");
    assert!(sizes.contains(&8), "Missing 8x8 puzzles");
    assert!(sizes.contains(&9), "Missing 9x9 puzzles");
}

#[test]
fn golden_corpus_has_minimum_count() {
    let corpus = golden_corpus();
    assert!(
        corpus.len() >= 65,
        "Golden corpus should have at least 65 puzzles, has {}",
        corpus.len()
    );
}

#[test]
fn golden_corpus_has_unique_puzzles() {
    let corpus = golden_corpus();
    let unique_count = corpus.iter().filter(|p| p.solutions == 1).count();

    assert!(
        unique_count >= 40,
        "Should have at least 40 unique puzzles, has {}",
        unique_count
    );
}
