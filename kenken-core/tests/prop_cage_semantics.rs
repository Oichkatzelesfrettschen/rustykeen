//! Property-based tests for cage arithmetic semantics.
//!
//! These tests verify:
//! - Tuple enumeration correctness (all tuples satisfy arithmetic constraint)
//! - Cell coordinate roundtrip
//! - Cage validation invariants

use kenken_core::puzzle::{Cage, CellId, Coord, cell_id, coord};
use kenken_core::rules::{Op, Ruleset};
use proptest::prelude::*;
use smallvec::SmallVec;

proptest! {
    /// All enumerated tuples for Add cages must sum to the target.
    #[test]
    fn add_cage_tuples_sum_to_target(
        n in 2u8..=6,
        size in 2usize..=4,
        target in 2i32..=24,
    ) {
        let cells: SmallVec<[CellId; 6]> = (0..size).map(|i| CellId(i as u16)).collect();
        let cage = Cage {
            cells,
            op: Op::Add,
            target,
        };

        let rules = Ruleset::keen_baseline();
        if let Ok(Some(tuples)) = cage.valid_permutations(n, rules, 10000) {
            for tuple in &tuples {
                let sum: i32 = tuple.iter().map(|&v| v as i32).sum();
                prop_assert_eq!(sum, target, "Tuple {:?} sum {} != target {}", tuple, sum, target);
            }
        }
    }

    /// All enumerated tuples for Mul cages must multiply to the target.
    #[test]
    fn mul_cage_tuples_product_equals_target(
        n in 2u8..=6,
        size in 2usize..=3,
        target in 1i32..=100,
    ) {
        let cells: SmallVec<[CellId; 6]> = (0..size).map(|i| CellId(i as u16)).collect();
        let cage = Cage {
            cells,
            op: Op::Mul,
            target,
        };

        let rules = Ruleset::keen_baseline();
        if let Ok(Some(tuples)) = cage.valid_permutations(n, rules, 10000) {
            for tuple in &tuples {
                let prod: i32 = tuple.iter().fold(1, |acc, &v| acc * v as i32);
                prop_assert_eq!(prod, target, "Tuple {:?} product {} != target {}", tuple, prod, target);
            }
        }
    }

    /// All enumerated tuples for Sub cages must have absolute difference equal to target.
    #[test]
    fn sub_cage_tuples_diff_equals_target(
        n in 2u8..=9,
        target in 1i32..=8,
    ) {
        let cells: SmallVec<[CellId; 6]> = [CellId(0), CellId(1)].into_iter().collect();
        let cage = Cage {
            cells,
            op: Op::Sub,
            target,
        };

        let rules = Ruleset::keen_baseline();
        if let Ok(Some(tuples)) = cage.valid_permutations(n, rules, 1000) {
            for tuple in &tuples {
                prop_assert_eq!(tuple.len(), 2);
                let diff = (tuple[0] as i32 - tuple[1] as i32).abs();
                prop_assert_eq!(diff, target, "Tuple {:?} diff {} != target {}", tuple, diff, target);
            }
        }
    }

    /// All enumerated tuples for Div cages must have quotient equal to target.
    #[test]
    fn div_cage_tuples_quotient_equals_target(
        n in 2u8..=9,
        target in 1i32..=8,
    ) {
        let cells: SmallVec<[CellId; 6]> = [CellId(0), CellId(1)].into_iter().collect();
        let cage = Cage {
            cells,
            op: Op::Div,
            target,
        };

        let rules = Ruleset::keen_baseline();
        if let Ok(Some(tuples)) = cage.valid_permutations(n, rules, 1000) {
            for tuple in &tuples {
                prop_assert_eq!(tuple.len(), 2);
                let (num, den) = if tuple[0] >= tuple[1] {
                    (tuple[0], tuple[1])
                } else {
                    (tuple[1], tuple[0])
                };
                if den != 0 {
                    let quot = num as i32 / den as i32;
                    let rem = num as i32 % den as i32;
                    prop_assert_eq!(rem, 0, "Tuple {:?} not evenly divisible", tuple);
                    prop_assert_eq!(quot, target, "Tuple {:?} quotient {} != target {}", tuple, quot, target);
                }
            }
        }
    }

    /// Cell coordinate roundtrip: cell_id -> coord -> cell_id is identity.
    #[test]
    fn cell_coord_roundtrip(
        n in 2u8..=16,
        row in 0u8..16,
        col in 0u8..16,
    ) {
        prop_assume!(row < n && col < n);
        let c = Coord { row, col };
        let id = cell_id(n, c).unwrap();
        let back = coord(n, id).unwrap();
        prop_assert_eq!(back, c);
    }

    /// Tuple values are always in valid range [1, n].
    #[test]
    fn tuple_values_in_range(
        n in 2u8..=6,
        size in 1usize..=3,
        target in 1i32..=50,
    ) {
        let cells: SmallVec<[CellId; 6]> = (0..size).map(|i| CellId(i as u16)).collect();
        let op = if size == 1 { Op::Eq } else { Op::Add };
        let cage = Cage { cells, op, target };

        let rules = Ruleset::keen_baseline();
        if let Ok(Some(tuples)) = cage.valid_permutations(n, rules, 1000) {
            for tuple in &tuples {
                for &v in tuple.iter() {
                    prop_assert!(v >= 1 && v <= n, "Value {} out of range [1, {}]", v, n);
                }
            }
        }
    }
}
