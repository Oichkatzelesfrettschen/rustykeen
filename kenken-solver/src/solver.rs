//! Deterministic solver for `kenken-core` puzzles.
//!
//! Design goals:
//! - **Deterministic**: stable ordering, no hash-iteration dependence.
//! - **Library-first**: errors are typed (`SolveError`) and callers control policy.
//! - **Performance-oriented**: optional arenas/instrumentation behind feature flags.
//!
//! Feature flags:
//! - `tracing`: enables `tracing::trace!` in hot paths (no subscriber required by the library).
//! - `perf-likely`: enables branch prediction hints via `likely_stable`.
//! - `alloc-bumpalo`: uses `bumpalo` scratch arenas for propagation temporaries.
//!
use kenken_core::rules::{Op, Ruleset};
use kenken_core::{Cage, CoreError, Puzzle};

#[cfg(feature = "tracing")]
use tracing::{instrument, trace};

#[cfg(not(feature = "tracing"))]
macro_rules! trace {
    ($($tt:tt)*) => {};
}

#[cfg(not(feature = "tracing"))]
macro_rules! instrument {
    ($($tt:tt)*) => {};
}

#[cfg(feature = "perf-likely")]
use likely_stable::likely;

#[cfg(not(feature = "perf-likely"))]
fn likely(v: bool) -> bool {
    v
}

#[cfg(feature = "alloc-bumpalo")]
use bumpalo::Bump;

use crate::error::SolveError;

#[cfg(feature = "simd-dispatch")]
#[allow(dead_code)]
fn popcount_u32(x: u32) -> u32 {
    kenken_simd::popcount_u32(x)
}

