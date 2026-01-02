# Rocq → OCaml → Rust Translation Mapping

## Overview

This document specifies the type and function mappings used to translate verified KenKen solver code from Rocq through OCaml extraction to pure Rust.

## Type Mappings

### Basic Types

| Rocq | OCaml | Rust | Notes |
|------|-------|------|-------|
| `nat` | `int` | `u32` / `u64` | Bounds checking required |
| `list A` | `'a list` | `Vec<A>` | Structural conversion |
| `bool` | `bool` | `bool` | Direct mapping |
| `Prop` | `unit` | `()` (erased) | Proofs erased during extraction |

### Domain & State

| Rocq | OCaml | Rust | Purpose |
|------|-------|------|---------|
| `Domain` = `list nat` | `int list` | `Vec<u8>` | Cell value possibilities |
| `State` = `list Domain` | `int list list` | `Vec<Vec<u8>>` | All cell domains |
| `Cell` = `nat` | `int` | `u32` | Cell linear index |
| `Solution` = `list nat` | `int list` | `Vec<u8>` | Final assignments |

### Puzzle & Cage

| Rocq | OCaml | Rust | Fields |
|------|-------|------|--------|
| `Cage` record | Record type | `Cage` struct | cells, op, target |
| `Puzzle` record | Record type | `Puzzle` struct | n, cages |
| `Operation` enum | Variant type | `Op` enum | Add, Sub, Mul, Div, Eq |

## Function Mappings

### Core Verification Functions

```
Rocq: valid_solution (puzzle : Puzzle) (sol : Solution) : Prop
↓ Extract
OCaml: val valid_solution : puzzle -> solution -> unit
↓ Translate
Rust: pub fn verify_solution(puzzle: &Puzzle, sol: &[u8]) -> Result<(), String>
```

### Search & MRV Algorithm

```
Rocq: search_spec (puzzle : Puzzle) (state : State) : Prop
↓ Extract
OCaml: val search_spec : puzzle -> state -> unit
↓ Translate
Rust: pub fn search(puzzle: &Puzzle, state: &mut State) -> Option<Solution>
```

### Uniqueness Checking

```
Rocq: count_solutions (puzzle : Puzzle) (limit : nat) : nat
↓ Extract
OCaml: val count_solutions : puzzle -> int -> int
↓ Translate
Rust: pub fn count_solutions_up_to(puzzle: &Puzzle, limit: usize) -> Result<usize, SolveError>
```

## Extraction Configuration

The file `../rcoq/KenKen/Extraction.v` configures:

1. **Proof Erasure**: All `Prop` types map to `unit` (proofs discarded)
2. **Arithmetic Optimization**: Native OCaml operators for +, -, *, /
3. **List Handling**: Standard OCaml list operations
4. **Output Files**: Extraction generates `.ml` files in `extraction/` directory

## Translation Decisions

### 1. Natural Numbers → Machine Integers

**Decision**: Use `u32` by default, `u64` for large grids (n > 31)

**Rationale**:
- OCaml extracts `nat` to `int` (platform-dependent size)
- Rust requires explicit integer types
- u32 fits grids up to 31×31 (961 cells)
- u64 required for 32×32 and larger grids

**Bounds Checks**:
- Enforce 1 ≤ value ≤ n in all cell value operations
- Panic on overflow in debug builds
- Return Result::Err in production for OOB

### 2. List → Vec Conversion

**Decision**: Convert all OCaml lists to Rust Vec<T>

**Rationale**:
- Lists provide recursion, vectors provide indexing
- Solver heavily uses indexed access (cell lookups)
- Vec<u8> more efficient for small domain elements
- Minimal translation effort

**Example**:
```ocaml
let cells : int list = [0; 1; 2]  (* OCaml *)
```
```rust
let cells: Vec<u8> = vec![0, 1, 2];  // Rust
```

### 3. Record → Struct

**Decision**: Use Rust structs with identical field layout

**Rationale**:
- Direct mapping, zero translation overhead
- Preserve field names for clarity
- Enable Rust idioms (impl blocks, trait impls)

### 4. Recursion → Iteration (where possible)

**Decision**: Convert recursive search to iterative with explicit stack

**Rationale**:
- Stack overflow risk with deep recursion
- OCaml tail-call optimization not guaranteed in Rust
- Explicit control flow easier to profile/optimize

**Example**: MRV search becomes loop with Vec<SearchState>

## Verification Strategy

### Phase 1: Compilation Verification
- Extracted OCaml code compiles with `ocamlfind`
- Run OCaml extraction tests (extracted unit tests from Rocq)
- Verify basic property calculations on small puzzles

### Phase 2: Translation Verification
- Rust code compiles without warnings (`-D warnings`)
- Unit tests match extracted behavior (2x2, 3x3 puzzles)
- Solution verification matches kenken-solver implementation

### Phase 3: Cross-Validation
- Run on golden corpus (52 puzzles, all sizes 2-6)
- Compare Rocq-verified solutions with native solver
- Verify uniqueness checking for all difficulty tiers

### Phase 4: Performance Validation
- Benchmark extracted vs native implementation
- Measure overhead of proof erasure + translation
- Profile SIMD popcount integration

## Common Translation Pitfalls

1. **Nat Overflow**: OCaml `int` can overflow; Rust requires explicit bounds
   - Solution: Use `u32::try_from()` or clamp to valid range

2. **List Indexing**: OCaml allows safe negative indices; Rust panics
   - Solution: Use `get()` or `get_mut()` for bounds checking

3. **Recursion Depth**: Search may exceed stack in recursive form
   - Solution: Convert to iterative with explicit stack Vec

4. **Memory Layout**: Rocq records don't guarantee layout (unlike Rust)
   - Solution: Use `#[repr(C)]` if FFI needed; otherwise okay

## Next Steps

1. Generate OCaml extraction: `coqc -R rcoq KenKen rcoq/KenKen/Extraction.v`
2. Compile extracted code: `ocamlfind ocamlopt -package unix extraction/*.ml`
3. Review extracted types in `extraction/` directory
4. Begin manual Rust translation in `src/verified_solver.rs`
5. Update `../translation/audit_trail.md` as decisions are made

---

**Last Updated**: 2026-01-01
**Status**: In Progress
**Next Phase**: Extraction and OCaml validation
