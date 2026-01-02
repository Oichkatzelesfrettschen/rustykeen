#[cfg(feature = "alloc-mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use kenken_core::format::sgt_desc::parse_keen_desc;
use kenken_core::puzzle::{Cage, CellId, Puzzle};
use kenken_core::rules::{Op, Ruleset};
use kenken_solver::{
    DeductionTier, count_solutions_up_to_with_deductions, solve_one_with_deductions,
};
use smallvec::SmallVec;
use std::time::Instant;

#[cfg(feature = "telemetry-subscriber")]
fn init_tracing() {
    use tracing_subscriber::EnvFilter;

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("kenken_solver=trace,kenken_gen=trace,kenken_io=info,kenken_cli=info")
    });

    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}

#[cfg(not(feature = "telemetry-subscriber"))]
fn init_tracing() {}

fn usage() -> &'static str {
    "kenken-cli\n\
\n\
USAGE:\n\
  kenken-cli solve --n <N> --desc <DESC> [--tier <none|easy|normal|hard>]\n\
  kenken-cli count --n <N> --desc <DESC> [--tier <none|easy|normal|hard>] [--limit <L>]\n\
  kenken-cli benchmark --n <N> --count <C> [--tier <none|easy|normal|hard>]\n\
\n\
EXAMPLES:\n\
  kenken-cli solve --n 2 --desc b__,a3a3 --tier normal\n\
  kenken-cli count --n 2 --desc b__,a3a3 --limit 2\n\
  kenken-cli benchmark --n 4 --count 10 --tier normal\n"
}

fn parse_tier(s: &str) -> Option<DeductionTier> {
    match s {
        "none" => Some(DeductionTier::None),
        "easy" => Some(DeductionTier::Easy),
        "normal" => Some(DeductionTier::Normal),
        "hard" => Some(DeductionTier::Hard),
        _ => None,
    }
}

fn parse_arg_value(args: &[String], i: &mut usize) -> Result<String, String> {
    *i += 1;
    args.get(*i)
        .cloned()
        .ok_or_else(|| "missing value".to_string())
}

fn main() {
    init_tracing();
    if let Err(err) = run() {
        eprintln!("{err}\n\n{}", usage());
        std::process::exit(2);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        return Err("missing command".to_string());
    }

    let cmd = args[1].as_str();
    let mut n: Option<u8> = None;
    let mut desc: Option<String> = None;
    let mut tier: DeductionTier = DeductionTier::Normal;
    let mut limit: u32 = 2;
    let mut count: u32 = 1;

    let mut i = 2usize;
    while i < args.len() {
        match args[i].as_str() {
            "--n" | "-n" => {
                let v = parse_arg_value(&args, &mut i)?;
                n = Some(v.parse::<u8>().map_err(|_| "invalid --n".to_string())?);
            }
            "--desc" | "-d" => {
                desc = Some(parse_arg_value(&args, &mut i)?);
            }
            "--tier" => {
                let v = parse_arg_value(&args, &mut i)?;
                tier = parse_tier(&v).ok_or_else(|| "invalid --tier".to_string())?;
            }
            "--limit" => {
                let v = parse_arg_value(&args, &mut i)?;
                limit = v
                    .parse::<u32>()
                    .map_err(|_| "invalid --limit".to_string())?;
            }
            "--count" => {
                let v = parse_arg_value(&args, &mut i)?;
                count = v
                    .parse::<u32>()
                    .map_err(|_| "invalid --count".to_string())?;
            }
            "--help" | "-h" => {
                println!("{}", usage());
                return Ok(());
            }
            other => {
                return Err(format!("unknown arg: {other}"));
            }
        }
        i += 1;
    }

    let Some(n) = n else {
        return Err("missing required flag: --n".to_string());
    };

    let rules = Ruleset::keen_baseline();

    match cmd {
        "solve" => {
            let Some(desc) = desc else {
                return Err("'solve' requires --desc".to_string());
            };
            let Ok(puzzle) = parse_keen_desc(n, &desc) else {
                return Err("failed to parse --desc".to_string());
            };

            let sol = solve_one_with_deductions(&puzzle, rules, tier).unwrap_or(None);
            let Some(sol) = sol else {
                println!("no-solution");
                return Ok(());
            };
            println!("n={}", sol.n);
            for r in 0..(sol.n as usize) {
                let row = &sol.grid[r * (sol.n as usize)..(r + 1) * (sol.n as usize)];
                let line = row
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                println!("{line}");
            }
        }
        "count" => {
            let Some(desc) = desc else {
                return Err("'count' requires --desc".to_string());
            };
            let Ok(puzzle) = parse_keen_desc(n, &desc) else {
                return Err("failed to parse --desc".to_string());
            };

            let cnt =
                count_solutions_up_to_with_deductions(&puzzle, rules, tier, limit).unwrap_or(0);
            println!("{cnt}");
        }
        "benchmark" => {
            benchmark_puzzles(n, count, tier, rules)?;
        }
        _ => {
            return Err(format!("unknown command: {cmd}"));
        }
    }

    Ok(())
}

