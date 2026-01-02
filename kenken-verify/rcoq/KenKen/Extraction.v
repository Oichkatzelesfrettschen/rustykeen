(** OCaml Extraction Configuration

    This module configures the extraction of Rocq definitions to OCaml.
    The extracted code serves as an intermediate step to pure Rust via
    careful manual translation with cross-validation.

    Extraction Mapping:
    - Rocq list → OCaml list
    - Rocq nat → OCaml int (with overflow checks)
    - Rocq Prop → OCaml unit (proofs erased)
    - Rocq recursion → OCaml recursion
*)

From Rocq Require Import Extraction.
Require Import KenKen.Core KenKen.Search KenKen.Uniqueness KenKen.Integration.

(* Extraction settings *)

(** Extract to OCaml with conservative settings *)
Extraction Language OCaml.

(** Blacklist: do not extract proofs *)
Extract Inductive Prop => unit [ "()" "()" ].

(** Map Rocq nat to OCaml int *)
(* Note: This requires careful bounds checking in extracted code *)
Extract Inductive nat => int
  [ "0" "(fun n -> n + 1)" ]
  "(fun zero succ n -> if n = 0 then zero () else succ (n - 1))".

(** Map Rocq bool to OCaml bool *)
Extract Inductive bool => bool [ "true" "false" ].

(** Map Rocq list to OCaml list *)
Extract Inductive list => list [ "[]" "(::)" ].

(** Optimize Rocq arithmetic *)
Extract Constant Nat.add => "( + )".
Extract Constant Nat.mul => "( * )".
Extract Constant Nat.sub => "( - )".
Extract Constant Nat.div => "( / )".
Extract Constant Nat.modulo => "( mod )".
Extract Constant Nat.ltb => "( < )".
Extract Constant Nat.leb => "( <= )".
Extract Constant Nat.eqb => "( = )".

(** Core types to extract *)
Extraction "extraction/kenken_core.ml"
  Puzzle Cage Cell Solution Operation
  valid_solution satisfies_cage valid_latin_square.

(** Search algorithm to extract *)
Extraction "extraction/kenken_search.ml"
  search_spec mrv_cell propagate Domain State.

(** Uniqueness verification to extract *)
Extraction "extraction/kenken_uniqueness.ml"
  count_solutions is_unique is_unsolvable.

(** Integration and verification to extract *)
Extraction "extraction/kenken_integration.ml"
  verify_solution_combined z3_verify sat_verify
  verify_via_z3 verify_via_sat verify_native.

(** Comments for extracted code *)
(* The extraction produces OCaml code with:
   1. All proofs erased (Prop → unit)
   2. Natural numbers mapped to int (requires bounds checking in Rust translation)
   3. Recursive functions preserved (MRV search, counting)
   4. List operations via OCaml std library

   Next steps after extraction:
   1. Verify extracted OCaml compiles and passes basic tests
   2. Manually translate OCaml to Rust with:
      - Type conversions (int → u32/u64)
      - Iterator/loop conversions
      - Error handling for bounds
   3. Run cross-validation: Rust implementation vs original solver
   4. Document translation decisions in ../translation/audit_trail.md
*)
