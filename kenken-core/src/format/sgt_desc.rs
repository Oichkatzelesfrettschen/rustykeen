use crate::error::CoreError;
use crate::puzzle::{Cage, CellId, Puzzle};
use crate::rules::{Op, Ruleset};

#[derive(Debug, thiserror::Error)]
pub enum SgtDescError {
    #[error("expected ',' after block structure")]
    MissingComma,

    #[error("invalid character in block structure")]
    InvalidBlockChar,

    #[error("block structure: too much data")]
    BlockTooMuchData,

    #[error("block structure: not enough data")]
    BlockNotEnoughData,

    #[error("unexpected end of clue stream")]
    CluesTooFew,

    #[error("too many clues for block structure")]
    CluesTooMany,

    #[error("unrecognized clue type")]
    ClueTypeUnknown,

    #[error("subtraction/division cages must have area 2")]
    SubDivMustBeTwoCell,

    #[error("invalid target number")]
    InvalidTarget,

    #[error(transparent)]
    Core(#[from] CoreError),
}

/// Parse the upstream sgt-puzzles Keen "desc" format into a `Puzzle`.
///
/// Notes:
/// - The upstream format does not explicitly represent 1-cell cages with an `Eq` op.
/// - This parser maps any 1-cell cage to `Op::Eq` regardless of clue type.
pub fn parse_keen_desc(n: u8, desc: &str) -> Result<Puzzle, SgtDescError> {
    if !(1..=16).contains(&n) {
        return Err(CoreError::InvalidGridSize(n).into());
    }

    let a = (n as usize) * (n as usize);
    let mut it = desc.chars().peekable();
    let mut dsf = Dsu::new(a);

    parse_block_structure(&mut it, n, &mut dsf)?;

    if it.next() != Some(',') {
        return Err(SgtDescError::MissingComma);
    }

    let (min_of, size_of) = dsf.component_mins_and_sizes();

    let mut cages_by_min: Vec<(usize, Cage)> = Vec::new();
    let mut members_by_min: Vec<Vec<CellId>> = (0..a).map(|_| Vec::new()).collect();
    for (idx, &min) in min_of.iter().enumerate() {
        members_by_min[min].push(CellId(idx as u16));
    }
    for (min, &cage_size) in size_of.iter().enumerate() {
        if cage_size == 0 {
            continue;
        }
        let (op, target) = parse_clue(&mut it, cage_size)?;
        let members = core::mem::take(&mut members_by_min[min]);
        let cage_op = if members.len() == 1 { Op::Eq } else { op };
        cages_by_min.push((
            min,
            Cage {
                cells: members.into(),
                op: cage_op,
                target,
            },
        ));
    }

    if it.peek().is_some() {
        return Err(SgtDescError::CluesTooMany);
    }

    cages_by_min.sort_by_key(|(min, _)| *min);
    let puzzle = Puzzle {
        n,
        cages: cages_by_min.into_iter().map(|(_, cage)| cage).collect(),
    };

    puzzle.validate(Ruleset::keen_baseline())?;
    Ok(puzzle)
}

/// Encode a `Puzzle` into the upstream sgt-puzzles Keen "desc" format.
///
/// This is intended for corpus tooling and compatibility tests.
pub fn encode_keen_desc(puzzle: &Puzzle, rules: Ruleset) -> Result<String, CoreError> {
    puzzle.validate(rules)?;
    let n = puzzle.n as usize;
    let a = n * n;

    let mut cage_of_cell = vec![usize::MAX; a];
    for (cage_idx, cage) in puzzle.cages.iter().enumerate() {
        for cell in &cage.cells {
            cage_of_cell[cell.0 as usize] = cage_idx;
        }
    }

    let mut edges = Vec::with_capacity(2 * n * (n - 1));
    // Internal vertical edges in reading order.
    for y in 0..n {
        for x in 0..(n - 1) {
            let p0 = y * n + x;
            let p1 = y * n + x + 1;
            edges.push(cage_of_cell[p0] != cage_of_cell[p1]);
        }
    }
    // Internal horizontal edges in transposed reading order (matches upstream).
    for x in 0..n {
        for y in 0..(n - 1) {
            let p0 = y * n + x;
            let p1 = (y + 1) * n + x;
            edges.push(cage_of_cell[p0] != cage_of_cell[p1]);
        }
    }
    debug_assert_eq!(edges.len(), 2 * n * (n - 1));

    let mut raw = String::new();
    let mut currrun = 0usize;
    for i in 0..=edges.len() {
        let is_edge = if i == edges.len() { true } else { edges[i] };
        if is_edge {
            while currrun > 25 {
                raw.push('z');
                currrun -= 25;
            }
            if currrun == 0 {
                raw.push('_');
            } else {
                raw.push((b'a' - 1 + (currrun as u8)) as char);
            }
            currrun = 0;
        } else {
            currrun += 1;
        }
    }

    let block = compress_runs(&raw);

    // Clues are ordered by minimal cell id per cage.
    let mut cages = puzzle.cages.clone();
    cages.sort_by_key(|c| c.cells.iter().map(|c| c.0).min().unwrap_or(u16::MAX));

    let mut out = String::new();
    out.push_str(&block);
    out.push(',');
    for cage in cages {
        let clue_op = match cage.op {
            Op::Add => 'a',
            Op::Mul => 'm',
            Op::Sub => 's',
            Op::Div => 'd',
            Op::Eq => 'a', // singleton cages aren't explicit upstream; use addition as a degenerate case
        };
        out.push(clue_op);
        out.push_str(&cage.target.to_string());
    }

    Ok(out)
}

fn parse_block_structure<I: Iterator<Item = char>>(
    it: &mut core::iter::Peekable<I>,
    n: u8,
    dsf: &mut Dsu,
) -> Result<(), SgtDescError> {
    let w = n as usize;
    let mut pos = 0usize;
    let mut repc = 0usize;
    let mut repn = 0usize;

    while let Some(&ch) = it.peek() {
        if repn == 0 && ch == ',' {
            break;
        }

        let c = if repn > 0 {
            repn -= 1;
            repc
        } else {
            let ch = it.next().ok_or(SgtDescError::InvalidBlockChar)?;
            if ch == '_' {
                0
            } else if ch.is_ascii_lowercase() {
                (ch as u8 - b'a' + 1) as usize
            } else {
                return Err(SgtDescError::InvalidBlockChar);
            }
        };

        // Optional run repetition count (e.g., "_12").
        if repn == 0 {
            let mut digits = String::new();
            while let Some(&d) = it.peek() {
                if d.is_ascii_digit() {
                    digits.push(d);
                    it.next();
                } else {
                    break;
                }
            }
            if !digits.is_empty() {
                repc = c;
                repn = digits
                    .parse::<usize>()
                    .map_err(|_| SgtDescError::InvalidBlockChar)?;
                repn = repn.saturating_sub(1);
            }
        }

        let adv = c != 25;
        let mut remaining = c;
        while remaining > 0 {
            if pos >= 2 * w * (w - 1) {
                return Err(SgtDescError::BlockTooMuchData);
            }
            let (p0, p1) = edge_cells(w, pos);
            dsf.union(p0, p1);
            pos += 1;
            remaining -= 1;
        }

        if adv {
            pos += 1;
            if pos > 2 * w * (w - 1) + 1 {
                return Err(SgtDescError::BlockTooMuchData);
            }
        }
    }

    if pos != 2 * w * (w - 1) + 1 {
        return Err(SgtDescError::BlockNotEnoughData);
    }

    Ok(())
}

fn parse_clue<I: Iterator<Item = char>>(
    it: &mut core::iter::Peekable<I>,
    cage_size: usize,
) -> Result<(Op, i32), SgtDescError> {
    let opch = it.next().ok_or(SgtDescError::CluesTooFew)?;
    let op = match opch {
        'a' => Op::Add,
        'm' => Op::Mul,
        's' => Op::Sub,
        'd' => Op::Div,
        _ => return Err(SgtDescError::ClueTypeUnknown),
    };

    if matches!(op, Op::Sub | Op::Div) && cage_size != 2 {
        return Err(SgtDescError::SubDivMustBeTwoCell);
    }

    let mut digits = String::new();
    while let Some(&d) = it.peek() {
        if d.is_ascii_digit() || (digits.is_empty() && d == '-') {
            digits.push(d);
            it.next();
        } else {
            break;
        }
    }
    if digits.is_empty() || digits == "-" {
        return Err(SgtDescError::InvalidTarget);
    }
    let target = digits
        .parse::<i32>()
        .map_err(|_| SgtDescError::InvalidTarget)?;
    Ok((op, target))
}

fn edge_cells(w: usize, pos: usize) -> (usize, usize) {
    if pos < w * (w - 1) {
        let y = pos / (w - 1);
        let x = pos % (w - 1);
        let p0 = y * w + x;
        let p1 = y * w + x + 1;
        (p0, p1)
    } else {
        let x = pos / (w - 1) - w;
        let y = pos % (w - 1);
        let p0 = y * w + x;
        let p1 = (y + 1) * w + x;
        (p0, p1)
    }
}

fn compress_runs(s: &str) -> String {
    let mut out = String::new();
    let bytes: Vec<u8> = s.as_bytes().to_vec();
    let mut i = 0usize;
    while i < bytes.len() {
        let c = bytes[i] as char;
        let mut j = i + 1;
        while j < bytes.len() && bytes[j] == bytes[i] {
            j += 1;
        }
        let len = j - i;
        out.push(c);
        match len.cmp(&2) {
            core::cmp::Ordering::Less => {}
            core::cmp::Ordering::Equal => out.push(c),
            core::cmp::Ordering::Greater => out.push_str(&len.to_string()),
        }
        i = j;
    }
    out
}

#[derive(Debug, Clone)]
struct Dsu {
    parent: Vec<usize>,
    size: Vec<usize>,
}

impl Dsu {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
            size: vec![1; n],
        }
    }

    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] == x {
            return x;
        }
        let root = self.find(self.parent[x]);
        self.parent[x] = root;
        root
    }

    fn union(&mut self, a: usize, b: usize) {
        let mut ra = self.find(a);
        let mut rb = self.find(b);
        if ra == rb {
            return;
        }
        if self.size[ra] < self.size[rb] {
            core::mem::swap(&mut ra, &mut rb);
        }
        self.parent[rb] = ra;
        self.size[ra] += self.size[rb];
    }

    fn component_mins_and_sizes(&mut self) -> (Vec<usize>, Vec<usize>) {
        let n = self.parent.len();
        let mut root_min = vec![usize::MAX; n];
        let mut root_size = vec![0usize; n];
        for i in 0..n {
            let r = self.find(i);
            root_min[r] = root_min[r].min(i);
            root_size[r] += 1;
        }
        let mut min_of = vec![0usize; n];
        let mut size_of_min = vec![0usize; n];
        for (i, min_slot) in min_of.iter_mut().enumerate() {
            let r = self.find(i);
            *min_slot = root_min[r];
        }
        for (r, &sz) in root_size.iter().enumerate() {
            let min = root_min[r];
            if sz > 0 && min != usize::MAX {
                size_of_min[min] = sz;
            }
        }
        (min_of, size_of_min)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_encode_small_example() {
        // 2x2 with two horizontal 2-cages:
        // [0 1]
        // [2 3]
        // cages: {0,1} and {2,3}, both add target 3
        let desc = "b__,a3a3";
        let p = parse_keen_desc(2, desc).unwrap();
        let enc = encode_keen_desc(&p, Ruleset::keen_baseline()).unwrap();
        assert_eq!(enc, desc);
    }
}
