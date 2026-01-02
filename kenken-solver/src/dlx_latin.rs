//! Latin-square exact-cover utilities using internal DLX implementation.
//!
//! This module intentionally encodes only the Latin constraints (cell, row-digit, col-digit).
//! Cage constraints remain in the main solver (and future SAT encodings).
//!
use crate::dlx::Solver;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LatinChoice {
    row: u8,
    col: u8,
    val: u8,
}

fn constraint_cell(n: usize, row: usize, col: usize) -> usize {
    // 1..=n^2
    1 + row * n + col
}

fn constraint_row_val(n: usize, row: usize, val: usize) -> usize {
    // 1+n^2 ..= 2*n^2
    1 + n * n + row * n + val
}

fn constraint_col_val(n: usize, col: usize, val: usize) -> usize {
    // 1+2*n^2 ..= 3*n^2
    1 + 2 * n * n + col * n + val
}

pub fn solve_latin_one(n: u8, givens: &[u8]) -> Option<Vec<u8>> {
    let n_usize = n as usize;
    let a = n_usize * n_usize;
    assert_eq!(givens.len(), a);

    let mut s = Solver::new(3 * a);
    for row in 0..n_usize {
        for col in 0..n_usize {
            let idx = row * n_usize + col;
            let given = givens[idx];
            if given != 0 {
                let val0 = (given as usize).checked_sub(1)?;
                s.add_option(
                    LatinChoice {
                        row: row as u8,
                        col: col as u8,
                        val: given,
                    },
                    &[
                        constraint_cell(n_usize, row, col),
                        constraint_row_val(n_usize, row, val0),
                        constraint_col_val(n_usize, col, val0),
                    ],
                );
                continue;
            }

            for val in 1..=n_usize {
                let val0 = val - 1;
                s.add_option(
                    LatinChoice {
                        row: row as u8,
                        col: col as u8,
                        val: val as u8,
                    },
                    &[
                        constraint_cell(n_usize, row, col),
                        constraint_row_val(n_usize, row, val0),
                        constraint_col_val(n_usize, col, val0),
                    ],
                );
            }
        }
    }

    let choices = s.next()?;
    let mut grid = vec![0u8; a];
    for ch in choices {
        grid[ch.row as usize * n_usize + ch.col as usize] = ch.val;
    }
    Some(grid)
}

/// Count Latin-square solutions up to `limit`.
pub fn count_latin_solutions_up_to(n: u8, givens: &[u8], limit: u32) -> u32 {
    if limit == 0 {
        return 0;
    }
    let n_usize = n as usize;
    let a = n_usize * n_usize;
    assert_eq!(givens.len(), a);

    let mut s = Solver::new(3 * a);
    for row in 0..n_usize {
        for col in 0..n_usize {
            let idx = row * n_usize + col;
            let given = givens[idx];
            if given != 0 {
                if given as usize > n_usize {
                    return 0;
                }
                let val0 = given as usize - 1;
                s.add_option(
                    LatinChoice {
                        row: row as u8,
                        col: col as u8,
                        val: given,
                    },
                    &[
                        constraint_cell(n_usize, row, col),
                        constraint_row_val(n_usize, row, val0),
                        constraint_col_val(n_usize, col, val0),
                    ],
                );
                continue;
            }
            for val in 1..=n_usize {
                let val0 = val - 1;
                s.add_option(
                    LatinChoice {
                        row: row as u8,
                        col: col as u8,
                        val: val as u8,
                    },
                    &[
                        constraint_cell(n_usize, row, col),
                        constraint_row_val(n_usize, row, val0),
                        constraint_col_val(n_usize, col, val0),
                    ],
                );
            }
        }
    }

    let mut count = 0u32;
    while count < limit {
        if s.next().is_none() {
            break;
        }
        count += 1;
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latin_2x2_counts_two() {
        let givens = [0u8; 4];
        assert_eq!(count_latin_solutions_up_to(2, &givens, 10), 2);
    }

    #[test]
    fn latin_2x2_respects_givens() {
        // Force cell (0,0)=1, which leaves exactly one Latin square for 2x2.
        let mut givens = [0u8; 4];
        givens[0] = 1;
        assert_eq!(count_latin_solutions_up_to(2, &givens, 10), 1);
        let sol = solve_latin_one(2, &givens).unwrap();
        assert_eq!(sol[0], 1);
    }
}
