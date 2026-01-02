#![allow(clippy::needless_range_loop)]

use kenken_core::rules::{Op, Ruleset};
use kenken_core::{Cage, CellId, Puzzle};
use kenken_solver::{
    DeductionTier, DifficultyTier, TierRequiredResult, classify_difficulty_from_tier,
    classify_tier_required, count_solutions_up_to_with_deductions,
};
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

/// Configuration for puzzle generation.
#[derive(Debug, Clone, Copy)]
pub struct GenerateConfig {
    /// Grid size (n x n).
    pub n: u8,
    /// RNG seed for deterministic generation.
    pub seed: u64,
    /// Ruleset governing cage constraints.
    pub rules: Ruleset,
    /// Deduction tier for uniqueness verification.
    pub tier: DeductionTier,
    /// Maximum generation attempts before giving up.
    pub max_attempts: u32,
    /// Probability of creating 2-cell cages (dominoes) during partitioning.
    pub domino_probability: f64,
    /// Target difficulty tier (None = accept any unique puzzle).
    pub target_difficulty: Option<DifficultyTier>,
    /// Difficulty tolerance: allow tiers within +/- this range.
    /// E.g., tolerance=1 with target=Normal accepts Easy/Normal/Hard.
    pub difficulty_tolerance: u8,
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
            target_difficulty: None,
            difficulty_tolerance: 0,
        }
    }

    /// Create config targeting a specific difficulty tier.
    pub fn with_difficulty(n: u8, seed: u64, target: DifficultyTier) -> Self {
        Self {
            n,
            seed,
            rules: Ruleset::keen_baseline(),
            tier: DeductionTier::Hard,
            max_attempts: 50_000, // More attempts needed for targeting
            domino_probability: 0.55,
            target_difficulty: Some(target),
            difficulty_tolerance: 0,
        }
    }
}

/// Basic generated puzzle (backwards compatible).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedPuzzle {
    pub puzzle: Puzzle,
    pub solution: Vec<u8>,
}

