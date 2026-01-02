use kenken_core::format::sgt_desc::parse_keen_desc;
use kenken_core::rules::Ruleset;
use kenken_solver::solve_one_with_deductions;
use kenken_solver::DeductionTier;

fn main() {
    let rules = Ruleset::keen_baseline();

    let puzzles = vec![
        (4, "a__b,a5bc,d5ec,dee", "4x4_mixed"),
        (6, "a___bc,a_def,gdhij,gkhij,kllm,nompm", "6x6_standard"),
    ];

    for (n, desc, label) in puzzles {
        eprintln!("\nTesting: {} ({})", label, desc);
        match parse_keen_desc(n, desc) {
            Ok(puzzle) => {
                eprintln!("  Parsed OK");
                match puzzle.validate(rules) {
                    Ok(_) => {
                        eprintln!("  Validated OK");
                        let result = solve_one_with_deductions(&puzzle, rules, DeductionTier::Normal);
                        eprintln!("  Solve result: {:?}", result.is_ok());
                    }
                    Err(e) => eprintln!("  Validation error: {:?}", e),
                }
            }
            Err(e) => eprintln!("  Parse error: {:?}", e),
        }
    }
}
