#[cfg(feature = "alloc-mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use kenken_core::format::sgt_desc::parse_keen_desc;
use kenken_core::rules::Ruleset;
use kenken_solver::{
    DeductionTier, count_solutions_up_to_with_deductions, solve_one_with_deductions,
};

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
\n\
EXAMPLES:\n\
  kenken-cli solve --n 2 --desc b__,a3a3 --tier normal\n\
  kenken-cli count --n 2 --desc b__,a3a3 --limit 2\n"
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
    let Some(desc) = desc else {
        return Err("missing required flag: --desc".to_string());
    };

    let rules = Ruleset::keen_baseline();
    let Ok(puzzle) = parse_keen_desc(n, &desc) else {
        return Err("failed to parse --desc".to_string());
    };

    match cmd {
        "solve" => {
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
            let count =
                count_solutions_up_to_with_deductions(&puzzle, rules, tier, limit).unwrap_or(0);
            println!("{count}");
        }
        _ => {
            return Err(format!("unknown command: {cmd}"));
        }
    }

    Ok(())
}
