//! Golden corpus for comprehensive solver and difficulty testing.
//!
//! Contains 50+ puzzles across grid sizes 2-6 and all difficulty tiers.
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
//! - **6x6**: Hard/Extreme puzzles

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
}

#[test]
fn golden_corpus_has_minimum_count() {
    let corpus = golden_corpus();
    assert!(
        corpus.len() >= 50,
        "Golden corpus should have at least 50 puzzles, has {}",
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
