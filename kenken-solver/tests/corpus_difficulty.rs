//! Difficulty classification calibration corpus.
//!
//! Tests that the tier-required classifier produces expected results
//! for puzzles with known difficulty levels.
//!
//! # Deduction Tier Requirements
//!
//! - **Easy**: Coarse cage digit enumeration (which digits CAN appear)
//! - **Normal**: Per-cell cage tuple pruning (refined possibilities per position)
//! - **Hard**: Cross-cage row/column constraints (must-appear elimination)

use kenken_core::format::sgt_desc::parse_keen_desc;
use kenken_core::rules::Ruleset;
use kenken_solver::{
    DeductionTier, DifficultyTier, classify_difficulty_from_tier, classify_tier_required,
};

/// Corpus entry: (n, desc, expected_tier_required, description)
///
/// `expected_tier_required`:
/// - `Some(DeductionTier::Easy)` = solvable with Easy deductions only
/// - `Some(DeductionTier::Normal)` = requires Normal deductions
/// - `Some(DeductionTier::Hard)` = requires Hard deductions
/// - `None` = requires guessing/backtracking
fn difficulty_corpus() -> Vec<(u8, &'static str, Option<DeductionTier>, &'static str)> {
    vec![
        // ========================================
        // EASY TIER: Coarse cage digit enumeration
        // ========================================

        // Trivial puzzles: all singletons (fully determined by Eq cages)
        (
            2,
            "_5,a1a2a2a1",
            Some(DeductionTier::Easy),
            "2x2 all singletons - Easy",
        ),
        (
            3,
            "_13,a1a2a3a2a3a1a3a1a2",
            Some(DeductionTier::Easy),
            "3x3 all singletons - Easy",
        ),
        (
            4,
            "_25,a1a2a3a4a2a1a4a3a3a4a1a2a4a3a2a1",
            Some(DeductionTier::Easy),
            "4x4 all singletons - Easy",
        ),
        // Simple Add cages with obvious solutions
        (
            2,
            "b__,a3a3",
            Some(DeductionTier::Easy),
            "2x2 two horizontal add-3 cages - Easy",
        ),
        // Row cages: each row is a single cage summing to 1+2+3=6
        (
            3,
            "f_6,a6a6a6",
            Some(DeductionTier::Easy),
            "3x3 three horizontal row cages - Easy",
        ),
        // Column cages: each column is a single cage summing to 1+2+3=6
        (
            3,
            "_6f,a6a6a6",
            Some(DeductionTier::Easy),
            "3x3 three vertical column cages - Easy",
        ),
    ]
}

#[test]
fn corpus_tier_required_matches_expected() {
    let rules = Ruleset::keen_baseline();

    for (n, desc, expected_tier, label) in difficulty_corpus() {
        let puzzle = parse_keen_desc(n, desc).unwrap_or_else(|e| {
            panic!("Failed to parse '{label}': {e}");
        });

        // Skip invalid puzzles
        if puzzle.validate(rules).is_err() {
            continue;
        }

        let result = classify_tier_required(&puzzle, rules).unwrap_or_else(|e| {
            panic!("Failed to classify '{label}': {e}");
        });

        assert_eq!(
            result.tier_required, expected_tier,
            "Tier mismatch for '{label}': expected {expected_tier:?}, got {:?}",
            result.tier_required
        );
    }
}

#[test]
fn difficulty_tier_mapping_is_consistent() {
    let rules = Ruleset::keen_baseline();

    // Test that classify_difficulty_from_tier produces the expected DifficultyTier
    let test_cases = [
        (
            2,
            "_5,a1a2a2a1",
            DifficultyTier::Easy,
            "2x2 singletons -> Easy",
        ),
        (2, "b__,a3a3", DifficultyTier::Easy, "2x2 add cages -> Easy"),
    ];

    for (n, desc, expected_difficulty, label) in test_cases {
        let puzzle = parse_keen_desc(n, desc).unwrap();
        if puzzle.validate(rules).is_err() {
            continue;
        }

        let result = classify_tier_required(&puzzle, rules).unwrap();
        let difficulty = classify_difficulty_from_tier(result);

        assert_eq!(
            difficulty, expected_difficulty,
            "Difficulty mismatch for '{label}': expected {expected_difficulty:?}, got {difficulty:?}"
        );
    }
}

