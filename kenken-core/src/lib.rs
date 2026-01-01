#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

#[cfg(feature = "core-bitvec")]
pub mod domain;
pub mod error;
#[cfg(feature = "format-sgt-desc")]
pub mod format;
pub mod puzzle;
pub mod rules;

#[cfg(feature = "core-bitvec")]
pub use crate::domain::BitDomain;
pub use crate::error::CoreError;
pub use crate::puzzle::{Cage, CellId, Coord, Puzzle};
