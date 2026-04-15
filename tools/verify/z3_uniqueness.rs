// Z3 uniqueness sketch stub
#![allow(unused)]
#[cfg(feature = "verification")]
pub fn verify_uniqueness_stub(n: i64, solution: &[i64]) -> Result<(), String> {
    use z3::{
        Config, SatResult, Solver,
        ast::{Ast, Int},
        with_z3_config,
    };
    if solution.len() as i64 != n * n {
        return Err("bad solution size".into());
    }
    let cfg = Config::new();

    with_z3_config(&cfg, || {
        let solver = Solver::new();

        // Vars
        let cells: Vec<Int> = (0..(n * n))
            .map(|i| Int::new_const(format!("cell_{i}")))
            .collect();

        // Domain 1..=n
        for c in &cells {
            solver.assert(c.ge(Int::from_i64(1)));
            solver.assert(c.le(Int::from_i64(n)));
        }

        // Distinct rows/cols (Latin)
        for r in 0..n {
            let row: Vec<&Int> = (0..n).map(|c| &cells[(r * n + c) as usize]).collect();
            solver.assert(Int::distinct(&row));
        }
        for c in 0..n {
            let col: Vec<&Int> = (0..n).map(|r| &cells[(r * n + c) as usize]).collect();
            solver.assert(Int::distinct(&col));
        }

        // Differ from known solution at least one cell
        let mut diffs = Vec::with_capacity((n * n) as usize);
        for i in 0..(n * n) {
            let known = Int::from_i64(solution[i as usize]);
            diffs.push(cells[i as usize].eq(&known).not());
        }
        let diff_refs: Vec<&_> = diffs.iter().collect();
        let any_diff = z3::ast::Bool::or(&diff_refs);
        solver.assert(&any_diff);

        // If SAT => another solution exists; UNSAT => unique
        match solver.check() {
            SatResult::Unsat => Ok(()),
            _ => Err("not unique".into()),
        }
    })
}
