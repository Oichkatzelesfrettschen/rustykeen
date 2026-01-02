//! Golden corpus tests for sgt-desc format puzzles.
//!
//! Tests various puzzle sizes with known solution counts to verify
//! solver correctness and determinism.

use kenken_core::format::sgt_desc::{encode_keen_desc, parse_keen_desc};
use kenken_core::rules::Ruleset;
use kenken_solver::{
    DeductionTier, count_solutions_up_to_with_deductions, solve_one_with_deductions,
};

/// Corpus entry: (grid_size, sgt_desc, expected_solution_count, description)
fn corpus() -> Vec<(u8, &'static str, u32, &'static str)> {
    vec![
        // 2x2 puzzles - minimal grid for basic testing
        (
            2,
            "b__,a3a3",
            2,
            "2x2 two horizontal 2-cages (row 0 sum=3, row 1 sum=3)",
        ),
        (
            2,
            "__b,a3a3",
            2,
            "2x2 two vertical 2-cages (col 0 sum=3, col 1 sum=3)",
        ),
        (
            2,
            "_5,a1a2a2a1",
            1,
            "2x2 all singletons with forced values (grid=[1,2;2,1])",
        ),
        // 3x3 puzzles - exercises cage enumeration
        (
            3,
            "f_6,a6a6a6",
            12,
            "3x3 three horizontal row cages (each sum=6, all Latin squares)",
        ),
        (
            3,
            "_6f,a6a6a6",
            12,
            "3x3 three vertical column cages (each sum=6, all Latin squares)",
        ),
        (
            3,
            "_13,a1a2a3a2a3a1a3a1a2",
            1,
            "3x3 all singletons with forced values (grid=[1,2,3;2,3,1;3,1,2])",
        ),
        // 4x4 puzzles - larger grid, stress tests
        (
            4,
            "_25,a1a2a3a4a2a1a4a3a3a4a1a2a4a3a2a1",
            1,
            "4x4 all singletons with forced values (grid=[1,2,3,4;2,1,4,3;3,4,1,2;4,3,2,1])",
        ),
    ]
}

#[test]
fn sgt_desc_corpus_counts_match_expectations() {
    let rules = Ruleset::keen_baseline();

    for (n, desc, expected, name) in corpus() {
        let puzzle = match parse_keen_desc(n, desc) {
            Ok(p) => p,
            Err(e) => {
                panic!("Failed to parse puzzle '{}': {}", name, e);
            }
        };

        if let Err(e) = puzzle.validate(rules) {
            panic!("Puzzle '{}' failed validation: {}", name, e);
        }

        // Use expected+1 as limit to verify exact count
        let limit = expected.saturating_add(1);
        let got =
            count_solutions_up_to_with_deductions(&puzzle, rules, DeductionTier::Normal, limit)
                .unwrap();

        assert_eq!(
            got, expected,
            "Puzzle '{}' (n={}, desc={}): expected {} solutions, got {}",
            name, n, desc, expected, got
        );
    }
}

#[test]
fn sgt_desc_roundtrip_encoding() {
    let rules = Ruleset::keen_baseline();

    for (n, desc, _, name) in corpus() {
        let puzzle = match parse_keen_desc(n, desc) {
            Ok(p) => p,
            Err(e) => {
                panic!("Failed to parse puzzle '{}': {}", name, e);
            }
        };

        let encoded = encode_keen_desc(&puzzle, rules).expect("encode failed");
        let reparsed = parse_keen_desc(n, &encoded).expect("reparse failed");

        assert_eq!(
            puzzle.cages.len(),
            reparsed.cages.len(),
            "Puzzle '{}': cage count mismatch after roundtrip",
            name
        );
    }
}

#[test]
fn unique_puzzles_have_single_solution() {
    let rules = Ruleset::keen_baseline();

    for (n, desc, expected, name) in corpus() {
        if expected != 1 {
            continue;
        }

        let puzzle = parse_keen_desc(n, desc).expect("parse failed");

        // Verify solve_one finds a solution
        let solution =
            solve_one_with_deductions(&puzzle, rules, DeductionTier::Normal).expect("solve failed");

        assert!(
            solution.is_some(),
            "Puzzle '{}' expected unique solution but solve_one returned None",
            name
        );

        // Verify count agrees
        let count = count_solutions_up_to_with_deductions(&puzzle, rules, DeductionTier::Normal, 2)
            .unwrap();

        assert_eq!(
            count, 1,
            "Puzzle '{}' claimed unique but count_solutions found {}",
            name, count
        );
    }
}

#[test]
fn deduction_tiers_produce_consistent_counts() {
    let rules = Ruleset::keen_baseline();
    let (n, desc, _, _) = (2, "b__,a3a3", 2, "2x2 baseline");

    let puzzle = parse_keen_desc(n, desc).expect("parse failed");

    let counts: Vec<u32> = [
        DeductionTier::None,
        DeductionTier::Easy,
        DeductionTier::Normal,
        DeductionTier::Hard,
    ]
    .iter()
    .map(|&tier| count_solutions_up_to_with_deductions(&puzzle, rules, tier, 10).unwrap())
    .collect();

    // All tiers should agree on the count
    assert!(
        counts.iter().all(|&c| c == counts[0]),
        "Deduction tiers produced inconsistent counts: {:?}",
        counts
    );
}
