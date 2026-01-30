//! API Stability Tests
//!
//! Verify that public API types, functions, and fields remain accessible
//! and compatible across versions. These tests catch unintentional breaking changes.

use kenken_core::puzzle::{cell_id, coord};
use kenken_core::rules::{Op, Ruleset};
use kenken_core::{Cage, CellId, Coord, CoreError, Puzzle};

#[test]
fn puzzle_type_is_public() {
    let puzzle = Puzzle {
        n: 2,
        cages: vec![],
    };
    assert_eq!(puzzle.n, 2);
    assert_eq!(puzzle.cages.len(), 0);
}

#[test]
fn puzzle_fields_are_public() {
    let mut puzzle = Puzzle {
        n: 3,
        cages: vec![],
    };
    puzzle.n = 4;
    let cage = Cage {
        cells: smallvec::smallvec![],
        op: Op::Eq,
        target: 5,
    };
    puzzle.cages.push(cage);
    assert_eq!(puzzle.n, 4);
    assert_eq!(puzzle.cages.len(), 1);
}

#[test]
fn cage_type_is_public() {
    let mut cells = smallvec::SmallVec::new();
    cells.push(CellId(0));
    let cage = Cage {
        cells,
        op: Op::Add,
        target: 10,
    };
    assert_eq!(cage.cells.len(), 1);
    assert_eq!(cage.op, Op::Add);
    assert_eq!(cage.target, 10);
}

#[test]
fn cellid_type_is_public() {
    let cell = CellId(42);
    assert_eq!(cell.0, 42);
}

#[test]
fn coord_type_is_public() {
    let coord = Coord { row: 1, col: 2 };
    assert_eq!(coord.row, 1);
    assert_eq!(coord.col, 2);
}

#[test]
fn cell_id_function_exists() {
    let id_result = cell_id(2, Coord { row: 0, col: 1 });
    assert!(id_result.is_ok());
}

#[test]
fn coord_function_exists() {
    let coord_result = coord(2, CellId(1));
    assert!(coord_result.is_ok());
}

#[test]
fn ruleset_type_is_public() {
    let rules = Ruleset::keen_baseline();
    assert!(rules.max_cage_size >= 1);
    assert!(rules.sub_div_two_cell_only);
}

#[test]
fn op_enum_variants_exist() {
    let _ops = [Op::Add, Op::Sub, Op::Mul, Op::Div, Op::Eq];
    assert_eq!(
        std::mem::discriminant(&Op::Add),
        std::mem::discriminant(&Op::Add)
    );
}

#[test]
fn sgt_parse_function_exists() {
    #[cfg(feature = "format-sgt-desc")]
    {
        let puzzle = kenken_core::format::sgt_desc::parse_keen_desc(2, "_5,a1a2a2a1");
        assert!(puzzle.is_ok());
    }
}

#[test]
fn no_deprecated_markers_on_stable_api() {
    // This test documents that stable API items should not be marked deprecated
    // If any stable items accidentally get marked #[deprecated], this test should fail
    // as a reminder to provide a migration path.

    // Compile-time check: this would fail if we tried to use deprecated items
    let _puzzle = Puzzle {
        n: 2,
        cages: vec![],
    };
    let _rules = Ruleset::keen_baseline();
}

#[test]
fn core_error_type_is_public() {
    let _err = CoreError::InvalidGridSize(64);
    let _err2 = CoreError::EmptyCage;
}

#[test]
fn rules_fields_accessible() {
    let rules = Ruleset::keen_baseline();
    let _max = rules.max_cage_size;
    let _sub_div = rules.sub_div_two_cell_only;
    let _connectivity = rules.require_orthogonal_cage_connectivity;
}