#[cfg(not(feature = "simd-dispatch"))]
#[allow(dead_code)]
fn popcount_u32(x: u32) -> u32 {
    x.count_ones()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Solution {
    pub n: u8,
    pub grid: Vec<u8>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SolveStats {
    pub nodes_visited: u64,
    pub assignments: u64,
    pub max_depth: u32,
    /// True if the solver tried multiple values at any cell (branched/guessed).
    /// When false, deductions alone determined all cell values.
    pub backtracked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DifficultyTier {
    Easy,
    Normal,
    Hard,
    Extreme,
    Unreasonable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeductionTier {
    None,
    Easy,
    Normal,
    Hard,
}

/// Solve and return the first solution (if any).
#[instrument(skip(puzzle, rules), fields(n = puzzle.n, cages = puzzle.cages.len()))]
pub fn solve_one(puzzle: &Puzzle, rules: Ruleset) -> Result<Option<Solution>, SolveError> {
    let mut first = None;
    let count = search(puzzle, rules, 1, &mut first)?;
    Ok(if count == 0 { None } else { first })
}

/// Solve and also return solver statistics for the search (nodes, assignments, depth).
pub fn solve_one_with_stats(
    puzzle: &Puzzle,
    rules: Ruleset,
) -> Result<(Option<Solution>, SolveStats), SolveError> {
    let mut first = None;
    let mut stats = SolveStats::default();
    let count = search_with_stats(puzzle, rules, 1, &mut first, &mut stats)?;
    Ok((if count == 0 { None } else { first }, stats))
}

/// Solve with a selectable deduction tier (propagation strength).
#[instrument(skip(puzzle, rules), fields(n = puzzle.n, cages = puzzle.cages.len(), tier = ?tier))]
pub fn solve_one_with_deductions(
    puzzle: &Puzzle,
    rules: Ruleset,
    tier: DeductionTier,
) -> Result<Option<Solution>, SolveError> {
    let mut first = None;
    let mut stats = SolveStats::default();
    let count = search_with_stats_deducing(puzzle, rules, tier, 1, &mut first, &mut stats)?;
    Ok(if count == 0 { None } else { first })
}

/// Count solutions up to `limit` (use `2` to check uniqueness).
#[instrument(skip(puzzle, rules), fields(n = puzzle.n, limit))]
pub fn count_solutions_up_to(
    puzzle: &Puzzle,
    rules: Ruleset,
    limit: u32,
) -> Result<u32, SolveError> {
    if limit == 0 {
        return Ok(0);
    }
    search(puzzle, rules, limit, &mut None)
}

/// Count solutions up to `limit` using a selectable deduction tier.
///
/// This is the primary “uniqueness check” building block for generator pipelines.
pub fn count_solutions_up_to_with_deductions(
    puzzle: &Puzzle,
    rules: Ruleset,
    tier: DeductionTier,
    limit: u32,
) -> Result<u32, SolveError> {
    if limit == 0 {
        return Ok(0);
    }
    let mut stats = SolveStats::default();
    search_with_stats_deducing(puzzle, rules, tier, limit, &mut None, &mut stats)
}

fn search(
    puzzle: &Puzzle,
    rules: Ruleset,
    limit: u32,
    first: &mut Option<Solution>,
) -> Result<u32, SolveError> {
    let mut stats = SolveStats::default();
    search_with_stats(puzzle, rules, limit, first, &mut stats)
}

fn search_with_stats(
    puzzle: &Puzzle,
    rules: Ruleset,
    limit: u32,
    first: &mut Option<Solution>,
    stats: &mut SolveStats,
) -> Result<u32, SolveError> {
    puzzle.validate(rules)?;

    let n = puzzle.n as usize;
    let a = n * n;

    let mut cage_of_cell = vec![usize::MAX; a];
    for (cage_idx, cage) in puzzle.cages.iter().enumerate() {
        for cell in &cage.cells {
            cage_of_cell[cell.0 as usize] = cage_idx;
        }
    }

    let mut state = State {
        n: puzzle.n,
        grid: vec![0; a],
        row_mask: vec![0u64; n],
        col_mask: vec![0u64; n],
        cage_of_cell,
        tuple_cache: HashMap::new(),
        mrv_cache: MrvCache::new(puzzle.n),
    };

    let mut count = 0u32;
    backtrack(
        puzzle, rules, limit, first, &mut state, &mut count, 0, stats,
    )?;
    Ok(count)
}

fn search_with_stats_deducing(
    puzzle: &Puzzle,
    rules: Ruleset,
    tier: DeductionTier,
    limit: u32,
    first: &mut Option<Solution>,
    stats: &mut SolveStats,
) -> Result<u32, SolveError> {
    puzzle.validate(rules)?;

    let n = puzzle.n as usize;
    let a = n * n;

    let mut cage_of_cell = vec![usize::MAX; a];
    for (cage_idx, cage) in puzzle.cages.iter().enumerate() {
        for cell in &cage.cells {
            cage_of_cell[cell.0 as usize] = cage_idx;
        }
    }

    let mut state = State {
        n: puzzle.n,
        grid: vec![0; a],
        row_mask: vec![0u64; n],
        col_mask: vec![0u64; n],
        cage_of_cell,
        tuple_cache: HashMap::new(),
        mrv_cache: MrvCache::new(puzzle.n),
    };

    let mut forced = Vec::new();
    if tier != DeductionTier::None && !propagate(puzzle, rules, tier, &mut state, &mut forced)? {
        return Ok(0);
    }

    // Tier 2.2: Cache needs recomputation after propagation modifies domains
    state.mrv_cache.valid = false;

    let mut count = 0u32;
    backtrack_deducing(
        puzzle, rules, tier, limit, first, &mut state, &mut count, 0, stats,
    )?;
    Ok(count)
}

use std::collections::HashMap;

/// Cache key for memoizing enumerate_cage_tuples results.
/// Key: (op_hash, target, cells_count, cells_hash, domain_state_hash)
#[allow(dead_code)]
type CacheTupleKey = (u8, u8, i32, usize, u64, u64);

/// Cached result from enumerate_cage_tuples.
#[derive(Clone)]
#[allow(dead_code)]
struct CachedTupleResult {
    per_pos: Vec<u64>,
    any_mask: u64,
}

struct State {
    n: u8,
    grid: Vec<u8>,
    row_mask: Vec<u64>,  // Extended to u64 to support n <= 63
    col_mask: Vec<u64>,  // Extended to u64 to support n <= 63
    cage_of_cell: Vec<usize>,
    /// Memoization cache for enumerate_cage_tuples results.
    /// Maps (cage_signature, domain_hash) -> (per_pos, any_mask).
    /// Only used for n >= 4; cache skipped for tiny puzzles (n <= 3).
    #[allow(dead_code)]
    tuple_cache: HashMap<CacheTupleKey, CachedTupleResult>,
    /// Incremental MRV cache for Tier 2.2 optimization.
    /// Tracks minimum-remaining-value cell and invalidates selectively.
    #[allow(dead_code)]
    mrv_cache: MrvCache,
}

/// Check if all cells in a cage are fully assigned (domain size == 1).
/// This enables Tier 1.2 optimization: skip enumeration for fully-assigned cages.
#[inline]
fn all_cells_fully_assigned(cells: &[usize], domains: &[u64]) -> bool {
    for &idx in cells {
        // Cell is fully assigned if exactly 1 bit is set (domain.popcount() == 1)
        let popcount = domains[idx].count_ones();
        if popcount != 1 {
            return false;
        }
    }
    true
}

/// State for incremental MRV computation (Tier 2.2 optimization).
/// Maintains the minimum-remaining-value cell and invalidates selectively.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct MrvCache {
    min_cell: usize,
    min_count: u32,
    valid: bool,
    dirty_cells: Vec<bool>,
}

impl MrvCache {
    fn new(n: u8) -> Self {
        let size = (n as usize) * (n as usize);
        Self {
            min_cell: 0,
            min_count: n as u32 + 1,
            valid: false,
            dirty_cells: vec![false; size],
        }
    }

    #[allow(dead_code)]
    fn reset_dirty(&mut self) {
        for dirty in &mut self.dirty_cells {
            *dirty = false;
        }
        self.valid = false;
    }

    #[allow(dead_code)]
    fn mark_dirty(&mut self, idx: usize) {
        self.dirty_cells[idx] = true;
        self.valid = false;
    }

    #[allow(dead_code)]
    fn mark_clean(&mut self, idx: usize) {
        self.dirty_cells[idx] = false;
    }

    #[allow(dead_code)]
    fn has_dirty_cells(&self) -> bool {
        self.dirty_cells.iter().any(|&d| d)
    }
}

/// Compute any_mask (union of valid values) from fully-assigned cage cells.
/// Used by Tier 1.2 to avoid enumeration when all cells have exactly one value.
#[inline]
fn compute_any_mask_from_assigned(cells: &[usize], domains: &[u64]) -> u64 {
    let mut any_mask = 0u64;
    for &idx in cells {
        any_mask |= domains[idx];
    }
    any_mask
}

/// Compute a cache key for a cage's tuple enumeration.
/// Uses a hash of the cage's cells and the domain state for those cells.
/// CRITICAL: Includes deduction tier to prevent cache mixing across different propagation contexts.
#[inline]
#[allow(dead_code)]
fn compute_cache_key(cage: &Cage, cells: &[usize], domains: &[u64], tier: DeductionTier) -> CacheTupleKey {
    // Simple hash of cell indices
    let mut cells_hash = 0u64;
    for &cell in cells.iter() {
        cells_hash = cells_hash.wrapping_mul(31).wrapping_add(cell as u64);
    }

    // Hash of domain state for cage cells
    let mut domain_hash = 0u64;
    for &cell in cells {
        domain_hash = domain_hash.wrapping_mul(31).wrapping_add(domains[cell]);
    }

    // Use Op::Add as 0, Op::Sub as 1, Op::Div as 2, Op::Mul as 3, Op::Eq as 4
    let op_byte = match cage.op {
        Op::Add => 0u8,
        Op::Sub => 1u8,
        Op::Div => 2u8,
        Op::Mul => 3u8,
        Op::Eq => 4u8,
    };

    // Encode deduction tier: None=0, Easy=1, Normal=2, Hard=3
    let tier_byte = match tier {
        DeductionTier::None => 0u8,
        DeductionTier::Easy => 1u8,
        DeductionTier::Normal => 2u8,
        DeductionTier::Hard => 3u8,
    };

    (op_byte, tier_byte, cage.target, cells.len(), cells_hash, domain_hash)
}

#[allow(clippy::too_many_arguments)]
#[instrument(skip(puzzle, rules, first, state, count, stats), fields(depth, n = state.n), level = "debug")]
fn backtrack(
    puzzle: &Puzzle,
    rules: Ruleset,
    limit: u32,
    first: &mut Option<Solution>,
    state: &mut State,
    count: &mut u32,
    depth: u32,
    stats: &mut SolveStats,
) -> Result<(), SolveError> {
    if *count >= limit {
        return Ok(());
    }

    stats.nodes_visited += 1;
    stats.max_depth = stats.max_depth.max(depth);

    let Some((cell_idx, domain)) = choose_mrv_cell(puzzle, state)? else {
        // Solved
        *count += 1;
        if first.is_none() {
            *first = Some(Solution {
                n: state.n,
                grid: state.grid.clone(),
            });
        }
        return Ok(());
    };

    let row = cell_idx / (state.n as usize);
    let col = cell_idx % (state.n as usize);

    let mut mask = domain;
    let mut tried = 0u32;
    while mask != 0 {
        let d = mask.trailing_zeros() as u8;
        mask &= mask - 1;
        if d == 0 {
            continue;
        }

        tried += 1;
        if tried > 1 {
            stats.backtracked = true;
        }

        trace!(cell = cell_idx, digit = d, "try");
        place(state, row, col, d);
        stats.assignments += 1;
        if likely(cages_still_feasible(puzzle, rules, state, cell_idx)?) {
            backtrack(puzzle, rules, limit, first, state, count, depth + 1, stats)?;
        }
        unplace(state, row, col, d);

        if *count >= limit {
            return Ok(());
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
#[instrument(skip(puzzle, rules, first, state, count, stats), fields(depth, tier = ?tier), level = "debug")]
fn backtrack_deducing(
    puzzle: &Puzzle,
    rules: Ruleset,
    tier: DeductionTier,
    limit: u32,
    first: &mut Option<Solution>,
    state: &mut State,
    count: &mut u32,
    depth: u32,
    stats: &mut SolveStats,
) -> Result<(), SolveError> {
    if *count >= limit {
        return Ok(());
    }

    stats.nodes_visited += 1;
    stats.max_depth = stats.max_depth.max(depth);

    let Some((cell_idx, domain)) = choose_mrv_cell(puzzle, state)? else {
        *count += 1;
        if first.is_none() {
            *first = Some(Solution {
                n: state.n,
                grid: state.grid.clone(),
            });
        }
        return Ok(());
    };

    let row = cell_idx / (state.n as usize);
    let col = cell_idx % (state.n as usize);

    let mut mask = domain;
    let mut tried = 0u32;
    while mask != 0 {
        let d = mask.trailing_zeros() as u8;
        mask &= mask - 1;
        if d == 0 {
            continue;
        }

        tried += 1;
        if tried > 1 {
            stats.backtracked = true;
        }

        place(state, row, col, d);
        stats.assignments += 1;

        let mut forced = Vec::new();
        let feasible = cages_still_feasible(puzzle, rules, state, cell_idx)?
            && if tier == DeductionTier::None {
                true
            } else {
                propagate(puzzle, rules, tier, state, &mut forced)?
            };

        // Tier 2.2: Invalidate MRV cache after propagation modifies domains
        if feasible && tier != DeductionTier::None {
            state.mrv_cache.valid = false;
        }

        if likely(feasible) {
            backtrack_deducing(
                puzzle,
                rules,
                tier,
                limit,
                first,
                state,
                count,
                depth + 1,
                stats,
            )?;
        }

        for (idx, val) in forced.into_iter().rev() {
            let r = idx / (state.n as usize);
            let c = idx % (state.n as usize);
            unplace(state, r, c, val);
        }

        unplace(state, row, col, d);

        if *count >= limit {
            return Ok(());
        }
    }

    Ok(())
}

/// Result of tier-required classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TierRequiredResult {
    /// Minimum deduction tier needed to solve without guessing.
    /// `None` means guessing (backtracking) was required.
    pub tier_required: Option<DeductionTier>,
    /// Search statistics from the successful solve attempt.
    pub stats: SolveStats,
}

/// Determine the minimum deduction tier required to solve the puzzle.
///
/// Tries solving at progressively stronger deduction tiers until success
/// without backtracking. This is the primary difficulty signal matching
/// upstream sgt-puzzles behavior.
///
/// Returns the minimum tier where the puzzle was solvable using only
/// deductions (no guessing). If even Hard tier requires guessing,
/// `tier_required` is `None`.
#[instrument(skip(puzzle, rules), fields(n = puzzle.n))]
pub fn classify_tier_required(
    puzzle: &Puzzle,
    rules: Ruleset,
) -> Result<TierRequiredResult, SolveError> {
    // Try tiers in order: Easy -> Normal -> Hard
    for tier in [
        DeductionTier::Easy,
        DeductionTier::Normal,
        DeductionTier::Hard,
    ] {
        let mut first = None;
        let mut stats = SolveStats::default();
        let count = search_with_stats_deducing(puzzle, rules, tier, 1, &mut first, &mut stats)?;

        if count > 0 && !stats.backtracked {
            return Ok(TierRequiredResult {
                tier_required: Some(tier),
                stats,
            });
        }
    }

    // Even Hard tier required backtracking; solve with full search
    let mut first = None;
    let mut stats = SolveStats::default();
    let _ = search_with_stats_deducing(
        puzzle,
        rules,
        DeductionTier::Hard,
        1,
        &mut first,
        &mut stats,
    )?;

    Ok(TierRequiredResult {
        tier_required: None,
        stats,
    })
}

/// Classify difficulty from a tier-required result.
///
/// This is the **primary difficulty classification** matching upstream behavior.
/// Difficulty is determined by which deduction tier was required:
/// - Easy tier sufficient -> Easy
/// - Normal tier sufficient -> Normal
/// - Hard tier sufficient -> Hard
/// - Guessing required -> Extreme or Unreasonable based on search cost
pub fn classify_difficulty_from_tier(result: TierRequiredResult) -> DifficultyTier {
    match result.tier_required {
        Some(DeductionTier::Easy) => DifficultyTier::Easy,
        Some(DeductionTier::Normal) => DifficultyTier::Normal,
        Some(DeductionTier::Hard) => DifficultyTier::Hard,
        Some(DeductionTier::None) => {
            // Shouldn't happen (None tier means no deductions), treat as backtracking
            classify_difficulty_from_stats(result.stats)
        }
        None => {
            // Required backtracking; use search cost for Extreme vs Unreasonable
            if result.stats.nodes_visited <= 50_000 {
                DifficultyTier::Extreme
            } else {
                DifficultyTier::Unreasonable
            }
        }
    }
}

/// Legacy difficulty classification from solve statistics alone.
///
/// **Deprecated**: Use `classify_tier_required` + `classify_difficulty_from_tier` instead.
/// This is retained for backwards compatibility and for cases where only stats are available.
pub fn classify_difficulty(stats: SolveStats) -> DifficultyTier {
    classify_difficulty_from_stats(stats)
}

/// Classify difficulty from solve statistics (search cost).
///
/// This is a fallback for puzzles that require backtracking.
/// The thresholds are approximate and may need calibration.
fn classify_difficulty_from_stats(stats: SolveStats) -> DifficultyTier {
    match stats.assignments {
        0..=200 => DifficultyTier::Easy,
        201..=2_000 => DifficultyTier::Normal,
        2_001..=20_000 => DifficultyTier::Hard,
        20_001..=200_000 => DifficultyTier::Extreme,
        _ => DifficultyTier::Unreasonable,
    }
}

#[instrument(skip(puzzle, state), fields(n = state.n, cached = false), level = "debug")]
fn choose_mrv_cell(puzzle: &Puzzle, state: &mut State) -> Result<Option<(usize, u64)>, SolveError> {
    let n = state.n as usize;
    let a = n * n;

    // Phase 2 optimization: use cache if still valid and no dirty cells
    // When cache is valid, we can return the cached min_cell without rescanning
    if state.mrv_cache.valid && !state.mrv_cache.has_dirty_cells() {
        // Cache hit: return cached result
        let min_idx = state.mrv_cache.min_cell;
        if state.grid[min_idx] == 0 {
            // Cell still unfilled; use cached domain computation
            let row = min_idx / n;
            let col = min_idx % n;
            if let Ok(dom) = domain_for_cell(puzzle, state, min_idx, row, col) {
                if popcount_u64(dom) > 0 {
                    return Ok(Some((min_idx, dom)));
                }
            }
        }
        // Cache miss (cell filled or domain empty): invalidate and rescan
    }

    // Cache miss or invalid: full rescan
    let mut best: Option<(usize, u64, u32)> = None; // (idx, domain, popcnt)

    for idx in 0..a {
        if state.grid[idx] != 0 {
            continue;
        }
        let row = idx / n;
        let col = idx % n;
        let dom = domain_for_cell(puzzle, state, idx, row, col)?;
        let pop = popcount_u64(dom);
        if pop == 0 {
            return Ok(None);
        }
        match best {
            None => best = Some((idx, dom, pop)),
            Some((_, _, best_pop)) if pop < best_pop => best = Some((idx, dom, pop)),
            _ => {}
        }
        if best.is_some_and(|(_, _, p)| p == 1) {
            break;
        }
    }

    // Update cache with new result before returning (Tier 2.2 optimization)
    if let Some((idx, _dom, pop)) = best {
        state.mrv_cache.min_cell = idx;
        state.mrv_cache.min_count = pop;
        state.mrv_cache.valid = true;
        state.mrv_cache.reset_dirty();
    }

    Ok(best.map(|(idx, dom, _)| (idx, dom)))
}

fn popcount_u64(x: u64) -> u32 {
    x.count_ones()
}

fn domain_for_cell(
    puzzle: &Puzzle,
    state: &State,
    idx: usize,
    row: usize,
    col: usize,
) -> Result<u64, CoreError> {
    let n = state.n;
    let mut dom = full_domain(n) & !state.row_mask[row] & !state.col_mask[col];

    let cage = &puzzle.cages[state.cage_of_cell[idx]];
    if cage.cells.len() == 1 && cage.op == Op::Eq {
        if cage.target <= 0 || cage.target > n as i32 {
            return Err(CoreError::EqTargetOutOfRange);
        }
        dom &= 1u64 << (cage.target as u32);
    }

    Ok(dom)
}

fn cages_still_feasible(
    puzzle: &Puzzle,
    rules: Ruleset,
    state: &State,
    changed_cell: usize,
) -> Result<bool, SolveError> {
    let cage_idx = state.cage_of_cell[changed_cell];
    let cage = &puzzle.cages[cage_idx];
    if !cage_feasible(puzzle, rules, state, cage)? {
        return Ok(false);
    }
    Ok(true)
}

#[instrument(skip(puzzle, rules, state, forced), fields(n = state.n, tier = ?tier, iterations = 0), level = "debug")]
fn propagate(
    puzzle: &Puzzle,
    rules: Ruleset,
    tier: DeductionTier,
    state: &mut State,
    forced: &mut Vec<(usize, u8)>,
) -> Result<bool, SolveError> {
    let n = state.n as usize;
    let a = n * n;

    #[cfg(feature = "alloc-bumpalo")]
    let mut bump = Bump::new();

    let mut domains = vec![0u64; a];

    loop {
        #[cfg(feature = "alloc-bumpalo")]
        bump.reset();

        domains.fill(0u64);
        for (idx, dom_slot) in domains.iter_mut().enumerate() {
            if state.grid[idx] != 0 {
                *dom_slot = 1u64 << (state.grid[idx] as u32);
                continue;
            }
            let r = idx / n;
            let c = idx % n;
            *dom_slot = full_domain(state.n) & !state.row_mask[r] & !state.col_mask[c];
        }

        for cage in &puzzle.cages {
            #[cfg(feature = "alloc-bumpalo")]
            apply_cage_deduction_with_bump(&bump, puzzle, rules, state, cage, tier, &mut domains)?;

            #[cfg(not(feature = "alloc-bumpalo"))]
            apply_cage_deduction(puzzle, rules, state, cage, tier, &mut domains)?;
        }

        for (idx, &dom) in domains.iter().enumerate() {
            if state.grid[idx] == 0 && dom == 0 {
                return Ok(false);
            }
        }

        let mut any_forced = false;
        for (idx, &dom) in domains.iter().enumerate() {
            if state.grid[idx] != 0 {
                continue;
            }
            if popcount_u64(dom) == 1 {
                let val = dom.trailing_zeros() as u8;
                let r = idx / n;
                let c = idx % n;
                place(state, r, c, val);
                forced.push((idx, val));
                any_forced = true;
            }
        }

        if !any_forced {
            return Ok(true);
        }
    }
}

#[cfg(not(feature = "alloc-bumpalo"))]
#[instrument(skip(_puzzle, rules, state, cage, domains), fields(op = ?cage.op, cells = cage.cells.len()), level = "debug")]
fn apply_cage_deduction(
    _puzzle: &Puzzle,
    rules: Ruleset,
    state: &mut State,
    cage: &Cage,
    tier: DeductionTier,
    domains: &mut [u64],
) -> Result<(), SolveError> {
    let n = state.n as usize;
    let a = n * n;
    let cells: Vec<usize> = cage.cells.iter().map(|c| c.0 as usize).collect();

    match cage.op {
        Op::Eq => {
            let idx = cells[0];
            domains[idx] &= 1u64 << (cage.target as u32);
            return Ok(());
        }
        Op::Sub | Op::Div if rules.sub_div_two_cell_only && cage.cells.len() != 2 => {
            return Err(CoreError::SubDivMustBeTwoCell.into());
        }
        Op::Sub | Op::Div if cage.cells.len() == 2 => {
            let a_idx = cells[0];
            let b_idx = cells[1];
            let a_dom = domains[a_idx];
            let b_dom = domains[b_idx];

            // TIER 1.2: If both cells are fully assigned, verify constraint directly
            if tier != DeductionTier::Hard
                && domains[a_idx].count_ones() == 1
                && domains[b_idx].count_ones() == 1 {
                // Both cells have exactly one value; check constraint directly
                let av = (a_dom.trailing_zeros() + 1) as u8;
                let bv = (b_dom.trailing_zeros() + 1) as u8;
                let ok = match cage.op {
                    Op::Sub => (av as i32 - bv as i32).abs() == cage.target,
                    Op::Div => {
                        let (num, den) = if av >= bv { (av, bv) } else { (bv, av) };
                        den != 0 && (num as i32) == (den as i32).saturating_mul(cage.target)
                    }
                    _ => false,
                };
                if ok {
                    // Constraint satisfied; domains unchanged
                } else {
                    // Constraint violated; domains empty
                    domains[a_idx] = 0u64;
                    domains[b_idx] = 0u64;
                }
            } else {
                // Standard enumeration (needed for Hard tier or when cells not fully assigned)
                let mut a_ok = 0u64;
                let mut b_ok = 0u64;
                let mut found = false;
                let coords = [(a_idx / n, a_idx % n), (b_idx / n, b_idx % n)];
                let mut must_row: Vec<Option<u64>> = vec![None; n];
                let mut must_col: Vec<Option<u64>> = vec![None; n];

                for av in domain_iter(a_dom) {
                    for bv in domain_iter(b_dom) {
                        let ok = match cage.op {
                            Op::Sub => (av as i32 - bv as i32).abs() == cage.target,
                            Op::Div => {
                                let (num, den) = if av >= bv { (av, bv) } else { (bv, av) };
                                den != 0 && (num as i32) == (den as i32).saturating_mul(cage.target)
                            }
                            _ => false,
                        };
                        if ok {
                            found = true;
                            a_ok |= 1u64 << (av as u32);
                            b_ok |= 1u64 << (bv as u32);

                            if tier == DeductionTier::Hard {
                                let pair = [av, bv];
                                let mut row_bits = vec![0u64; n];
                                let mut col_bits = vec![0u64; n];
                                for (i, &(r, c)) in coords.iter().enumerate() {
                                    row_bits[r] |= 1u64 << (pair[i] as u32);
                                    col_bits[c] |= 1u64 << (pair[i] as u32);
                                }
                                for r in 0..n {
                                    if row_bits[r] != 0 {
                                        must_row[r] = Some(match must_row[r] {
                                            None => row_bits[r],
                                            Some(m) => m & row_bits[r],
                                        });
                                    }
                                }
                                for c in 0..n {
                                    if col_bits[c] != 0 {
                                        must_col[c] = Some(match must_col[c] {
                                            None => col_bits[c],
                                            Some(m) => m & col_bits[c],
                                        });
                                    }
                                }
                            }
                        }
                    }
                }

                domains[a_idx] &= a_ok;
                domains[b_idx] &= b_ok;

                if tier == DeductionTier::Hard && found {
                    let mut in_cage = vec![false; a];
                    in_cage[a_idx] = true;
                    in_cage[b_idx] = true;
                    for (r, maybe_must) in must_row.into_iter().enumerate() {
                        let Some(must) = maybe_must else { continue };
                        for c in 0..n {
                            let idx = r * n + c;
                            if !in_cage[idx] {
                                domains[idx] &= !must;
                            }
                        }
                    }
                    for (c, maybe_must) in must_col.into_iter().enumerate() {
                        let Some(must) = maybe_must else { continue };
                        for r in 0..n {
                            let idx = r * n + c;
                            if !in_cage[idx] {
                                domains[idx] &= !must;
                            }
                        }
                    }
                }
            }
            return Ok(());
        }
        Op::Add | Op::Mul => {
            let coords: Vec<(usize, usize)> = cells.iter().map(|&idx| (idx / n, idx % n)).collect();
            let (per_pos, any_mask, must_row, must_col, found) = if tier == DeductionTier::Hard {
                enumerate_cage_tuples_with_must(n, cage, &cells, &coords, domains)
            } else {
                // TIER 1.2: Skip enumeration if all cage cells are fully assigned.
                // Only for Easy/Normal tiers (Hard tier needs full enumeration for constraint learning).
                if tier != DeductionTier::Hard && all_cells_fully_assigned(&cells, domains) {
                    // All cells have exactly one value; skip enumeration and compute any_mask directly
                    let any_mask = compute_any_mask_from_assigned(&cells, domains);
                    let per_pos = vec![any_mask; cells.len()];
                    (per_pos, any_mask, vec![0u64; n], vec![0u64; n], any_mask != 0)
                } else if n >= 6 {
                    // TIER 1.1: Cache enumeration results (only for n >= 6)
                    let cache_key = compute_cache_key(cage, &cells, domains, tier);
                    if let Some(cached) = state.tuple_cache.get(&cache_key) {
                        // Cache hit: use cached result
                        (
                            cached.per_pos.clone(),
                            cached.any_mask,
                            vec![0u64; n],
                            vec![0u64; n],
                            cached.any_mask != 0,
                        )
                    } else {
                        // Cache miss: compute and store
                        let mut per_pos = vec![0u64; cells.len()];
                        let mut any_mask = 0u64;
                        enumerate_cage_tuples(
                            cage,
                            &cells,
                            &coords,
                            domains,
                            0,
                            &mut Vec::new(),
                            &mut per_pos,
                            &mut any_mask,
                        );

                        // Store in cache before returning
                        state.tuple_cache.insert(
                            cache_key,
                            CachedTupleResult {
                                per_pos: per_pos.clone(),
                                any_mask,
                            },
                        );

                        (
                            per_pos,
                            any_mask,
                            vec![0u64; n],
                            vec![0u64; n],
                            any_mask != 0,
                        )
                    }
                } else {
                    // For small puzzles (n <= 5), skip cache and just compute
                    let mut per_pos = vec![0u64; cells.len()];
                    let mut any_mask = 0u64;
                    enumerate_cage_tuples(
                        cage,
                        &cells,
                        &coords,
                        domains,
                        0,
                        &mut Vec::new(),
                        &mut per_pos,
                        &mut any_mask,
                    );

                    (
                        per_pos,
                        any_mask,
                        vec![0u64; n],
                        vec![0u64; n],
                        any_mask != 0,
                    )
                }
            };

            if tier == DeductionTier::Easy {
                for &idx in &cells {
                    domains[idx] &= any_mask;
                }
            } else {
                for (pos, &idx) in cells.iter().enumerate() {
                    domains[idx] &= per_pos[pos];
                }
            }

            if tier == DeductionTier::Hard && found {
                let mut in_cage = vec![false; a];
                for &idx in &cells {
                    in_cage[idx] = true;
                }
                for (r, must) in must_row.into_iter().enumerate() {
                    if must == 0 {
                        continue;
                    }
                    for c in 0..n {
                        let idx = r * n + c;
                        if !in_cage[idx] {
                            domains[idx] &= !must;
                        }
                    }
                }
                for (c, must) in must_col.into_iter().enumerate() {
                    if must == 0 {
                        continue;
                    }
                    for r in 0..n {
                        let idx = r * n + c;
                        if !in_cage[idx] {
                            domains[idx] &= !must;
                        }
                    }
                }
            }
            return Ok(());
        }
        _ => {}
    }

    Ok(())
}

#[cfg(feature = "alloc-bumpalo")]
#[instrument(skip(bump, _puzzle, rules, state, cage, domains), fields(op = ?cage.op, cells = cage.cells.len()), level = "debug")]
fn apply_cage_deduction_with_bump(
    bump: &Bump,
    _puzzle: &Puzzle,
    rules: Ruleset,
    state: &mut State,
    cage: &Cage,
    tier: DeductionTier,
    domains: &mut [u64],
) -> Result<(), SolveError> {
    // Use bump-allocated temporary vectors to reduce per-iteration heap churn in propagation.
    let n = state.n as usize;
    let a = n * n;
    let mut cells = bumpalo::collections::Vec::with_capacity_in(cage.cells.len(), bump);
    for c in &cage.cells {
        cells.push(c.0 as usize);
    }

    match cage.op {
        Op::Eq => {
            let idx = cells[0];
            domains[idx] &= 1u64 << (cage.target as u32);
            return Ok(());
        }
        Op::Sub | Op::Div if rules.sub_div_two_cell_only && cage.cells.len() != 2 => {
            return Err(CoreError::SubDivMustBeTwoCell.into());
        }
        Op::Sub | Op::Div if cage.cells.len() == 2 => {
            // Delegate to the existing implementation, but avoid allocating the `cells` Vec on the heap.
            let a_idx = cells[0];
            let b_idx = cells[1];
            let a_dom = domains[a_idx];
            let b_dom = domains[b_idx];
            let mut a_ok = 0u64;
            let mut b_ok = 0u64;
            let mut found = false;
            let mut must_row: bumpalo::collections::Vec<Option<u64>> =
                bumpalo::collections::Vec::with_capacity_in(n, bump);
            let mut must_col: bumpalo::collections::Vec<Option<u64>> =
                bumpalo::collections::Vec::with_capacity_in(n, bump);
            must_row.resize(n, None);
            must_col.resize(n, None);
            let coords = [(a_idx / n, a_idx % n), (b_idx / n, b_idx % n)];
            for av in domain_iter(a_dom) {
                for bv in domain_iter(b_dom) {
                    let ok = match cage.op {
                        Op::Sub => (av as i32 - bv as i32).abs() == cage.target,
                        Op::Div => {
                            let (num, den) = if av >= bv { (av, bv) } else { (bv, av) };
                            den != 0 && (num as i32) == (den as i32).saturating_mul(cage.target)
                        }
                        _ => false,
                    };
                    if ok {
                        found = true;
                        a_ok |= 1u64 << (av as u32);
                        b_ok |= 1u64 << (bv as u32);

                        if tier == DeductionTier::Hard {
                            let (ra, ca) = coords[0];
                            let (rb, cb) = coords[1];
                            let a_bit = 1u64 << (av as u32);
                            let b_bit = 1u64 << (bv as u32);

                            if ra == rb {
                                let bits = a_bit | b_bit;
                                must_row[ra] = Some(match must_row[ra] {
                                    None => bits,
                                    Some(m) => m & bits,
                                });
                            } else {
                                must_row[ra] = Some(match must_row[ra] {
                                    None => a_bit,
                                    Some(m) => m & a_bit,
                                });
                                must_row[rb] = Some(match must_row[rb] {
                                    None => b_bit,
                                    Some(m) => m & b_bit,
                                });
                            }

                            if ca == cb {
                                let bits = a_bit | b_bit;
                                must_col[ca] = Some(match must_col[ca] {
                                    None => bits,
                                    Some(m) => m & bits,
                                });
                            } else {
                                must_col[ca] = Some(match must_col[ca] {
                                    None => a_bit,
                                    Some(m) => m & a_bit,
                                });
                                must_col[cb] = Some(match must_col[cb] {
                                    None => b_bit,
                                    Some(m) => m & b_bit,
                                });
                            }
                        }
                    }
                }
            }
            domains[a_idx] &= a_ok;
            domains[b_idx] &= b_ok;

            if tier == DeductionTier::Hard && found {
                for (r, maybe_must) in must_row.into_iter().enumerate() {
                    let Some(must) = maybe_must else { continue };
                    for c in 0..n {
                        let idx = r * n + c;
                        if idx != a_idx && idx != b_idx {
                            domains[idx] &= !must;
                        }
                    }
                }
                for (c, maybe_must) in must_col.into_iter().enumerate() {
                    let Some(must) = maybe_must else { continue };
                    for r in 0..n {
                        let idx = r * n + c;
                        if idx != a_idx && idx != b_idx {
                            domains[idx] &= !must;
                        }
                    }
                }
            }

            return Ok(());
        }
        Op::Add | Op::Mul => {
            let mut coords = bumpalo::collections::Vec::with_capacity_in(cells.len(), bump);
            for &idx in cells.iter() {
                coords.push((idx / n, idx % n));
            }

            if tier == DeductionTier::Hard {
                let mut per_pos = bumpalo::collections::Vec::with_capacity_in(cells.len(), bump);
                per_pos.resize(cells.len(), 0u64);
                let mut any_mask = 0u64;
                let mut must_row: bumpalo::collections::Vec<Option<u64>> =
                    bumpalo::collections::Vec::with_capacity_in(n, bump);
                let mut must_col: bumpalo::collections::Vec<Option<u64>> =
                    bumpalo::collections::Vec::with_capacity_in(n, bump);
                must_row.resize(n, None);
                must_col.resize(n, None);
                let mut found = false;

                let mut chosen = bumpalo::collections::Vec::with_capacity_in(cells.len(), bump);
                let mut row_bits = bumpalo::collections::Vec::with_capacity_in(n, bump);
                let mut col_bits = bumpalo::collections::Vec::with_capacity_in(n, bump);
                row_bits.resize(n, 0u64);
                col_bits.resize(n, 0u64);

                enumerate_cage_tuples_collect_bump(
                    n,
                    cage,
                    &cells,
                    &coords,
                    domains,
                    0,
                    &mut chosen,
                    &mut per_pos,
                    &mut any_mask,
                    &mut must_row,
                    &mut must_col,
                    &mut found,
                    &mut row_bits,
                    &mut col_bits,
                );

                for (pos, &idx) in cells.iter().enumerate() {
                    domains[idx] &= per_pos[pos];
                }

                if found {
                    let mut in_cage = bumpalo::collections::Vec::with_capacity_in(a, bump);
                    in_cage.resize(a, false);
                    for &idx in &cells {
                        in_cage[idx] = true;
                    }

                    for (r, maybe_must) in must_row.into_iter().enumerate() {
                        let Some(must) = maybe_must else { continue };
                        if must == 0 {
                            continue;
                        }
                        for c in 0..n {
                            let idx = r * n + c;
                            if !in_cage[idx] {
                                domains[idx] &= !must;
                            }
                        }
                    }
                    for (c, maybe_must) in must_col.into_iter().enumerate() {
                        let Some(must) = maybe_must else { continue };
                        if must == 0 {
                            continue;
                        }
                        for r in 0..n {
                            let idx = r * n + c;
                            if !in_cage[idx] {
                                domains[idx] &= !must;
                            }
                        }
                    }
                }

                return Ok(());
            }

            // Easy/Normal tier: no "must" elimination needed.
            let mut per_pos = bumpalo::collections::Vec::with_capacity_in(cells.len(), bump);
            per_pos.resize(cells.len(), 0u64);
            let mut any_mask = 0u64;
            let mut chosen = bumpalo::collections::Vec::with_capacity_in(cells.len(), bump);
            enumerate_cage_tuples_bump(
                cage,
                &cells,
                &coords,
                domains,
                0,
                &mut chosen,
                &mut per_pos,
                &mut any_mask,
            );

            if tier == DeductionTier::Easy {
                for &idx in &cells {
                    domains[idx] &= any_mask;
                }
            } else {
                for (pos, &idx) in cells.iter().enumerate() {
                    domains[idx] &= per_pos[pos];
                }
            }

            return Ok(());
        }
        _ => {}
    }

    Ok(())
}

#[cfg(feature = "alloc-bumpalo")]
#[allow(clippy::too_many_arguments)]
fn enumerate_cage_tuples_bump(
    cage: &Cage,
    cells: &[usize],
    coords: &[(usize, usize)],
    domains: &[u64],
    pos: usize,
    chosen: &mut bumpalo::collections::Vec<u8>,
    per_pos: &mut [u64],
    any_mask: &mut u64,
) {
    if pos == cells.len() {
        if cage_tuple_satisfies(cage, chosen) {
            for (i, &v) in chosen.iter().enumerate() {
                per_pos[i] |= 1u64 << (v as u32);
                *any_mask |= 1u64 << (v as u32);
            }
        }
        return;
    }

    let idx = cells[pos];
    for v in domain_iter(domains[idx]) {
        if violates_in_cage_rowcol(coords, chosen, pos, v) {
            continue;
        }
        chosen.push(v);

        if cage.op == Op::Add {
            let sum: i32 = chosen.iter().map(|&x| x as i32).sum();
            if sum <= cage.target {
                enumerate_cage_tuples_bump(
                    cage,
                    cells,
                    coords,
                    domains,
                    pos + 1,
                    chosen,
                    per_pos,
                    any_mask,
                );
            }
        } else if cage.op == Op::Mul {
            let mut prod: i32 = 1;
            for &x in chosen.iter() {
                prod = prod.saturating_mul(x as i32);
            }
            if prod != 0 && cage.target % prod == 0 {
                enumerate_cage_tuples_bump(
                    cage,
                    cells,
                    coords,
                    domains,
                    pos + 1,
                    chosen,
                    per_pos,
                    any_mask,
                );
            }
        } else {
            enumerate_cage_tuples_bump(
                cage,
                cells,
                coords,
                domains,
                pos + 1,
                chosen,
                per_pos,
                any_mask,
            );
        }

        chosen.pop();
    }
}

#[cfg(feature = "alloc-bumpalo")]
#[allow(clippy::too_many_arguments)]
fn enumerate_cage_tuples_collect_bump(
    n: usize,
    cage: &Cage,
    cells: &[usize],
    coords: &[(usize, usize)],
    domains: &[u64],
    pos: usize,
    chosen: &mut bumpalo::collections::Vec<u8>,
    per_pos: &mut [u64],
    any_mask: &mut u64,
    must_row: &mut [Option<u64>],
    must_col: &mut [Option<u64>],
    found: &mut bool,
    row_bits: &mut [u64],
    col_bits: &mut [u64],
) {
    if pos == cells.len() {
        if cage_tuple_satisfies(cage, chosen) {
            *found = true;
            for (i, &v) in chosen.iter().enumerate() {
                per_pos[i] |= 1u64 << (v as u32);
                *any_mask |= 1u64 << (v as u32);
            }

            row_bits.fill(0u64);
            col_bits.fill(0u64);
            for (i, &(r, c)) in coords.iter().enumerate() {
                row_bits[r] |= 1u64 << (chosen[i] as u32);
                col_bits[c] |= 1u64 << (chosen[i] as u32);
            }
            for r in 0..n {
                if row_bits[r] != 0 {
                    must_row[r] = Some(match must_row[r] {
                        None => row_bits[r],
                        Some(m) => m & row_bits[r],
                    });
                }
            }
            for c in 0..n {
                if col_bits[c] != 0 {
                    must_col[c] = Some(match must_col[c] {
                        None => col_bits[c],
                        Some(m) => m & col_bits[c],
                    });
                }
            }
        }
        return;
    }

    let idx = cells[pos];
    for v in domain_iter(domains[idx]) {
        if violates_in_cage_rowcol(coords, chosen, pos, v) {
            continue;
        }
        chosen.push(v);

        if cage.op == Op::Add {
            let sum: i32 = chosen.iter().map(|&x| x as i32).sum();
            if sum <= cage.target {
                enumerate_cage_tuples_collect_bump(
                    n,
                    cage,
                    cells,
                    coords,
                    domains,
                    pos + 1,
                    chosen,
                    per_pos,
                    any_mask,
                    must_row,
                    must_col,
                    found,
                    row_bits,
                    col_bits,
                );
            }
        } else if cage.op == Op::Mul {
            let mut prod: i32 = 1;
            for &x in chosen.iter() {
                prod = prod.saturating_mul(x as i32);
            }
            if prod != 0 && cage.target % prod == 0 {
                enumerate_cage_tuples_collect_bump(
                    n,
                    cage,
                    cells,
                    coords,
                    domains,
                    pos + 1,
                    chosen,
                    per_pos,
                    any_mask,
                    must_row,
                    must_col,
                    found,
                    row_bits,
                    col_bits,
                );
            }
        } else {
            enumerate_cage_tuples_collect_bump(
                n,
                cage,
                cells,
                coords,
                domains,
                pos + 1,
                chosen,
                per_pos,
                any_mask,
                must_row,
                must_col,
                found,
                row_bits,
                col_bits,
            );
        }

        chosen.pop();
    }
}

#[cfg(not(feature = "alloc-bumpalo"))]
#[allow(clippy::too_many_arguments)]
#[instrument(skip(cage, cells, coords, domains, chosen, per_pos, any_mask), fields(op = ?cage.op, pos, cells_len = cells.len()), level = "debug")]
fn enumerate_cage_tuples(
    cage: &Cage,
    cells: &[usize],
    coords: &[(usize, usize)],
    domains: &[u64],
    pos: usize,
    chosen: &mut Vec<u8>,
    per_pos: &mut [u64],
    any_mask: &mut u64,
) {
    if pos == cells.len() {
        if cage_tuple_satisfies(cage, chosen) {
            for (i, &v) in chosen.iter().enumerate() {
                per_pos[i] |= 1u64 << (v as u32);
                *any_mask |= 1u64 << (v as u32);
            }
        }
        return;
    }

    let idx = cells[pos];
    for v in domain_iter(domains[idx]) {
        if violates_in_cage_rowcol(coords, chosen, pos, v) {
            continue;
        }
        chosen.push(v);

        if cage.op == Op::Add {
            let sum: i32 = chosen.iter().map(|&x| x as i32).sum();
            if sum <= cage.target {
                enumerate_cage_tuples(
                    cage,
                    cells,
                    coords,
                    domains,
                    pos + 1,
                    chosen,
                    per_pos,
                    any_mask,
                );
            }
        } else if cage.op == Op::Mul {
            let mut prod: i32 = 1;
            for &x in chosen.iter() {
                prod = prod.saturating_mul(x as i32);
            }
            if prod != 0 && cage.target % prod == 0 {
                enumerate_cage_tuples(
                    cage,
                    cells,
                    coords,
                    domains,
                    pos + 1,
                    chosen,
                    per_pos,
                    any_mask,
                );
            }
        } else {
            enumerate_cage_tuples(
                cage,
                cells,
                coords,
                domains,
                pos + 1,
                chosen,
                per_pos,
                any_mask,
            );
        }

        chosen.pop();
    }
}

#[cfg(not(feature = "alloc-bumpalo"))]
fn enumerate_cage_tuples_with_must(
    n: usize,
    cage: &Cage,
    cells: &[usize],
    coords: &[(usize, usize)],
    domains: &[u64],
) -> (Vec<u64>, u64, Vec<u64>, Vec<u64>, bool) {
    let mut per_pos = vec![0u64; cells.len()];
    let mut any_mask = 0u64;
    let mut must_row: Vec<Option<u64>> = vec![None; n];
    let mut must_col: Vec<Option<u64>> = vec![None; n];
    let mut found = false;

    enumerate_cage_tuples_collect(
        n,
        cage,
        cells,
        coords,
        domains,
        0,
        &mut Vec::new(),
        &mut per_pos,
        &mut any_mask,
        &mut must_row,
        &mut must_col,
        &mut found,
    );

    let must_row = must_row.into_iter().map(|m| m.unwrap_or(0)).collect();
    let must_col = must_col.into_iter().map(|m| m.unwrap_or(0)).collect();
    (per_pos, any_mask, must_row, must_col, found)
}

#[cfg(not(feature = "alloc-bumpalo"))]
#[allow(clippy::too_many_arguments)]
#[instrument(skip(cage, cells, coords, domains, chosen, per_pos, any_mask, must_row, must_col, found), fields(op = ?cage.op, pos, cells_len = cells.len()), level = "debug")]
fn enumerate_cage_tuples_collect(
    n: usize,
    cage: &Cage,
    cells: &[usize],
    coords: &[(usize, usize)],
    domains: &[u64],
    pos: usize,
    chosen: &mut Vec<u8>,
    per_pos: &mut [u64],
    any_mask: &mut u64,
    must_row: &mut [Option<u64>],
    must_col: &mut [Option<u64>],
    found: &mut bool,
) {
    if pos == cells.len() {
        if cage_tuple_satisfies(cage, chosen) {
            *found = true;
            for (i, &v) in chosen.iter().enumerate() {
                per_pos[i] |= 1u64 << (v as u32);
                *any_mask |= 1u64 << (v as u32);
            }

            let mut row_bits = vec![0u64; n];
            let mut col_bits = vec![0u64; n];
            for (i, &(r, c)) in coords.iter().enumerate() {
                row_bits[r] |= 1u64 << (chosen[i] as u32);
                col_bits[c] |= 1u64 << (chosen[i] as u32);
            }
            for r in 0..n {
                if row_bits[r] != 0 {
                    must_row[r] = Some(match must_row[r] {
                        None => row_bits[r],
                        Some(m) => m & row_bits[r],
                    });
                }
            }
            for c in 0..n {
                if col_bits[c] != 0 {
                    must_col[c] = Some(match must_col[c] {
                        None => col_bits[c],
                        Some(m) => m & col_bits[c],
                    });
                }
            }
        }
        return;
    }

    let idx = cells[pos];
    for v in domain_iter(domains[idx]) {
        if violates_in_cage_rowcol(coords, chosen, pos, v) {
            continue;
        }
        chosen.push(v);

        if cage.op == Op::Add {
            let sum: i32 = chosen.iter().map(|&x| x as i32).sum();
            if sum <= cage.target {
                enumerate_cage_tuples_collect(
                    n,
                    cage,
                    cells,
                    coords,
                    domains,
                    pos + 1,
                    chosen,
                    per_pos,
                    any_mask,
                    must_row,
                    must_col,
                    found,
                );
            }
        } else if cage.op == Op::Mul {
            let mut prod: i32 = 1;
            for &x in chosen.iter() {
                prod = prod.saturating_mul(x as i32);
            }
            if prod != 0 && cage.target % prod == 0 {
                enumerate_cage_tuples_collect(
                    n,
                    cage,
                    cells,
                    coords,
                    domains,
                    pos + 1,
                    chosen,
                    per_pos,
                    any_mask,
                    must_row,
                    must_col,
                    found,
                );
            }
        } else {
            enumerate_cage_tuples_collect(
                n,
                cage,
                cells,
                coords,
                domains,
                pos + 1,
                chosen,
                per_pos,
                any_mask,
                must_row,
                must_col,
                found,
            );
        }

        chosen.pop();
    }
}

fn cage_tuple_satisfies(cage: &Cage, values: &[u8]) -> bool {
    match cage.op {
        Op::Add => values.iter().map(|&v| v as i32).sum::<i32>() == cage.target,
        Op::Mul => values.iter().map(|&v| v as i32).product::<i32>() == cage.target,
        _ => false,
    }
}

fn violates_in_cage_rowcol(coords: &[(usize, usize)], chosen: &[u8], pos: usize, v: u8) -> bool {
    let (r, c) = coords[pos];
    for (i, &prev) in chosen.iter().enumerate() {
        let (pr, pc) = coords[i];
        if (pr == r || pc == c) && prev == v {
            return true;
        }
    }
    false
}

#[instrument(skip(puzzle, rules, state, cage), fields(op = ?cage.op, cells = cage.cells.len()), level = "debug")]
fn cage_feasible(
    puzzle: &Puzzle,
    rules: Ruleset,
    state: &State,
    cage: &Cage,
) -> Result<bool, SolveError> {
    let n = state.n as usize;
    let mut assigned: Vec<i32> = Vec::new();
    let mut unassigned: Vec<usize> = Vec::new();

    for cell in &cage.cells {
        let idx = cell.0 as usize;
        let v = state.grid[idx];
        if v == 0 {
            unassigned.push(idx);
        } else {
            assigned.push(v as i32);
        }
    }

    match cage.op {
        Op::Eq => {
            if cage.cells.len() != 1 {
                return Err(CoreError::InvalidOpForCageSize {
                    op: cage.op,
                    len: cage.cells.len(),
                }
                .into());
            }
            let t = cage.target;
            if assigned.is_empty() {
                return Ok(true);
            }
            return Ok(assigned[0] == t);
        }
        Op::Sub | Op::Div if rules.sub_div_two_cell_only && cage.cells.len() != 2 => {
            return Err(CoreError::SubDivMustBeTwoCell.into());
        }
        _ => {}
    }

    if unassigned.is_empty() {
        return Ok(cage_satisfied(cage, &assigned));
    }

    match cage.op {
        Op::Sub => {
            // Two-cell only: check existence against remaining domain.
            let (a_idx, b_idx) = (cage.cells[0].0 as usize, cage.cells[1].0 as usize);
            Ok(two_cell_sub_feasible(
                puzzle,
                state,
                a_idx,
                b_idx,
                cage.target,
            )?)
        }
        Op::Div => {
            let (a_idx, b_idx) = (cage.cells[0].0 as usize, cage.cells[1].0 as usize);
            Ok(two_cell_div_feasible(
                puzzle,
                state,
                a_idx,
                b_idx,
                cage.target,
            )?)
        }
        Op::Add => {
            let sum_assigned: i32 = assigned.iter().sum();
            if sum_assigned > cage.target {
                return Ok(false);
            }
            let mut min_remaining = 0i32;
            let mut max_remaining = 0i32;
            for &idx in &unassigned {
                let row = idx / n;
                let col = idx % n;
                let dom = domain_for_cell(puzzle, state, idx, row, col)?;
                let (mn, mx) =
                    domain_min_max(dom).ok_or(SolveError::Core(CoreError::TargetMustBeNonZero))?;
                min_remaining += mn as i32;
                max_remaining += mx as i32;
            }
            let t = cage.target;
            Ok(sum_assigned + min_remaining <= t && t <= sum_assigned + max_remaining)
        }
        Op::Mul => {
            let mut prod_assigned: i32 = 1;
            for &v in &assigned {
                prod_assigned = prod_assigned.saturating_mul(v);
            }
            if prod_assigned == 0 || cage.target % prod_assigned != 0 {
                return Ok(false);
            }
            let mut min_prod: i32 = 1;
            let mut max_prod: i32 = 1;
            for &idx in &unassigned {
                let row = idx / n;
                let col = idx % n;
                let dom = domain_for_cell(puzzle, state, idx, row, col)?;
                let (mn, mx) =
                    domain_min_max(dom).ok_or(SolveError::Core(CoreError::TargetMustBeNonZero))?;
                min_prod = min_prod.saturating_mul(mn as i32);
                max_prod = max_prod.saturating_mul(mx as i32);
            }
            let t = cage.target;
            Ok(prod_assigned.saturating_mul(min_prod) <= t
                && t <= prod_assigned.saturating_mul(max_prod))
        }
        Op::Eq => unreachable!("Eq cages are handled earlier in cage_feasible"),
    }
}

fn cage_satisfied(cage: &Cage, values: &[i32]) -> bool {
    match cage.op {
        Op::Eq => values.len() == 1 && values[0] == cage.target,
        Op::Add => values.iter().sum::<i32>() == cage.target,
        Op::Mul => values.iter().product::<i32>() == cage.target,
        Op::Sub => values.len() == 2 && (values[0] - values[1]).abs() == cage.target,
        Op::Div => {
            if values.len() != 2 {
                return false;
            }
            let a = values[0].max(values[1]);
            let b = values[0].min(values[1]);
            b != 0 && a % b == 0 && a / b == cage.target
        }
    }
}

fn two_cell_sub_feasible(
    puzzle: &Puzzle,
    state: &State,
    a: usize,
    b: usize,
    target: i32,
) -> Result<bool, CoreError> {
    let n = state.n as usize;
    let av = state.grid[a];
    let bv = state.grid[b];
    match (av, bv) {
        (0, 0) => Ok(true),
        (x, 0) => {
            let row = b / n;
            let col = b % n;
            let dom = domain_for_cell(puzzle, state, b, row, col)?;
            Ok(domain_iter(dom).any(|y| (x as i32 - y as i32).abs() == target))
        }
        (0, y) => {
            let row = a / n;
            let col = a % n;
            let dom = domain_for_cell(puzzle, state, a, row, col)?;
            Ok(domain_iter(dom).any(|x| (x as i32 - y as i32).abs() == target))
        }
        (x, y) => Ok((x as i32 - y as i32).abs() == target),
    }
}

fn two_cell_div_feasible(
    puzzle: &Puzzle,
    state: &State,
    a: usize,
    b: usize,
    target: i32,
) -> Result<bool, CoreError> {
    let n = state.n as usize;
    let av = state.grid[a];
    let bv = state.grid[b];
    let ok_pair = |x: u8, y: u8| {
        let (num, den) = if x >= y { (x, y) } else { (y, x) };
        den != 0 && (num as i32) == (den as i32).saturating_mul(target)
    };
    match (av, bv) {
        (0, 0) => Ok(true),
        (x, 0) => {
            let row = b / n;
            let col = b % n;
            let dom = domain_for_cell(puzzle, state, b, row, col)?;
            Ok(domain_iter(dom).any(|y| ok_pair(x, y)))
        }
        (0, y) => {
            let row = a / n;
            let col = a % n;
            let dom = domain_for_cell(puzzle, state, a, row, col)?;
            Ok(domain_iter(dom).any(|x| ok_pair(x, y)))
        }
        (x, y) => Ok(ok_pair(x, y)),
    }
}

fn place(state: &mut State, row: usize, col: usize, d: u8) {
    let idx = row * (state.n as usize) + col;
    state.grid[idx] = d;
    state.row_mask[row] |= 1u64 << (d as u32);
    state.col_mask[col] |= 1u64 << (d as u32);
}

fn unplace(state: &mut State, row: usize, col: usize, d: u8) {
    let idx = row * (state.n as usize) + col;
    state.grid[idx] = 0;
    state.row_mask[row] &= !(1u64 << (d as u32));
    state.col_mask[col] &= !(1u64 << (d as u32));

    // Tier 2.2: Invalidate MRV cache when domains change (unplace expands domains)
    state.mrv_cache.valid = false;
}

fn full_domain(n: u8) -> u64 {
    // bits 1..=n set
    if n >= 63 {
        u64::MAX
    } else {
        ((1u64 << (n as u32 + 1)) - 1) & !1u64
    }
}

fn domain_min_max(dom: u64) -> Option<(u8, u8)> {
    if dom == 0 {
        return None;
    }
    let min = dom.trailing_zeros() as u8;
    let max = (63 - dom.leading_zeros()) as u8;
    Some((min, max))
}

fn domain_iter(dom: u64) -> impl Iterator<Item = u8> {
    let mut mask = dom;
    core::iter::from_fn(move || {
        if mask == 0 {
            return None;
        }
        let bit = mask.trailing_zeros();
        mask &= mask - 1;
        Some(bit as u8)
    })
}

#[cfg(test)]
mod tests {
    use kenken_core::format::sgt_desc::parse_keen_desc;

    use super::*;

    #[test]
    fn counts_two_solutions_for_simple_2x2() {
        let p = parse_keen_desc(2, "b__,a3a3").unwrap();
        let count = count_solutions_up_to(&p, Ruleset::keen_baseline(), 2).unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn stops_counting_at_limit() {
        let p = parse_keen_desc(2, "b__,a3a3").unwrap();
        let count = count_solutions_up_to(&p, Ruleset::keen_baseline(), 1).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn solve_one_returns_a_solution_when_one_exists() {
        let p = parse_keen_desc(2, "b__,a3a3").unwrap();
        let sol = solve_one(&p, Ruleset::keen_baseline()).unwrap().unwrap();
        assert_eq!(sol.n, 2);
        assert_eq!(sol.grid.len(), 4);
    }

    #[test]
    fn solve_one_with_deductions_works() {
        let p = parse_keen_desc(2, "b__,a3a3").unwrap();
        let sol = solve_one_with_deductions(&p, Ruleset::keen_baseline(), DeductionTier::Hard)
            .unwrap()
            .unwrap();
        assert_eq!(sol.n, 2);
        assert_eq!(sol.grid.len(), 4);
    }
}

/// Kani formal verification harnesses for Latin constraint invariants.
///
/// These proofs verify that the row_mask/col_mask representation correctly
/// enforces Latin square constraints (no duplicate digits in rows or columns).
#[cfg(kani)]
mod kani_verification {
    use super::*;

    /// Proves full_domain(n) has exactly n bits set (bits 1..=n).
    #[kani::proof]
    fn full_domain_has_n_bits() {
        let n: u8 = kani::any();
        kani::assume(n >= 1 && n <= 30);

        let dom = full_domain(n);
        let count = dom.count_ones();

        kani::assert(count == n as u32, "full_domain should have exactly n bits");

        // Verify bit 0 is never set (digits are 1-indexed)
        kani::assert((dom & 1) == 0, "bit 0 should never be set");

        // Verify all bits 1..=n are set
        for d in 1..=n {
            kani::assert((dom & (1u32 << d)) != 0, "bit d should be set");
        }
    }

    /// Proves place() sets the digit bit in row_mask.
    #[kani::proof]
    fn place_sets_row_mask() {
        let n: u8 = kani::any();
        kani::assume(n >= 2 && n <= 9);

        let row: usize = kani::any();
        let col: usize = kani::any();
        let d: u8 = kani::any();
        kani::assume(row < n as usize && col < n as usize);
        kani::assume(d >= 1 && d <= n);

        let a = (n as usize) * (n as usize);
        let mut state = State {
            n,
            grid: vec![0; a],
            row_mask: vec![0u32; n as usize],
            col_mask: vec![0u32; n as usize],
            cage_of_cell: vec![0; a],
        };

        place(&mut state, row, col, d);
        let bit_after = state.row_mask[row] & (1u32 << d);

        kani::assert(bit_after != 0, "place should set digit bit in row_mask");
    }

    /// Proves place() sets the digit bit in col_mask.
    #[kani::proof]
    fn place_sets_col_mask() {
        let n: u8 = kani::any();
        kani::assume(n >= 2 && n <= 9);

        let row: usize = kani::any();
        let col: usize = kani::any();
        let d: u8 = kani::any();
        kani::assume(row < n as usize && col < n as usize);
        kani::assume(d >= 1 && d <= n);

        let a = (n as usize) * (n as usize);
        let mut state = State {
            n,
            grid: vec![0; a],
            row_mask: vec![0u32; n as usize],
            col_mask: vec![0u32; n as usize],
            cage_of_cell: vec![0; a],
        };

        place(&mut state, row, col, d);
        let bit_after = state.col_mask[col] & (1u32 << d);

        kani::assert(bit_after != 0, "place should set digit bit in col_mask");
    }

    /// Proves unplace() clears the digit bit in row_mask.
    #[kani::proof]
    fn unplace_clears_row_mask() {
        let n: u8 = kani::any();
        kani::assume(n >= 2 && n <= 9);

        let row: usize = kani::any();
        let col: usize = kani::any();
        let d: u8 = kani::any();
        kani::assume(row < n as usize && col < n as usize);
        kani::assume(d >= 1 && d <= n);

        let a = (n as usize) * (n as usize);
        let mut state = State {
            n,
            grid: vec![0; a],
            row_mask: vec![0u32; n as usize],
            col_mask: vec![0u32; n as usize],
            cage_of_cell: vec![0; a],
        };

        // Place then unplace
        place(&mut state, row, col, d);
        unplace(&mut state, row, col, d);

        let bit_after = state.row_mask[row] & (1u32 << d);
        kani::assert(bit_after == 0, "unplace should clear digit bit in row_mask");
    }

    /// Proves unplace() clears the digit bit in col_mask.
    #[kani::proof]
    fn unplace_clears_col_mask() {
        let n: u8 = kani::any();
        kani::assume(n >= 2 && n <= 9);

        let row: usize = kani::any();
        let col: usize = kani::any();
        let d: u8 = kani::any();
        kani::assume(row < n as usize && col < n as usize);
        kani::assume(d >= 1 && d <= n);

        let a = (n as usize) * (n as usize);
        let mut state = State {
            n,
            grid: vec![0; a],
            row_mask: vec![0u32; n as usize],
            col_mask: vec![0u32; n as usize],
            cage_of_cell: vec![0; a],
        };

        // Place then unplace
        place(&mut state, row, col, d);
        unplace(&mut state, row, col, d);

        let bit_after = state.col_mask[col] & (1u32 << d);
        kani::assert(bit_after == 0, "unplace should clear digit bit in col_mask");
    }

    /// Proves place/unplace roundtrip restores masks to original state.
    #[kani::proof]
    fn place_unplace_roundtrip() {
        let n: u8 = kani::any();
        kani::assume(n >= 2 && n <= 9);

        let row: usize = kani::any();
        let col: usize = kani::any();
        let d: u8 = kani::any();
        kani::assume(row < n as usize && col < n as usize);
        kani::assume(d >= 1 && d <= n);

        let a = (n as usize) * (n as usize);
        let mut state = State {
            n,
            grid: vec![0; a],
            row_mask: vec![0u32; n as usize],
            col_mask: vec![0u32; n as usize],
            cage_of_cell: vec![0; a],
        };

        let row_before = state.row_mask[row];
        let col_before = state.col_mask[col];
        let grid_before = state.grid[row * (n as usize) + col];

        place(&mut state, row, col, d);
        unplace(&mut state, row, col, d);

        kani::assert(
            state.row_mask[row] == row_before,
            "row_mask should be restored after roundtrip",
        );
        kani::assert(
            state.col_mask[col] == col_before,
            "col_mask should be restored after roundtrip",
        );
        kani::assert(
            state.grid[row * (n as usize) + col] == grid_before,
            "grid cell should be restored after roundtrip",
        );
    }

    /// Proves domain computation excludes digits placed in the same row.
    ///
    /// Key Latin constraint: if digit d is placed in row r,
    /// then domain for any other cell in row r must NOT include d.
    #[kani::proof]
    fn domain_excludes_placed_in_row() {
        let n: u8 = kani::any();
        kani::assume(n >= 2 && n <= 9);

        let row: usize = kani::any();
        let col1: usize = kani::any();
        let col2: usize = kani::any();
        let d: u8 = kani::any();
        kani::assume(row < n as usize);
        kani::assume(col1 < n as usize && col2 < n as usize);
        kani::assume(col1 != col2);
        kani::assume(d >= 1 && d <= n);

        let a = (n as usize) * (n as usize);
        let mut state = State {
            n,
            grid: vec![0; a],
            row_mask: vec![0u32; n as usize],
            col_mask: vec![0u32; n as usize],
            cage_of_cell: vec![0; a],
        };

        // Place digit d at (row, col1)
        place(&mut state, row, col1, d);

        // Compute domain for cell (row, col2) - another cell in same row
        let full = full_domain(n);
        let domain = full & !state.row_mask[row] & !state.col_mask[col2];

        // Domain should NOT include digit d
        kani::assert(
            (domain & (1u32 << d)) == 0,
            "domain should exclude digit placed in same row",
        );
    }

    /// Proves domain computation excludes digits placed in the same column.
    ///
    /// Key Latin constraint: if digit d is placed in column c,
    /// then domain for any other cell in column c must NOT include d.
    #[kani::proof]
    fn domain_excludes_placed_in_col() {
        let n: u8 = kani::any();
        kani::assume(n >= 2 && n <= 9);

        let col: usize = kani::any();
        let row1: usize = kani::any();
        let row2: usize = kani::any();
        let d: u8 = kani::any();
        kani::assume(col < n as usize);
        kani::assume(row1 < n as usize && row2 < n as usize);
        kani::assume(row1 != row2);
        kani::assume(d >= 1 && d <= n);

        let a = (n as usize) * (n as usize);
        let mut state = State {
            n,
            grid: vec![0; a],
            row_mask: vec![0u32; n as usize],
            col_mask: vec![0u32; n as usize],
            cage_of_cell: vec![0; a],
        };

        // Place digit d at (row1, col)
        place(&mut state, row1, col, d);

        // Compute domain for cell (row2, col) - another cell in same column
        let full = full_domain(n);
        let domain = full & !state.row_mask[row2] & !state.col_mask[col];

        // Domain should NOT include digit d
        kani::assert(
            (domain & (1u32 << d)) == 0,
            "domain should exclude digit placed in same column",
        );
    }

    /// Proves grid cell value matches placed digit.
    #[kani::proof]
    fn place_sets_grid_value() {
        let n: u8 = kani::any();
        kani::assume(n >= 2 && n <= 9);

        let row: usize = kani::any();
        let col: usize = kani::any();
        let d: u8 = kani::any();
        kani::assume(row < n as usize && col < n as usize);
        kani::assume(d >= 1 && d <= n);

        let a = (n as usize) * (n as usize);
        let mut state = State {
            n,
            grid: vec![0; a],
            row_mask: vec![0u32; n as usize],
            col_mask: vec![0u32; n as usize],
            cage_of_cell: vec![0; a],
        };

        place(&mut state, row, col, d);
        let idx = row * (n as usize) + col;

        kani::assert(state.grid[idx] == d, "grid should contain placed digit");
    }
}
