//! Text snapshot support using the upstream-compatible SGT `desc` format.
//!
//! This backend provides a human-readable/non-binary persistence path alongside
//! `rkyv` snapshots for portability and audit workflows.

use kenken_core::{Puzzle, rules::Ruleset};

use crate::error::IoError;

/// Encode a puzzle to SGT `desc` text.
pub fn encode_puzzle_sgt_desc(puzzle: &Puzzle, rules: Ruleset) -> Result<String, IoError> {
    Ok(kenken_core::format::sgt_desc::encode_keen_desc(
        puzzle, rules,
    )?)
}

/// Decode a puzzle from SGT `desc` text.
pub fn decode_puzzle_sgt_desc(n: u8, desc: &str) -> Result<Puzzle, IoError> {
    Ok(kenken_core::format::sgt_desc::parse_keen_desc(n, desc)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sgt_desc_roundtrip() {
        let rules = Ruleset::keen_baseline();
        let puzzle = kenken_core::format::sgt_desc::parse_keen_desc(2, "b__,a3a3").unwrap();
        puzzle.validate(rules).unwrap();

        let encoded = encode_puzzle_sgt_desc(&puzzle, rules).unwrap();
        let decoded = decode_puzzle_sgt_desc(2, &encoded).unwrap();
        assert_eq!(decoded, puzzle);
    }
}