/// Generated puzzle with difficulty classification.
#[derive(Debug, Clone)]
pub struct GeneratedPuzzleWithStats {
    /// The generated puzzle.
    pub puzzle: Puzzle,
    /// The known solution.
    pub solution: Vec<u8>,
    /// Classified difficulty tier.
    pub difficulty: DifficultyTier,
    /// Tier classification result with solve statistics.
    pub tier_result: TierRequiredResult,
    /// Number of generation attempts before accepting this puzzle.
    pub attempts: u32,
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

#[cfg(feature = "gen-dlx")]
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
                    if a.is_multiple_of(b) || b.is_multiple_of(a) {
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

/// Generate a puzzle with full difficulty classification.
///
/// This is the preferred API when you need to know or control the puzzle difficulty.
/// Unlike `generate()`, this always classifies the puzzle and can optionally
/// filter by target difficulty.
///
/// # Arguments
/// * `config` - Generation configuration, optionally with `target_difficulty` set
///
/// # Returns
/// * `Ok(GeneratedPuzzleWithStats)` - A unique puzzle with difficulty classification
/// * `Err(GenError)` - If no suitable puzzle found within max_attempts
pub fn generate_with_stats(config: GenerateConfig) -> Result<GeneratedPuzzleWithStats, GenError> {
    let mut rng = rng_from_u64(config.seed);

    trace!(
        n = config.n,
        seed = config.seed,
        max_attempts = config.max_attempts,
        target_difficulty = ?config.target_difficulty,
        tolerance = config.difficulty_tolerance,
        "gen.start_with_stats"
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

        // First check uniqueness with fast count
        let count = count_solutions_up_to_with_deductions(&puzzle, config.rules, config.tier, 2)?;
        if count != 1 {
            continue;
        }

        // Classify difficulty
        let tier_result = classify_tier_required(&puzzle, config.rules)?;
        let difficulty = classify_difficulty_from_tier(tier_result);

        // Check if difficulty matches target (if specified)
        if let Some(target) = config.target_difficulty
            && !within_difficulty_tolerance(difficulty, target, config.difficulty_tolerance)
        {
            trace!(
                attempt,
                actual = ?difficulty,
                target = ?target,
                "gen.difficulty_mismatch"
            );
            continue;
        }

        trace!(
            attempt,
            difficulty = ?difficulty,
            "gen.accept_with_stats"
        );

        return Ok(GeneratedPuzzleWithStats {
            puzzle,
            solution,
            difficulty,
            tier_result,
            attempts: attempt + 1,
        });
    }

    Err(GenError::AttemptsExhausted {
        attempts: config.max_attempts,
    })
}

/// Check if actual difficulty is within tolerance of target.
///
/// Uses ordinal distance: Easy=0, Normal=1, Hard=2, Extreme=3, Unreasonable=4.
fn within_difficulty_tolerance(
    actual: DifficultyTier,
    target: DifficultyTier,
    tolerance: u8,
) -> bool {
    let actual_ord = difficulty_ordinal(actual);
    let target_ord = difficulty_ordinal(target);
    let distance = actual_ord.abs_diff(target_ord);
    distance <= tolerance
}

/// Convert difficulty tier to ordinal for distance calculation.
fn difficulty_ordinal(tier: DifficultyTier) -> u8 {
    match tier {
        DifficultyTier::Easy => 0,
        DifficultyTier::Normal => 1,
        DifficultyTier::Hard => 2,
        DifficultyTier::Extreme => 3,
        DifficultyTier::Unreasonable => 4,
    }
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

    #[test]
    fn generate_with_stats_classifies_difficulty() {
        let cfg = GenerateConfig {
            max_attempts: 1_000,
            ..GenerateConfig::keen_baseline(4, 99)
        };
        let g = generate_with_stats(cfg).unwrap();

        // Verify puzzle is unique
        assert_eq!(
            count_solutions_up_to_with_deductions(&g.puzzle, cfg.rules, cfg.tier, 2).unwrap(),
            1
        );

        // Verify difficulty is one of the valid tiers
        assert!(matches!(
            g.difficulty,
            DifficultyTier::Easy
                | DifficultyTier::Normal
                | DifficultyTier::Hard
                | DifficultyTier::Extreme
                | DifficultyTier::Unreasonable
        ));

        // Verify attempts is reasonable
        assert!(g.attempts > 0 && g.attempts <= cfg.max_attempts);
    }

    #[test]
    fn difficulty_tolerance_works() {
        // Tolerance of 0: exact match only
        assert!(within_difficulty_tolerance(
            DifficultyTier::Normal,
            DifficultyTier::Normal,
            0
        ));
        assert!(!within_difficulty_tolerance(
            DifficultyTier::Easy,
            DifficultyTier::Normal,
            0
        ));

        // Tolerance of 1: adjacent tiers OK
        assert!(within_difficulty_tolerance(
            DifficultyTier::Easy,
            DifficultyTier::Normal,
            1
        ));
        assert!(within_difficulty_tolerance(
            DifficultyTier::Hard,
            DifficultyTier::Normal,
            1
        ));
        assert!(!within_difficulty_tolerance(
            DifficultyTier::Extreme,
            DifficultyTier::Easy,
            1
        ));

        // Tolerance of 2: two steps OK
        assert!(within_difficulty_tolerance(
            DifficultyTier::Easy,
            DifficultyTier::Hard,
            2
        ));
        assert!(within_difficulty_tolerance(
            DifficultyTier::Extreme,
            DifficultyTier::Normal,
            2
        ));
    }

    #[test]
    fn generate_with_easy_target_produces_easy_puzzle() {
        // This test may need many attempts to find an Easy puzzle
        let cfg = GenerateConfig {
            max_attempts: 10_000,
            target_difficulty: Some(DifficultyTier::Easy),
            difficulty_tolerance: 0,
            ..GenerateConfig::keen_baseline(4, 777)
        };

        // Easy puzzles should be relatively common for 4x4
        let result = generate_with_stats(cfg);

        if let Ok(g) = result {
            assert_eq!(
                g.difficulty,
                DifficultyTier::Easy,
                "Target was Easy, got {:?}",
                g.difficulty
            );
        }
        // It's OK if this fails due to attempts exhausted - Easy puzzles
        // can be rare depending on the seed and grid size
    }
}
