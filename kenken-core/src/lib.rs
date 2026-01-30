#![forbid(unsafe_code)]
#![feature(likely_unlikely)]
#![doc = include_str!("../README.md")]

/// BitDomain representation using bitvec (requires `core-bitvec` feature).
///
/// Alternative fixed-size domain for n > 63 grids.
#[cfg(feature = "core-bitvec")]
pub mod domain;

/// Core puzzle validation errors.
pub mod error;

/// SGT-desc format parser for puzzle serialization (requires `format-sgt-desc` feature).
///
/// Module for parsing Simon Tatham's puzzle "desc" format strings into Puzzle objects.
#[cfg(feature = "format-sgt-desc")]
pub mod format;

/// Branch prediction hints for performance optimization.
///
/// Internal utilities for likely/unlikely hints. Part of stable API for consistency.
pub mod hints;

/// Puzzle model: Grid, Cells, Cages, and Coords.
///
/// Core types for representing KenKen puzzles and validation logic.
pub mod puzzle;

/// Rules and operations for cage constraints.
///
/// Defines cage operations (Add, Mul, Sub, Div, Eq) and rule configuration (Ruleset).
pub mod rules;

/// BitDomain representation using bitvec (requires `core-bitvec` feature).
///
/// Alternative fixed-size domain for n > 63 grids.
#[cfg(feature = "core-bitvec")]
pub use crate::domain::BitDomain;

/// Puzzle validation error type.
pub use crate::error::CoreError;

/// Stable core types for puzzle representation.
pub use crate::puzzle::{Cage, CellId, Coord, Puzzle};
