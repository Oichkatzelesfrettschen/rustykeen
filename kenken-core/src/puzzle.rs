use smallvec::SmallVec;

use crate::error::CoreError;
use crate::rules::{Op, Ruleset};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CellId(pub u16);

impl core::fmt::Display for CellId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Coord {
    pub row: u8,
    pub col: u8,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cage {
    pub cells: SmallVec<[CellId; 6]>,
    pub op: Op,
    pub target: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Puzzle {
    pub n: u8,
    pub cages: Vec<Cage>,
}

#[cfg(feature = "perf-assertions")]
mod _layout_assertions {
    use static_assertions::{assert_eq_align, assert_eq_size};

    use super::{CellId, Coord};

    assert_eq_size!(CellId, u16);
    assert_eq_align!(CellId, u16);
    assert_eq_size!(Coord, [u8; 2]);
}

impl Puzzle {
    pub fn validate(&self, rules: Ruleset) -> Result<(), CoreError> {
        let n = self.n;

        // Feature-gated grid size validation
        #[cfg(not(any(feature = "core-u64", feature = "core-bitvec")))]
        if !(1..=31).contains(&n) {
            return Err(CoreError::InvalidGridSize(n));
        }

        #[cfg(all(feature = "core-u64", not(feature = "core-bitvec")))]
        if !(1..=63).contains(&n) {
            return Err(CoreError::InvalidGridSize(n));
        }

        #[cfg(feature = "core-bitvec")]
        if !(1..=255).contains(&n) {
            return Err(CoreError::InvalidGridSize(n));
        }
        let a = (n as usize) * (n as usize);

        let mut seen = vec![false; a];
        for cage in &self.cages {
            cage.validate_shape(n, rules)?;
            for &cell in &cage.cells {
                let idx = cell_index(n, cell)?;
                if seen[idx] {
                    return Err(CoreError::CellDuplicated(cell));
                }
                seen[idx] = true;
            }
        }

        for (idx, covered) in seen.into_iter().enumerate() {
            if !covered {
                return Err(CoreError::CellUncovered(CellId(idx as u16)));
            }
        }

        Ok(())
    }
}

impl Cage {
    pub fn validate_shape(&self, n: u8, rules: Ruleset) -> Result<(), CoreError> {
        if self.cells.is_empty() {
            return Err(CoreError::EmptyCage);
        }

        if self.cells.len() > rules.max_cage_size as usize {
            return Err(CoreError::CageTooLarge {
                len: self.cells.len(),
                max: rules.max_cage_size,
            });
        }

        match (self.op, self.cells.len()) {
            (Op::Eq, 1) => {}
            (Op::Eq, len) => {
                return Err(CoreError::InvalidOpForCageSize { op: self.op, len });
            }
            (Op::Sub | Op::Div, len) if rules.sub_div_two_cell_only && len != 2 => {
                return Err(CoreError::SubDivMustBeTwoCell);
            }
            (_, _) => {}
        }

        if self.target == 0 {
            return Err(CoreError::TargetMustBeNonZero);
        }
        if self.op == Op::Eq && !(1..=(n as i32)).contains(&self.target) {
            return Err(CoreError::EqTargetOutOfRange);
        }

        for &cell in &self.cells {
            cell_index(n, cell)?;
        }

        if rules.require_orthogonal_cage_connectivity && !is_orthogonally_connected(n, &self.cells)
        {
            return Err(CoreError::CageNotConnected);
        }

        Ok(())
    }

    /// Enumerate value assignments (ordered tuples) that satisfy this cage's arithmetic constraint.
    ///
    /// This helper is intended for tuple-based encodings (e.g., SAT allowlists) where encoding
    /// a set of satisfying assignments is cheaper than building binary arithmetic circuits.
    ///
    /// Notes:
    /// - Returned tuples are **ordered**: each tuple position corresponds to `self.cells[i]`.
    /// - Latin row/column uniqueness constraints are *not* baked into this enumeration; it is purely
    ///   arithmetic. Consumers that want additional pruning can filter based on coordinates.
    /// - If the number of satisfying tuples exceeds `max_tuples`, this returns `Ok(None)` so callers
    ///   can fall back to a different strategy.
    pub fn valid_permutations(
        &self,
        n: u8,
        rules: Ruleset,
        max_tuples: usize,
    ) -> Result<Option<Vec<SmallVec<[u8; 6]>>>, CoreError> {
        let len = self.cells.len();
        if len == 0 {
            return Err(CoreError::EmptyCage);
        }

        if rules.sub_div_two_cell_only && matches!(self.op, Op::Sub | Op::Div) && len != 2 {
            return Err(CoreError::SubDivMustBeTwoCell);
        }
        if self.op == Op::Eq && len != 1 {
            return Err(CoreError::InvalidOpForCageSize { op: self.op, len });
        }

        let target = self.target;
        let n_i32 = n as i32;
        let max_tuples = max_tuples.max(1);

        let mut out: Vec<SmallVec<[u8; 6]>> = Vec::new();

        match self.op {
            Op::Eq => {
                if !(1..=n_i32).contains(&target) {
                    Ok(Some(out))
                } else {
                    let mut t = SmallVec::<[u8; 6]>::new();
                    t.push(target as u8);
                    out.push(t);
                    Ok(Some(out))
                }
            }
            Op::Sub => {
                if target <= 0 {
                    Ok(Some(out))
                } else {
                    for a in 1..=n {
                        for b in 1..=n {
                            if (a as i32 - b as i32).abs() == target {
                                let mut t = SmallVec::<[u8; 6]>::with_capacity(2);
                                t.push(a);
                                t.push(b);
                                out.push(t);
                                if out.len() >= max_tuples {
                                    return Ok(None);
                                }
                            }
                        }
                    }
                    Ok(Some(out))
                }
            }
            Op::Div => {
                if target <= 0 {
                    Ok(Some(out))
                } else {
                    for a in 1..=n {
                        for b in 1..=n {
                            let (num, den) = if a >= b { (a, b) } else { (b, a) };
                            if den != 0 && (num as i32) == (den as i32).saturating_mul(target) {
                                let mut t = SmallVec::<[u8; 6]>::with_capacity(2);
                                t.push(a);
                                t.push(b);
                                out.push(t);
                                if out.len() >= max_tuples {
                                    return Ok(None);
                                }
                            }
                        }
                    }
                    Ok(Some(out))
                }
            }
            Op::Add => {
                if target <= 0 {
                    Ok(Some(out))
                } else {
                    #[allow(clippy::too_many_arguments)]
                    fn rec(
                        n: u8,
                        target: i32,
                        pos: usize,
                        len: usize,
                        sum: i32,
                        cur: &mut SmallVec<[u8; 6]>,
                        out: &mut Vec<SmallVec<[u8; 6]>>,
                        max_tuples: usize,
                    ) -> bool {
                        if pos == len {
                            if sum == target {
                                out.push(cur.clone());
                                if out.len() >= max_tuples {
                                    return false;
                                }
                            }
                            return true;
                        }
                        for v in 1..=n {
                            let next_sum = sum + v as i32;
                            if next_sum > target {
                                continue;
                            }
                            cur.push(v);
                            if !rec(n, target, pos + 1, len, next_sum, cur, out, max_tuples) {
                                return false;
                            }
                            cur.pop();
                        }
                        true
                    }

                    let mut cur = SmallVec::<[u8; 6]>::with_capacity(len);
                    if !rec(n, target, 0, len, 0, &mut cur, &mut out, max_tuples) {
                        return Ok(None);
                    }
                    Ok(Some(out))
                }
            }
            Op::Mul => {
                if target <= 0 {
                    Ok(Some(out))
                } else {
                    #[allow(clippy::too_many_arguments)]
                    fn rec(
                        n: u8,
                        target: i32,
                        pos: usize,
                        len: usize,
                        prod: i32,
                        cur: &mut SmallVec<[u8; 6]>,
                        out: &mut Vec<SmallVec<[u8; 6]>>,
                        max_tuples: usize,
                    ) -> bool {
                        if pos == len {
                            if prod == target {
                                out.push(cur.clone());
                                if out.len() >= max_tuples {
                                    return false;
                                }
                            }
                            return true;
                        }
                        for v in 1..=n {
                            let next = prod.saturating_mul(v as i32);
                            if next == 0 {
                                continue;
                            }
                            if target % next != 0 {
                                continue;
                            }
                            cur.push(v);
                            if !rec(n, target, pos + 1, len, next, cur, out, max_tuples) {
                                return false;
                            }
                            cur.pop();
                        }
                        true
                    }

                    let mut cur = SmallVec::<[u8; 6]>::with_capacity(len);
                    if !rec(n, target, 0, len, 1, &mut cur, &mut out, max_tuples) {
                        return Ok(None);
                    }
                    Ok(Some(out))
                }
            }
        }
    }
}

#[cfg(test)]
mod tuple_enum_tests {
    use super::{Cage, CellId};
    use crate::rules::{Op, Ruleset};

    #[test]
    fn enumerates_two_cell_sub_pairs() {
        let cage = Cage {
            cells: [CellId(0), CellId(1)].into_iter().collect(),
            op: Op::Sub,
            target: 1,
        };
        let tuples = cage
            .valid_permutations(4, Ruleset::keen_baseline(), 1024)
            .unwrap()
            .unwrap();
        assert!(tuples.iter().any(|t| t.as_slice() == [1, 2]));
        assert!(tuples.iter().any(|t| t.as_slice() == [2, 1]));
    }

    #[test]
    fn threshold_returns_none() {
        let cage = Cage {
            cells: [CellId(0), CellId(1)].into_iter().collect(),
            op: Op::Add,
            target: 5,
        };
        // For n=9, there are many ordered pairs summing to 5; cap to 1 to force overflow.
        assert!(
            cage.valid_permutations(9, Ruleset::keen_baseline(), 1)
                .unwrap()
                .is_none()
        );
    }
}

pub fn cell_id(n: u8, coord: Coord) -> Result<CellId, CoreError> {
    if coord.row >= n || coord.col >= n {
        return Err(CoreError::CellOutOfRange {
            n,
            cell: CellId((coord.row as u16) * (n as u16) + coord.col as u16),
        });
    }
    Ok(CellId((coord.row as u16) * (n as u16) + coord.col as u16))
}

pub fn coord(n: u8, cell: CellId) -> Result<Coord, CoreError> {
    let idx = cell_index(n, cell)?;
    Ok(Coord {
        row: (idx / (n as usize)) as u8,
        col: (idx % (n as usize)) as u8,
    })
}

fn cell_index(n: u8, cell: CellId) -> Result<usize, CoreError> {
    let a = (n as usize) * (n as usize);
    let idx = cell.0 as usize;
    if idx >= a {
        return Err(CoreError::CellOutOfRange { n, cell });
    }
    Ok(idx)
}

fn is_orthogonally_connected(n: u8, cells: &[CellId]) -> bool {
    if cells.len() <= 1 {
        return true;
    }

    let a = (n as usize) * (n as usize);
    let mut in_cage = vec![false; a];
    for &c in cells {
        let idx = c.0 as usize;
        if idx < a {
            in_cage[idx] = true;
        }
    }

    let start = cells[0].0 as usize;
    if start >= a {
        return false;
    }

    let mut stack = vec![start];
    let mut visited = vec![false; a];
    visited[start] = true;
    let mut count = 0usize;

    while let Some(idx) = stack.pop() {
        if !in_cage[idx] {
            continue;
        }
        count += 1;
        let r = idx / (n as usize);
        let c = idx % (n as usize);

        let mut push = |nr: isize, nc: isize| {
            if nr < 0 || nc < 0 {
                return;
            }
            let nr = nr as usize;
            let nc = nc as usize;
            if nr >= n as usize || nc >= n as usize {
                return;
            }
            let nidx = nr * (n as usize) + nc;
            if !visited[nidx] {
                visited[nidx] = true;
                stack.push(nidx);
            }
        };

        push(r as isize - 1, c as isize);
        push(r as isize + 1, c as isize);
        push(r as isize, c as isize - 1);
        push(r as isize, c as isize + 1);
    }

    count == cells.len()
}

// ============================================================
// Kani Verification Harnesses
// ============================================================
//
// Run with: cargo kani --harness <harness_name>
// Or run all: cargo kani

#[cfg(kani)]
mod kani_verification {
    use super::*;

    /// Proves cell_id and coord form a roundtrip bijection for valid inputs.
    ///
    /// For any valid grid size N and coordinate (row, col) where row < N and col < N:
    /// - cell_id(N, Coord{row, col}) produces a CellId
    /// - coord(N, that_cell_id) returns the original (row, col)
    #[kani::proof]
    fn cell_coord_roundtrip() {
        let n: u8 = kani::any();
        kani::assume(n >= 2 && n <= 9);

        let row: u8 = kani::any();
        let col: u8 = kani::any();
        kani::assume(row < n && col < n);

        let c = Coord { row, col };
        let cell = cell_id(n, c).expect("cell_id should succeed for valid coords");
        let back = coord(n, cell).expect("coord should succeed for valid cell");

        kani::assert(back.row == row, "row roundtrip failed");
        kani::assert(back.col == col, "col roundtrip failed");
    }

    /// Proves cell index calculation is always in bounds.
    ///
    /// For any N in [2,9] and CellId < N*N, cell_index returns a valid index.
    #[kani::proof]
    fn cell_index_bounds() {
        let n: u8 = kani::any();
        kani::assume(n >= 2 && n <= 9);

        let cell_val: u16 = kani::any();
        let a = (n as u16) * (n as u16);
        kani::assume(cell_val < a);

        let cell = CellId(cell_val);
        let idx = cell_index(n, cell).expect("cell_index should succeed");

        kani::assert(idx < (n as usize) * (n as usize), "index out of bounds");
    }

    /// Proves that cell_id rejects out-of-bounds coordinates.
    #[kani::proof]
    fn cell_id_rejects_oob() {
        let n: u8 = kani::any();
        kani::assume(n >= 2 && n <= 9);

        let row: u8 = kani::any();
        let col: u8 = kani::any();

        // Either row or col is out of bounds
        kani::assume(row >= n || col >= n);

        let c = Coord { row, col };
        let result = cell_id(n, c);

        kani::assert(result.is_err(), "should reject OOB coordinates");
    }

    /// Proves that coord rejects out-of-bounds cell IDs.
    #[kani::proof]
    fn coord_rejects_oob() {
        let n: u8 = kani::any();
        kani::assume(n >= 2 && n <= 9);

        let cell_val: u16 = kani::any();
        let a = (n as u16) * (n as u16);
        kani::assume(cell_val >= a);

        let cell = CellId(cell_val);
        let result = coord(n, cell);

        kani::assert(result.is_err(), "should reject OOB cell ID");
    }

    /// Proves CellId ordering matches row-major grid position.
    #[kani::proof]
    fn cellid_ordering_is_row_major() {
        let n: u8 = kani::any();
        kani::assume(n >= 2 && n <= 9);

        let r1: u8 = kani::any();
        let c1: u8 = kani::any();
        let r2: u8 = kani::any();
        let c2: u8 = kani::any();

        kani::assume(r1 < n && c1 < n && r2 < n && c2 < n);

        let coord1 = Coord { row: r1, col: c1 };
        let coord2 = Coord { row: r2, col: c2 };

        let cell1 = cell_id(n, coord1).unwrap();
        let cell2 = cell_id(n, coord2).unwrap();

        // If (r1, c1) < (r2, c2) in row-major order, then cell1 < cell2
        let row_major_less = r1 < r2 || (r1 == r2 && c1 < c2);
        let cellid_less = cell1 < cell2;

        kani::assert(row_major_less == cellid_less, "ordering mismatch");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eq(n: u8, row: u8, col: u8, target: i32) -> Cage {
        Cage {
            cells: SmallVec::from_slice(&[CellId((row as u16) * (n as u16) + col as u16)]),
            op: Op::Eq,
            target,
        }
    }

    #[test]
    fn validate_rejects_uncovered_cell() {
        let n = 2;
        let p = Puzzle {
            n,
            cages: vec![eq(n, 0, 0, 1), eq(n, 0, 1, 2), eq(n, 1, 0, 2)],
        };
        assert!(matches!(
            p.validate(Ruleset::keen_baseline()),
            Err(CoreError::CellUncovered(_))
        ));
    }

    #[test]
    fn validate_rejects_duplicate_cell() {
        let n = 2;
        let p = Puzzle {
            n,
            cages: vec![
                eq(n, 0, 0, 1),
                eq(n, 0, 0, 1),
                eq(n, 0, 1, 2),
                eq(n, 1, 0, 2),
                eq(n, 1, 1, 1),
            ],
        };
        assert!(matches!(
            p.validate(Ruleset::keen_baseline()),
            Err(CoreError::CellDuplicated(_))
        ));
    }
}
