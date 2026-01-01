# Formal Verification Stack (2026)

## Goals
- Prove puzzle uniqueness and validity; ensure solver/generator panic freedom; validate architecture state machines.

## Tools
- Z3 (crate: `z3`): Encode Latin+cage constraints; assert existence of S2 != S1; UNSAT => unique.
- Kani (crate/tool: `kani`, `cargo-kani`): #[kani::proof] harnesses for index math, bit operations, bounds.
- Creusot (crate: `creusot`): Contracts on critical functions; translated to Why3 for deductive proofs.
- TLA+: External spec for Hybrid Solver transitions; check liveness/deadlock freedom.
- Bolero (crate: `bolero`): Fuzz generator/solver APIs for performance coverage when proofs are slow.

## Integration
- Feature gate `verification` enables z3/kani/creusot/bolero; keep optional in CI.
- Place Kani harnesses under `tests/kani/` or `tools/verify/`; run `cargo kani` in verification CI job.
- Z3 runs only on final candidates to certify uniqueness; cache results via rkyv.
- Bolero targets parsing, cage assignment, and solver callbacks; limit runtime with CI budget.

## Example Kani Harness
```rust
#[kani::proof]
fn grid_index_is_in_bounds() {
    let n: usize = kani::any();
    kani::assume(n > 0 && n <= 9);
    let x: usize = kani::any();
    let y: usize = kani::any();
    kani::assume(x < n && y < n);
    let idx = y * n + x; // row-major
    assert!(idx < n * n);
}
```

## Example Z3 Uniqueness Sketch
- Build Int vars cell[r][c] in 1..N, assert row/col all-different, cages arithmetic.
- Add disjointness: exists (r,c) s.t. cell[r][c] != known_solution[r][c].
- If solver returns UNSAT, uniqueness is proven.
