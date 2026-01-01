//! Versioned engine-owned snapshots using `rkyv`.
//!
//! This is not the upstream sgt-puzzles “desc” format. Snapshots are intended for:
//! - save states
//! - caches (e.g., generated puzzle banks)
//! - reproducible corpora without re-parsing text formats
//!
use kenken_core::rules::Op;
use kenken_core::{Cage, CellId, Puzzle};

use rkyv::{Archive, Deserialize, Serialize};

use crate::error::IoError;

const SNAPSHOT_MAGIC_V1: [u8; 8] = *b"KEENRKYV";
const SNAPSHOT_ENVELOPE_MAGIC: [u8; 8] = *b"KEENSNAP";
const SNAPSHOT_ENVELOPE_VERSION_V2: u16 = 2;
const SNAPSHOT_ENVELOPE_HEADER_LEN_V2: u16 = 16;

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[rkyv(derive(Debug))]
pub struct SnapshotFileV1 {
    pub magic: [u8; 8],
    pub puzzle: SnapshotPuzzleV1,
}

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[rkyv(derive(Debug))]
pub struct SnapshotPuzzleV1 {
    pub n: u8,
    pub cages: Vec<SnapshotCageV1>,
}

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[rkyv(derive(Debug))]
pub struct SnapshotCageV1 {
    pub cells: Vec<u16>,
    pub op: u8,
    pub target: i32,
}

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[rkyv(derive(Debug))]
pub struct SnapshotPayloadV2 {
    pub rules: SnapshotRulesetV1,
    pub puzzle: SnapshotPuzzleV2,
}

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[rkyv(derive(Debug))]
pub struct SnapshotRulesetV1 {
    pub sub_div_two_cell_only: bool,
    pub require_orthogonal_cage_connectivity: bool,
    pub max_cage_size: u8,
}

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[rkyv(derive(Debug))]
pub struct SnapshotPuzzleV2 {
    pub n: u8,
    pub cages: Vec<SnapshotCageV1>,
}

fn encode_op(op: Op) -> u8 {
    match op {
        Op::Add => 0,
        Op::Mul => 1,
        Op::Sub => 2,
        Op::Div => 3,
        Op::Eq => 4,
    }
}

fn decode_op(op: u8) -> Option<Op> {
    match op {
        0 => Some(Op::Add),
        1 => Some(Op::Mul),
        2 => Some(Op::Sub),
        3 => Some(Op::Div),
        4 => Some(Op::Eq),
        _ => None,
    }
}

impl From<&Puzzle> for SnapshotPuzzleV1 {
    fn from(p: &Puzzle) -> Self {
        let cages = p
            .cages
            .iter()
            .map(|c| SnapshotCageV1 {
                cells: c.cells.iter().map(|id| id.0).collect(),
                op: encode_op(c.op),
                target: c.target,
            })
            .collect();
        Self { n: p.n, cages }
    }
}

impl From<&Puzzle> for SnapshotPuzzleV2 {
    fn from(p: &Puzzle) -> Self {
        Self {
            n: p.n,
            cages: SnapshotPuzzleV1::from(p).cages,
        }
    }
}

impl TryFrom<SnapshotPuzzleV1> for Puzzle {
    type Error = IoError;

    fn try_from(p: SnapshotPuzzleV1) -> Result<Self, Self::Error> {
        let cages = p
            .cages
            .into_iter()
            .map(|c| {
                let op = decode_op(c.op).ok_or(IoError::InvalidSnapshotData)?;
                Ok(Cage {
                    cells: c.cells.into_iter().map(CellId).collect(),
                    op,
                    target: c.target,
                })
            })
            .collect::<Result<Vec<_>, IoError>>()?;
        Ok(Puzzle { n: p.n, cages })
    }
}

impl TryFrom<SnapshotPuzzleV2> for Puzzle {
    type Error = IoError;

    fn try_from(p: SnapshotPuzzleV2) -> Result<Self, Self::Error> {
        Puzzle::try_from(SnapshotPuzzleV1 {
            n: p.n,
            cages: p.cages,
        })
    }
}

pub fn encode_puzzle_v1(puzzle: &Puzzle) -> Result<Vec<u8>, IoError> {
    let file = SnapshotFileV1 {
        magic: SNAPSHOT_MAGIC_V1,
        puzzle: SnapshotPuzzleV1::from(puzzle),
    };
    Ok(rkyv::to_bytes::<rkyv::rancor::Error>(&file)?.to_vec())
}

pub fn decode_puzzle_v1(bytes: &[u8]) -> Result<Puzzle, IoError> {
    let archived = rkyv::access::<ArchivedSnapshotFileV1, rkyv::rancor::Error>(bytes)?;
    if archived.magic != SNAPSHOT_MAGIC_V1 {
        return Err(IoError::InvalidSnapshotMagic);
    }
    let file: SnapshotFileV1 = rkyv::deserialize::<SnapshotFileV1, rkyv::rancor::Error>(archived)?;
    Puzzle::try_from(file.puzzle)
}