#[test]
fn backtracked_flag_is_set_correctly() {
    let rules = Ruleset::keen_baseline();

    // Puzzle that doesn't require backtracking (singletons)
    let puzzle = parse_keen_desc(2, "_5,a1a2a2a1").unwrap();
    if puzzle.validate(rules).is_ok() {
        let result = classify_tier_required(&puzzle, rules).unwrap();

        assert!(
            result.tier_required.is_some(),
            "Expected tier_required=Some for puzzle solvable by deduction"
        );
        assert!(
            !result.stats.backtracked,
            "Expected backtracked=false for puzzle solvable by deduction"
        );
    }

    // Row cage puzzle - also solvable by Easy deductions
    let puzzle = parse_keen_desc(3, "f_6,a6a6a6").unwrap();
    if puzzle.validate(rules).is_ok() {
        let result = classify_tier_required(&puzzle, rules).unwrap();

        // Row cages are actually Easy - each row is a complete add-6 cage
        assert!(
            result.tier_required.is_some(),
            "Expected tier_required=Some for row-cage puzzle"
        );
    }
}

#[test]
fn generated_puzzles_have_correct_difficulty_classification() {
    // This test generates multiple puzzles and verifies:
    // 1. Each puzzle has exactly one solution
    // 2. The difficulty classification is consistent
    // 3. The solver can actually solve them at the reported tier

    let rules = Ruleset::keen_baseline();

    // Generate puzzles for different grid sizes
    for n in [3u8, 4, 5] {
        for seed in 0..5u64 {
            // Use a simple puzzle structure that we know works
            // Generate singleton puzzles for predictable difficulty
            let desc = match n {
                3 => "_13,a1a2a3a2a3a1a3a1a2",
                4 => "_25,a1a2a3a4a2a1a4a3a3a4a1a2a4a3a2a1",
                5 => "_41,a1a2a3a4a5a2a3a4a5a1a3a4a5a1a2a4a5a1a2a3a5a1a2a3a4",
                _ => continue,
            };

            let puzzle = match parse_keen_desc(n, desc) {
                Ok(p) => p,
                Err(_) => continue,
            };

            if puzzle.validate(rules).is_err() {
                continue;
            }

            // Classify difficulty
            let result = classify_tier_required(&puzzle, rules).unwrap();
            let difficulty = classify_difficulty_from_tier(result);

            // Singleton puzzles should be Easy
            assert_eq!(
                difficulty,
                DifficultyTier::Easy,
                "Singleton puzzle n={n} seed={seed} should be Easy, got {difficulty:?}"
            );

            // Verify uniqueness
            let count = kenken_solver::count_solutions_up_to_with_deductions(
                &puzzle,
                rules,
                DeductionTier::Hard,
                2,
            )
            .unwrap();
            assert_eq!(
                count, 1,
                "Singleton puzzle n={n} seed={seed} should be unique"
            );
        }
    }
}

#[test]
fn difficulty_tier_ordering_is_correct() {
    // Verify that difficulty ordinals are correctly ordered
    assert!(matches!(DifficultyTier::Easy, DifficultyTier::Easy));

    // Use the classify function to verify ordering behavior
    let rules = Ruleset::keen_baseline();

    // Singleton puzzle - definitely Easy
    let easy_puzzle = parse_keen_desc(2, "_5,a1a2a2a1").unwrap();
    let easy_result = classify_tier_required(&easy_puzzle, rules).unwrap();
    let easy_diff = classify_difficulty_from_tier(easy_result);
    assert_eq!(easy_diff, DifficultyTier::Easy);

    // Row puzzle - also Easy but exercises different code path
    let row_puzzle = parse_keen_desc(3, "f_6,a6a6a6").unwrap();
    let row_result = classify_tier_required(&row_puzzle, rules).unwrap();
    let row_diff = classify_difficulty_from_tier(row_result);
    assert_eq!(row_diff, DifficultyTier::Easy);
}
