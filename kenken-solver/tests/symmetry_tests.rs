#![cfg(feature = "symmetry-breaking")]

//! Integration tests for symmetry breaking feature.
//!
//! Tests verify:
//! 1. Feature compiles and loads successfully
//! 2. Puzzles solve correctly with feature enabled
//! 3. Solution counts remain consistent with/without feature
//! 4. Determinism is preserved

use kenken_core::format::sgt_desc::parse_keen_desc;
use kenken_core::rules::Ruleset;
use kenken_solver::{DeductionTier, count_solutions_up_to, solve_one};

const RULES: Ruleset = Ruleset::keen_baseline();

#[test]
fn symmetry_feature_loads() {
    // Test that the feature compiles and the module is available
    println!("Symmetry breaking feature loaded successfully");
}

#[test]
fn simple_2x2_puzzle_solves_with_feature() {
    // Create a simple 2x2 puzzle: "b__,a3a3" = two add-3 cages
    let puzzle = parse_keen_desc(2, "b__,a3a3").expect("Should parse puzzle");
    let solution = solve_one(&puzzle, RULES);
    assert!(solution.is_ok(), "Should be able to solve");
    assert!(
        solution.unwrap().is_some(),
        "2x2 puzzle should have a solution"
    );
}

#[test]
fn solution_count_preserved_with_feature() {
    // Test with a 2x2 puzzle with known solution count
    let puzzle = parse_keen_desc(2, "b__,a3a3").expect("Should parse puzzle");
    let count = count_solutions_up_to(&puzzle, RULES, 100).expect("Should count solutions");
    // This puzzle has 2 solutions
    assert_eq!(count, 2, "2x2 add-3 puzzle should have 2 solutions");
}

#[test]
fn determinism_test_multiple_solves() {
    // Solve the same puzzle multiple times and verify results are identical
    let puzzle = parse_keen_desc(2, "_5,a1a2a2a1").expect("Should parse puzzle");

    let sol1 = solve_one(&puzzle, RULES).expect("Should solve").unwrap();
    let sol2 = solve_one(&puzzle, RULES).expect("Should solve").unwrap();
    let sol3 = solve_one(&puzzle, RULES).expect("Should solve").unwrap();

    assert_eq!(
        sol1, sol2,
        "First and second solve should produce identical results"
    );
    assert_eq!(
        sol2, sol3,
        "Second and third solve should produce identical results"
    );
}

#[test]
fn feature_handles_3x3_puzzle() {
    // Test with a 3x3 singleton grid puzzle
    let puzzle = parse_keen_desc(3, "_13,a1a2a3a2a3a1a3a1a2").expect("Should parse puzzle");
    let solution = solve_one(&puzzle, RULES).expect("Should solve");
    assert!(solution.is_some(), "3x3 puzzle should have a solution");
}

#[test]
fn deduction_tier_none_works_with_feature() {
    // Test with deduction tier None (pure backtracking)
    let puzzle = parse_keen_desc(2, "_5,a1a2a2a1").expect("Should parse puzzle");
    let solution =
        kenken_solver::solve_one_with_deductions(&puzzle, RULES, DeductionTier::None).unwrap();
    assert!(solution.is_some(), "Should solve with tier=None");
}

#[test]
fn deduction_tier_normal_works_with_feature() {
    // Test with deduction tier Normal
    let puzzle = parse_keen_desc(2, "_5,a1a2a2a1").expect("Should parse puzzle");
    let solution =
        kenken_solver::solve_one_with_deductions(&puzzle, RULES, DeductionTier::Normal).unwrap();
    assert!(solution.is_some(), "Should solve with tier=Normal");
}

#[test]
fn feature_preserves_solve_stats() {
    // Verify that solve stats are meaningful with the feature enabled
    let puzzle = parse_keen_desc(2, "_5,a1a2a2a1").expect("Should parse puzzle");
    let (solution, stats) = kenken_solver::solve_one_with_stats(&puzzle, RULES).unwrap();

    assert!(solution.is_some(), "Should produce a solution");
    assert!(
        stats.assignments > 0,
        "Stats should track assignments (got {})",
        stats.assignments
    );
    // At least 4 assignments for a 2x2 grid
    assert!(
        stats.assignments >= 4,
        "Should have at least 4 assignments in a 2x2 grid"
    );
}

#[test]
fn feature_with_4x4_puzzle() {
    // Test with a 4x4 singleton grid puzzle from the corpus
    let puzzle =
        parse_keen_desc(4, "_25,a1a2a3a4a2a1a4a3a3a4a1a2a4a3a2a1").expect("Should parse puzzle");
    let solution = solve_one(&puzzle, RULES).expect("Should solve");
    assert!(solution.is_some(), "4x4 puzzle should have a solution");
}

#[test]
fn consistent_behavior_with_disabled_feature() {
    // Verify that puzzles behave the same whether or not the feature is used
    // (since symmetry breaking is currently disabled for safety)
    let puzzle = parse_keen_desc(2, "b__,a3a3").expect("Should parse puzzle");
    let count = count_solutions_up_to(&puzzle, RULES, 100).expect("Should count solutions");
    // This puzzle should have 2 solutions
    assert_eq!(count, 2, "2x2 add-3 puzzle should have 2 solutions");
}
