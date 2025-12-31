// docs/solve_dlx.rs
// DLX (Algorithm X) solver scaffolding with matrix builder for Latin + optional cage columns.
#![allow(unused)]

use bitvec::prelude::*;

pub struct Matrix {
    pub ncols: usize,
    pub rows: Vec<Row>,
}

#[derive(Clone)]
pub struct Row {
    pub bits: BitVec,
    pub payload: (u8, u8, u8), // (r,c,n)
}

pub struct Solution {
    pub assignments: Vec<(u8,u8,u8)>,
}

pub fn col_offsets(n: u8, cage_cols: usize) -> (usize, usize, usize, usize) {
    let nn = (n as usize) * (n as usize);
    let c0 = 0;
    let c1 = c0 + nn; // RowNum
    let c2 = c1 + nn; // ColNum
    let c3 = c2 + nn; // Cage
    (c0, c1, c2, c3 + cage_cols)
}

#[inline]
pub fn col_cell(n: u8, r: u8, c: u8, c0: usize) -> usize { c0 + (r as usize) * (n as usize) + (c as usize) }
#[inline]
pub fn col_rownum(n: u8, r: u8, val: u8, c1: usize) -> usize { c1 + (r as usize) * (n as usize) + ((val-1) as usize) }
#[inline]
pub fn col_colnum(n: u8, col: u8, val: u8, c2: usize) -> usize { c2 + (col as usize) * (n as usize) + ((val-1) as usize) }

pub fn build_matrix(n: u8, cage_cols: usize, cage_hit_fn: Option<&dyn Fn(u8,u8,u8, &mut BitVec, usize)>) -> Matrix {
    let (c0, c1, c2, total_cols) = col_offsets(n, cage_cols);
    let mut rows = Vec::with_capacity((n as usize)*(n as usize)*(n as usize));
    for r in 0..n { for c in 0..n { for val in 1..=n {
        let mut bits = bitvec![0; total_cols];
        bits.set(col_cell(n, r, c, c0), true);
        bits.set(col_rownum(n, r, val, c1), true);
        bits.set(col_colnum(n, c, val, c2), true);
        if let Some(hit) = cage_hit_fn { hit(r, c, val, &mut bits, total_cols); }
        rows.push(Row { bits, payload: (r,c,val) });
    }}}
    Matrix { ncols: total_cols, rows }
}

// SolverContext hook
pub struct SolverContext<'a> {
    pub n: u8,
    pub cage_masks: Vec<BitVec>,
    pub cage_full: BitVec,
    pub verify_cage: Box<dyn Fn(usize, &[u8]) -> bool + 'a>,
}

impl<'a> SolverContext<'a> {
    #[inline]
    pub fn on_step(&mut self, grid: &[u8]) -> bool {
        for (k, mask) in self.cage_masks.iter().enumerate() {
            let mut full = true;
            for (idx, bit) in mask.iter().enumerate() {
                if *bit && grid[idx] == 0 { full = false; break; }
            }
            if full && !self.cage_full[k] {
                self.cage_full.set(k, true);
                if !(self.verify_cage)(k, grid) { return false; }
            }
        }
        true
    }
}

// Bitset-based backtracking with lazy callback; counts up to limit solutions
pub fn solve_dlx_unique_with_context(mat: &Matrix, n: u8, ctx: &mut SolverContext, limit: usize) -> (usize, Option<Solution>) {
    let nn = (n as usize) * (n as usize);
    let mut grid = vec![0u8; nn];
    let mut used = bitvec![0; mat.ncols];
    let mut soln: Option<Solution> = None;
    let mut count = 0usize;

    fn place_row(used: &BitVec, row: &Row) -> bool {
        // Check no column conflict
        for (i, b) in row.bits.iter().enumerate() { if *b && used[i] { return false; } }
        true
    }
    fn apply_row(used: &mut BitVec, row: &Row) {
        for (i, b) in row.bits.iter().enumerate() { if *b { used.set(i, true); } }
    }
    fn retract_row(used: &mut BitVec, row: &Row) {
        for (i, b) in row.bits.iter().enumerate() { if *b { used.set(i, false); } }
    }

    // Simple DFS over rows; in practice, choose MRV by column sizes
    fn dfs(idx: usize, mat: &Matrix, n: u8, grid: &mut [u8], used: &mut BitVec, ctx: &mut SolverContext, limit: usize, count: &mut usize, soln: &mut Option<Solution>) {
        if *count >= limit { return; }
        // If all cells filled (nn assignments), record solution
        let nn = (n as usize) * (n as usize);
        let filled = grid.iter().all(|&v| v != 0);
        if filled {
            *count += 1;
            if soln.is_none() { soln.replace(Solution { assignments: grid.chunks_exact(n as usize).enumerate().flat_map(|(r, row)| {
                row.iter().enumerate().map(move |(c, &v)| (r as u8, c as u8, v))
            }).collect() }); }
            return;
        }
        for i in idx..mat.rows.len() {
            let row = &mat.rows[i];
            let (r,c,v) = row.payload;
            let cell_idx = (r as usize)*(n as usize) + (c as usize);
            if grid[cell_idx] != 0 { continue; }
            if !place_row(used, row) { continue; }
            // apply
            grid[cell_idx] = v;
            apply_row(used, row);
            // lazy cage check
            if ctx.on_step(grid) {
                dfs(i+1, mat, n, grid, used, ctx, limit, count, soln);
            }
            // retract
            retract_row(used, row);
            grid[cell_idx] = 0;
            if *count >= limit { return; }
        }
    }

    dfs(0, mat, n, &mut grid, &mut used, ctx, limit, &mut count, &mut soln);
    (count, soln)
}
