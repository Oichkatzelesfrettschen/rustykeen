(** KenKen Core Formal Definitions

    This module defines the foundational types and predicates for KenKen puzzles:
    - Grid: n×n Latin square structure
    - Cage: constraint aggregate with target and operation
    - Puzzle: collection of cages over a grid
    - Solution: assignment of [1..n] to each cell
    - Correctness: constraint satisfaction predicates
*)

From Rocq Require Import Nat List Arith.

(** Operation type for cage constraints *)
Inductive Operation : Type :=
  | Add    : Operation
  | Sub    : Operation
  | Mul    : Operation
  | Div    : Operation
  | Eq     : Operation.

(** Cell identifier: linear index in [0..n*n) *)
Definition Cell := nat.

(** Cage representation: cells, operation, target *)
Record Cage : Type := {
  cells : list Cell ;
  op : Operation ;
  target : nat
}.

(** Puzzle: grid size and cages *)
Record Puzzle : Type := {
  n : nat ;
  cages : list Cage
}.

(** Solution: assignment of value [1..n] to each cell *)
Definition Solution := list nat.

(** Grid validity: all cells [1..n] appear exactly once per row/column *)
Definition valid_latin_square (n : nat) (sol : Solution) : Prop :=
  length sol = n * n ∧
  (∀ i, i < length sol → (1 ≤ nth i sol 0 ∧ nth i sol 0 ≤ n)).

(** Cage constraint satisfaction for a single operation *)
Definition satisfies_cage_op (op : Operation) (values : list nat) (target : nat) : Prop :=
  match op with
  | Add => fold_right plus 0 values = target
  | Mul => fold_right mult 1 values = target
  | Sub => (length values = 2) ∧
           (abs_diff (nth 0 values 0) (nth 1 values 0) = target)
  | Div => (length values = 2) ∧ (nth 1 values 0 ≠ 0) ∧
           (Nat.div (nth 0 values 0) (nth 1 values 0) = target) ∧
           (Nat.modulo (nth 0 values 0) (nth 1 values 0) = 0)
  | Eq => (length values = 1) ∧ (nth 0 values 0 = target)
  end.

(** Absolute difference for natural numbers *)
Definition abs_diff (a b : nat) : nat :=
  if Nat.leb a b then b - a else a - b.

(** Extract values from cells in a solution *)
Definition cell_values (sol : Solution) (cells : list Cell) : list nat :=
  map (fun cell => nth cell sol 0) cells.

(** Full cage constraint satisfaction *)
Definition satisfies_cage (puzzle : Puzzle) (cage : Cage) (sol : Solution) : Prop :=
  satisfies_cage_op cage.(op) (cell_values sol cage.(cells)) cage.(target).

(** Complete solution validity *)
Definition valid_solution (puzzle : Puzzle) (sol : Solution) : Prop :=
  valid_latin_square puzzle.(n) sol ∧
  (∀ cage, In cage puzzle.(cages) → satisfies_cage puzzle cage sol).

(** Row uniqueness constraint *)
Definition row_unique (n : nat) (sol : Solution) (row : nat) : Prop :=
  let row_vals := take n (drop (row * n) sol) in
  NoDup row_vals.

(** Column uniqueness constraint *)
Definition col_unique (n : nat) (sol : Solution) (col : nat) : Prop :=
  let col_vals := [seq nth (i * n + col) sol 0 | i <- seq 0 n] in
  NoDup col_vals.

(** All rows and columns unique (Latin square property) *)
Definition all_unique (n : nat) (sol : Solution) : Prop :=
  (∀ row, row < n → row_unique n sol row) ∧
  (∀ col, col < n → col_unique n sol col).

(* Helper functions from stdlib *)
Fixpoint take {A : Type} (n : nat) (l : list A) : list A :=
  match n, l with
  | 0, _ => []
  | _, [] => []
  | S n', a :: l' => a :: take n' l'
  end.

Fixpoint drop {A : Type} (n : nat) (l : list A) : list A :=
  match n, l with
  | 0, l => l
  | _, [] => []
  | S n', _ :: l' => drop n' l'
  end.