pub fn encode_puzzle_v2(
    puzzle: &Puzzle,
    rules: kenken_core::rules::Ruleset,
) -> Result<Vec<u8>, IoError> {
    let payload = SnapshotPayloadV2 {
        rules: SnapshotRulesetV1 {
            sub_div_two_cell_only: rules.sub_div_two_cell_only,
            require_orthogonal_cage_connectivity: rules.require_orthogonal_cage_connectivity,
            max_cage_size: rules.max_cage_size,
        },
        puzzle: SnapshotPuzzleV2::from(puzzle),
    };
    let mut out = Vec::new();
    out.extend_from_slice(&SNAPSHOT_ENVELOPE_MAGIC);
    out.extend_from_slice(&SNAPSHOT_ENVELOPE_VERSION_V2.to_le_bytes());
    out.extend_from_slice(&SNAPSHOT_ENVELOPE_HEADER_LEN_V2.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&rkyv::to_bytes::<rkyv::rancor::Error>(&payload)?);
    Ok(out)
}

pub fn decode_puzzle_v2(bytes: &[u8]) -> Result<(Puzzle, kenken_core::rules::Ruleset), IoError> {
    if bytes.len() < SNAPSHOT_ENVELOPE_HEADER_LEN_V2 as usize {
        return Err(IoError::InvalidSnapshotData);
    }
    let magic: [u8; 8] = bytes[..8]
        .try_into()
        .map_err(|_| IoError::InvalidSnapshotData)?;
    if magic != SNAPSHOT_ENVELOPE_MAGIC {
        return Err(IoError::InvalidSnapshotMagic);
    }
    let version = u16::from_le_bytes(bytes[8..10].try_into().unwrap());
    if version != SNAPSHOT_ENVELOPE_VERSION_V2 {
        return Err(IoError::InvalidSnapshotData);
    }

    let header_len = u16::from_le_bytes(bytes[10..12].try_into().unwrap());
    if header_len != SNAPSHOT_ENVELOPE_HEADER_LEN_V2 {
        return Err(IoError::InvalidSnapshotData);
    }
    let payload_bytes = &bytes[header_len as usize..];
    let archived = rkyv::access::<ArchivedSnapshotPayloadV2, rkyv::rancor::Error>(payload_bytes)?;
    let payload: SnapshotPayloadV2 =
        rkyv::deserialize::<SnapshotPayloadV2, rkyv::rancor::Error>(archived)?;

    let puzzle = Puzzle::try_from(payload.puzzle)?;
    let rules = kenken_core::rules::Ruleset {
        sub_div_two_cell_only: payload.rules.sub_div_two_cell_only,
        require_orthogonal_cage_connectivity: payload.rules.require_orthogonal_cage_connectivity,
        max_cage_size: payload.rules.max_cage_size,
    };
    Ok((puzzle, rules))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotVersion {
    V1,
    V2,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DecodedSnapshot {
    pub version: SnapshotVersion,
    pub puzzle: Puzzle,
    pub rules: Option<kenken_core::rules::Ruleset>,
}

pub fn decode_snapshot(bytes: &[u8]) -> Result<DecodedSnapshot, IoError> {
    // v2+ framing check first.
    if bytes.len() >= SNAPSHOT_ENVELOPE_HEADER_LEN_V2 as usize
        && bytes[..8] == SNAPSHOT_ENVELOPE_MAGIC
    {
        let (puzzle, rules) = decode_puzzle_v2(bytes)?;
        return Ok(DecodedSnapshot {
            version: SnapshotVersion::V2,
            puzzle,
            rules: Some(rules),
        });
    }

    // Legacy v1 (unframed rkyv root).
    let puzzle = decode_puzzle_v1(bytes)?;
    Ok(DecodedSnapshot {
        version: SnapshotVersion::V1,
        puzzle,
        rules: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use kenken_core::rules::Ruleset;

    #[test]
    fn rkyv_roundtrips_puzzle() {
        let puzzle = kenken_core::format::sgt_desc::parse_keen_desc(2, "b__,a3a3").unwrap();
        puzzle.validate(Ruleset::keen_baseline()).unwrap();

        let bytes = encode_puzzle_v1(&puzzle).unwrap();
        let decoded = decode_puzzle_v1(&bytes).unwrap();
        assert_eq!(puzzle, decoded);
    }

    #[test]
    fn decode_snapshot_detects_v1() {
        let puzzle = kenken_core::format::sgt_desc::parse_keen_desc(2, "b__,a3a3").unwrap();
        let bytes = encode_puzzle_v1(&puzzle).unwrap();
        let decoded = decode_snapshot(&bytes).unwrap();
        assert_eq!(decoded.version, SnapshotVersion::V1);
        assert_eq!(decoded.rules, None);
        assert_eq!(decoded.puzzle, puzzle);
    }

    #[test]
    fn v2_roundtrips_and_preserves_rules() {
        let puzzle = kenken_core::format::sgt_desc::parse_keen_desc(2, "b__,a3a3").unwrap();
        let rules = Ruleset::keen_baseline();
        let bytes = encode_puzzle_v2(&puzzle, rules).unwrap();
        let decoded = decode_snapshot(&bytes).unwrap();
        assert_eq!(decoded.version, SnapshotVersion::V2);
        assert_eq!(decoded.rules, Some(rules));
        assert_eq!(decoded.puzzle, puzzle);
    }
}
