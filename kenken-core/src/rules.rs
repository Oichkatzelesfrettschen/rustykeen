#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Op {
    Add,
    Mul,
    Sub,
    Div,
    Eq,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Ruleset {
    pub sub_div_two_cell_only: bool,
    pub require_orthogonal_cage_connectivity: bool,
    pub max_cage_size: u8,
}

impl Ruleset {
    pub const fn keen_baseline() -> Self {
        Self {
            sub_div_two_cell_only: true,
            require_orthogonal_cage_connectivity: true,
            max_cage_size: 6,
        }
    }
}
