use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("grid size N={0} not supported by this configuration")]
    InvalidGridSize(u8),

    #[error("cage has no cells")]
    EmptyCage,

    #[error("cell id {cell} out of range for N={n}")]
    CellOutOfRange { n: u8, cell: CellId },

    #[error("cell id {0} appears in more than one cage")]
    CellDuplicated(CellId),

    #[error("grid cell {0} is not covered by any cage")]
    CellUncovered(CellId),

    #[error("cage operation {op:?} not valid for cage size {len}")]
    InvalidOpForCageSize { op: crate::rules::Op, len: usize },

    #[error("subtraction/division cages must have exactly 2 cells under the baseline ruleset")]
    SubDivMustBeTwoCell,

    #[error("cage has {len} cells, exceeding max {max} for this ruleset")]
    CageTooLarge { len: usize, max: u8 },

    #[error("Eq cages must have target in 1..=N")]
    EqTargetOutOfRange,

    #[error("cage target must be non-zero")]
    TargetMustBeNonZero,

    #[error("cage is not orthogonally connected")]
    CageNotConnected,
}

use crate::puzzle::CellId;