fn benchmark_puzzles(n: u8, count: u32, tier: DeductionTier, rules: Ruleset) -> Result<(), String> {
    // Generate benchmark puzzle using cyclic Latin square pattern
    // For sizes 2-16: Uses SGT format
    // For sizes 17-32: Creates Puzzle objects directly
    let puzzle = get_benchmark_puzzle(n)?;

    // Validate the puzzle before benchmarking
    puzzle
        .validate(rules)
        .map_err(|e| format!("Puzzle validation failed: {}", e))?;

    let start = Instant::now();
    let mut solved = 0u32;

    for _ in 0..count {
        if solve_one_with_deductions(&puzzle, rules, tier)
            .unwrap_or(None)
            .is_some()
        {
            solved += 1;
        }
    }

    let elapsed = start.elapsed().as_secs_f64();
    let rate = if elapsed > 0.0 {
        solved as f64 / elapsed
    } else {
        0.0
    };

    println!("Puzzles/second: {:.3}", rate);

    Ok(())
}

fn get_benchmark_puzzle(n: u8) -> Result<Puzzle, String> {
    // Return all-singleton benchmark puzzles using cyclic Latin square pattern.
    // Each cell is its own 1-cell cage with value: ((row + col) % n) + 1
    //
    // For sizes 2-16: Use SGT format (string-based)
    // For sizes 17-32: Create Puzzle objects directly (avoids SGT format 16-cell limit)

    if n <= 16 {
        // Use SGT format for sizes 2-16
        let total_positions = 2 * (n as usize) * ((n as usize) - 1) + 1;
        let block_struct = format!("_{}", total_positions);

        // Generate clues: cyclic pattern a1 a2 ... an a2 a3 ... a1 etc.
        // For cyclic Latin square: cell at row i, col j has value ((i+j) % n) + 1
        let mut clues = String::new();
        for cell_idx in 0..(n as usize * n as usize) {
            let row = cell_idx / (n as usize);
            let col = cell_idx % (n as usize);
            let value = ((row + col) % (n as usize)) + 1;
            clues.push('a');
            clues.push_str(&value.to_string());
        }

        let desc = format!("{},{}", block_struct, clues);
        parse_keen_desc(n, &desc)
            .map_err(|e| format!("Failed to parse SGT format for n={}: {}", n, e))
    } else if n <= 32 {
        // Create Puzzle object directly for sizes 17-32
        let mut cages = Vec::new();

        // Each cell is a 1-cell cage with value ((row + col) % n) + 1
        for cell_idx in 0..(n as usize * n as usize) {
            let row = cell_idx / (n as usize);
            let col = cell_idx % (n as usize);
            let value = ((row + col) % (n as usize)) + 1;

            let cells: SmallVec<[CellId; 6]> = vec![CellId(cell_idx as u16)].into_iter().collect();
            let cage = Cage {
                cells,
                op: Op::Eq,
                target: value as i32,
            };
            cages.push(cage);
        }

        Ok(Puzzle { n, cages })
    } else {
        Err(format!("Grid size {} not supported. Max: 32x32", n))
    }
}

#[cfg(test)]
mod bench_puzzle_tests {
    use super::*;

    #[test]
    fn benchmark_puzzles_generate_valid_for_all_sizes() {
        // Verify all sizes 2-32 generate valid puzzles
        let rules = Ruleset::keen_baseline();

        for n in 2..=32 {
            let result = get_benchmark_puzzle(n as u8);
            assert!(result.is_ok(), "Failed to generate puzzle for n={}", n);

            let puzzle = result.unwrap();

            // Verify basic structure
            assert_eq!(puzzle.n, n as u8, "Puzzle grid size mismatch for n={}", n);

            // Verify we have n*n cages (one per cell)
            let expected_cages = (n * n) as usize;
            assert_eq!(
                puzzle.cages.len(),
                expected_cages,
                "Expected {} cages for n={}, got {}",
                expected_cages,
                n,
                puzzle.cages.len()
            );

            // Verify the puzzle is valid
            assert!(
                puzzle.validate(rules).is_ok(),
                "Puzzle validation failed for n={}",
                n
            );
        }
    }
}
