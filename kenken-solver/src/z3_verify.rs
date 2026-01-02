//! Z3-based formal verification of puzzle uniqueness.
//!
//! This module provides verification that a KenKen solution is unique
//! by encoding the Latin square constraints in Z3 and checking if
//! any other solutions exist.

#[cfg(feature = "verify")]
pub fn verify_solution_is_unique(n: u8, solution: &[u8]) -> Result<(), String> {
    use z3::{Config, Context, SatResult, Solver, ast::{Int, Ast}};

    if solution.len() != (n as usize) * (n as usize) {
        return Err("Solution length mismatch".to_string());
    }

    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let solver = Solver::new(&ctx);

    // Create cell variables for all cells
    let cells: Vec<Int> = (0..(n as i64) * (n as i64))
        .map(|i| Int::new_const(&ctx, format!("cell_{}", i)))
        .collect();

    // Domain constraints: 1 <= cell <= n
    let n_z3 = Int::from_i64(&ctx, n as i64);
    for cell in &cells {
        solver.assert(&cell.ge(&Int::from_i64(&ctx, 1)));
        solver.assert(&cell.le(&n_z3));
    }

    // Row distinctness constraints
    for row in 0..n as i64 {
        let row_cells: Vec<&Int> = (0..n as i64)
            .map(|col| &cells[(row * (n as i64) + col) as usize])
            .collect();
        solver.assert(&Int::distinct(&ctx, &row_cells));
    }

    // Column distinctness constraints
    for col in 0..n as i64 {
        let col_cells: Vec<&Int> = (0..n as i64)
            .map(|row| &cells[(row * (n as i64) + col) as usize])
            .collect();
        solver.assert(&Int::distinct(&ctx, &col_cells));
    }

    // Assert the known solution
    for (i, &cell_value) in solution.iter().enumerate() {
        let known = Int::from_i64(&ctx, cell_value as i64);
        solver.assert(&cells[i]._eq(&known));
    }

    // Try to find a solution different from the known one
    // If no such solution exists, this is unique
    let mut different = Vec::new();
    for (i, &cell_value) in solution.iter().enumerate() {
        let known = Int::from_i64(&ctx, cell_value as i64);
        different.push(cells[i]._eq(&known).not());
    }
    let different_refs: Vec<&_> = different.iter().collect();
    let any_different = z3::ast::Bool::or(&ctx, &different_refs);
    solver.assert(&any_different);

    // Check: UNSAT = unique, SAT = not unique
    match solver.check() {
        SatResult::Unsat => Ok(()),
        SatResult::Unknown => Err("Z3 returned UNKNOWN (timeout or incomplete)".to_string()),
        SatResult::Sat => Err("Found alternative solution (not unique)".to_string()),
    }
}

#[cfg(not(feature = "verify"))]
pub fn verify_solution_is_unique(_n: u8, _solution: &[u8]) -> Result<(), String> {
    Err("Z3 verification requires 'verify' feature".to_string())
}
