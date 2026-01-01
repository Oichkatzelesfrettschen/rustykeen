#![allow(clippy::needless_range_loop)]

use kenken_core::rules::{Op, Ruleset};
use kenken_core::{Cage, CellId, Puzzle};
use kenken_solver::{DeductionTier, count_solutions_up_to_with_deductions};
use rand::Rng;
use rand::seq::SliceRandom;
use smallvec::SmallVec;

use crate::GenError;
use crate::seed::rng_from_u64;

#[cfg(feature = "telemetry-tracing")]
use tracing::trace;

#[cfg(not(feature = "telemetry-tracing"))]
macro_rules! trace {
    ($($tt:tt)*) => {};
}

#[derive(Debug, Clone, Copy)]
pub struct GenerateConfig {
    pub n: u8,
    pub seed: u64,
    pub rules: Ruleset,
    pub tier: DeductionTier,
    pub max_attempts: u32,
    pub domino_probability: f64,
}

impl GenerateConfig {
    pub fn keen_baseline(n: u8, seed: u64) -> Self {
        Self {
            n,
            seed,
            rules: Ruleset::keen_baseline(),
            tier: DeductionTier::Hard,
            max_attempts: 10_000,
            domino_probability: 0.55,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedPuzzle {
    pub puzzle: Puzzle,
    pub solution: Vec<u8>,
}

#[cfg(feature = "gen-dlx")]
fn latin_solution_seeded(n: u8, seed: u64) -> Result<Vec<u8>, GenError> {
    use kenken_solver::dlx_latin::solve_latin_one;

    let a = (n as usize) * (n as usize);
    let base =
        solve_latin_one(n, &vec![0u8; a]).ok_or(GenError::AttemptsExhausted { attempts: 1 })?;

    // DLX returns a deterministic “first” solution; add variety via group actions
    // (row/col/symbol permutations) under a deterministic RNG stream.
    let mut rng = rng_from_u64(seed);
    Ok(permute_latin(n, &base, &mut rng))
}

#[cfg(not(feature = "gen-dlx"))]
fn latin_solution_seeded(_n: u8, _seed: u64) -> Result<Vec<u8>, GenError> {
    Err(GenError::DlxRequired)
}

fn permute_latin<R: Rng + ?Sized>(n: u8, grid: &[u8], rng: &mut R) -> Vec<u8> {
    let n_usize = n as usize;
    let a = n_usize * n_usize;
    assert_eq!(grid.len(), a);

    let mut rows: Vec<usize> = (0..n_usize).collect();
    let mut cols: Vec<usize> = (0..n_usize).collect();
    rows.shuffle(rng);
    cols.shuffle(rng);

    let mut syms: Vec<u8> = (1..=n).collect();
    syms.shuffle(rng);
    let mut map = vec![0u8; n_usize + 1];
    for (from0, &to) in syms.iter().enumerate() {
        map[from0 + 1] = to;
    }

    let mut out = vec![0u8; a];
    for r in 0..n_usize {
        for c in 0..n_usize {
            let v = grid[rows[r] * n_usize + cols[c]];
            out[r * n_usize + c] = map[v as usize];
        }
    }
    out
}

fn neighbors(n: usize, idx: usize) -> [Option<usize>; 4] {
    let row = idx / n;
    let col = idx % n;
    [
        (row > 0).then(|| (row - 1) * n + col),
        (row + 1 < n).then(|| (row + 1) * n + col),
        (col > 0).then(|| row * n + (col - 1)),
        (col + 1 < n).then(|| row * n + (col + 1)),
    ]
}

fn random_cage_partition<R: Rng + ?Sized>(
    n: u8,
    rules: Ruleset,
    domino_probability: f64,
    rng: &mut R,
) -> Option<Vec<SmallVec<[CellId; 6]>>> {
    let n_usize = n as usize;
    let a = n_usize * n_usize;
    let max_size = rules.max_cage_size as usize;

    let mut cages: Vec<SmallVec<[CellId; 6]>> = (0..a)
        .map(|i| {
            let mut v = SmallVec::new();
            v.push(CellId(i as u16));
            v
        })
        .collect();
    let mut cage_of: Vec<usize> = (0..a).collect();

    fn merge_cages(
        cages: &mut [SmallVec<[CellId; 6]>],
        cage_of: &mut [usize],
        dst: usize,
        src: usize,
        max_size: usize,
    ) -> bool {
        if dst == src || cages[src].is_empty() {
            return false;
        }
        if cages[dst].len() + cages[src].len() > max_size {
            return false;
        }

        let moved: SmallVec<[CellId; 6]> = cages[src].drain(..).collect();
        for cell in moved {
            let idx = cell.0 as usize;
            cage_of[idx] = dst;
            cages[dst].push(cell);
        }
        true
    }

    // Phase 1: try to create a reasonable number of dominoes (2-cages) early.
    let mut order: Vec<usize> = (0..a).collect();
    order.shuffle(rng);
    for &cell in &order {
        let cid = cage_of[cell];
        if cages[cid].len() != 1 {
            continue;
        }
        if !rng.random_bool(domino_probability) {
            continue;
        }

        let mut neighs: Vec<usize> = neighbors(n_usize, cell).into_iter().flatten().collect();
        neighs.shuffle(rng);
        let Some(&ncell) = neighs.iter().find(|&&j| cages[cage_of[j]].len() == 1) else {
            continue;
        };
        let nid = cage_of[ncell];
        merge_cages(&mut cages, &mut cage_of, cid, nid, max_size);
    }

    // Phase 2: merge remaining singletons into neighbors, respecting size cap.
    let mut singletons: Vec<usize> = (0..a).filter(|&i| cages[cage_of[i]].len() == 1).collect();
    singletons.shuffle(rng);

    for cell in singletons {
        let cid = cage_of[cell];
        if cages[cid].len() != 1 {
            continue;
        }
        let mut options: Vec<usize> = neighbors(n_usize, cell)
            .into_iter()
            .flatten()
            .map(|j| cage_of[j])
            .filter(|&other| {
                other != cid && !cages[other].is_empty() && cages[other].len() < max_size
            })
            .collect();
        options.sort_unstable();
        options.dedup();
        options.shuffle(rng);

        let dst = options.into_iter().next()?;
        merge_cages(&mut cages, &mut cage_of, dst, cid, max_size);
    }

    let out: Vec<SmallVec<[CellId; 6]>> = cages.into_iter().filter(|c| !c.is_empty()).collect();
    Some(out)
}

fn assign_ops_and_targets<R: Rng + ?Sized>(
    n: u8,
    solution: &[u8],
    cages: Vec<SmallVec<[CellId; 6]>>,
    rules: Ruleset,
    rng: &mut R,
) -> Result<Puzzle, GenError> {
    let n_usize = n as usize;
    let a = n_usize * n_usize;
    if solution.len() != a {
        return Err(GenError::AttemptsExhausted { attempts: 1 });
    }

    let mut out_cages: Vec<Cage> = Vec::with_capacity(cages.len());
    for cells in cages {
        let values: SmallVec<[u8; 6]> = cells.iter().map(|c| solution[c.0 as usize]).collect();

        let (op, target) = match cells.len() {
            1 => (Op::Eq, values[0] as i32),
            2 => {
                let a = values[0];
                let b = values[1];
                let mut ops: SmallVec<[Op; 4]> = SmallVec::new();
                ops.push(Op::Add);
                ops.push(Op::Mul);
                if rules.sub_div_two_cell_only {
                    ops.push(Op::Sub);
                    if a % b == 0 || b % a == 0 {
                        ops.push(Op::Div);
                    }
                }
                ops.shuffle(rng);
                let chosen = ops[0];
                let target = match chosen {
                    Op::Add => (a as i32) + (b as i32),
                    Op::Mul => (a as i32) * (b as i32),
                    Op::Sub => (a as i32 - b as i32).abs(),
                    Op::Div => {
                        let (num, den) = if a >= b { (a, b) } else { (b, a) };
                        (num / den) as i32
                    }
                    Op::Eq => unreachable!(),
                };
                (chosen, target)
            }
            _ => {
                let op = if rng.random_bool(0.55) {
                    Op::Add
                } else {
                    Op::Mul
                };
                let target = match op {
                    Op::Add => values.iter().map(|&v| v as i32).sum(),
                    Op::Mul => values.iter().fold(1i32, |acc, &v| acc * (v as i32)),
                    _ => unreachable!(),
                };
                (op, target)
            }
        };

        out_cages.push(Cage { cells, op, target });
    }

    let puzzle = Puzzle {
        n,
        cages: out_cages,
    };
    puzzle.validate(rules)?;
    Ok(puzzle)
}

pub fn generate(config: GenerateConfig) -> Result<GeneratedPuzzle, GenError> {
    let mut rng = rng_from_u64(config.seed);

    trace!(
        n = config.n,
        seed = config.seed,
        max_attempts = config.max_attempts,
        "gen.start"
    );

    for attempt in 0..config.max_attempts {
        // Derive attempt-local streams deterministically.
        let attempt_seed = config.seed ^ ((attempt as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
        trace!(attempt, attempt_seed, "gen.attempt");
        let solution = latin_solution_seeded(config.n, attempt_seed)?;

        let Some(partition) =
            random_cage_partition(config.n, config.rules, config.domino_probability, &mut rng)
        else {
            continue;
        };

        let puzzle =
            assign_ops_and_targets(config.n, &solution, partition, config.rules, &mut rng)?;

        let count = count_solutions_up_to_with_deductions(&puzzle, config.rules, config.tier, 2)?;
        if count == 1 {
            trace!(attempt, "gen.accept");
            return Ok(GeneratedPuzzle { puzzle, solution });
        }
    }

    Err(GenError::AttemptsExhausted {
        attempts: config.max_attempts,
    })
}

#[cfg(all(test, feature = "gen-dlx"))]
mod tests {
    use super::*;

    #[test]
    fn cage_partition_covers_grid_and_is_connected() {
        let rules = Ruleset::keen_baseline();
        let mut rng = rng_from_u64(123);
        let cages = random_cage_partition(4, rules, 1.0, &mut rng).unwrap();

        let puzzle = Puzzle {
            n: 4,
            cages: cages
                .into_iter()
                .map(|cells| Cage {
                    cells,
                    op: Op::Add,
                    target: 1,
                })
                .collect(),
        };
        puzzle.validate(rules).unwrap();
    }

    #[test]
    fn generate_produces_a_unique_puzzle_eventually() {
        let cfg = GenerateConfig {
            max_attempts: 1_000,
            ..GenerateConfig::keen_baseline(4, 42)
        };
        let g = generate(cfg).unwrap();
        assert_eq!(
            count_solutions_up_to_with_deductions(&g.puzzle, cfg.rules, cfg.tier, 2).unwrap(),
            1
        );
    }
}
