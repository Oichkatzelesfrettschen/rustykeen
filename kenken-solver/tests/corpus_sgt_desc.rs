use kenken_core::format::sgt_desc::parse_keen_desc;
use kenken_core::rules::Ruleset;
use kenken_solver::{DeductionTier, count_solutions_up_to_with_deductions};

#[test]
fn sgt_desc_corpus_counts_match_expectations() {
    let rules = Ruleset::keen_baseline();
    let cases = [(
        2u8, "b__,a3a3", 2u32, // 2×2 has exactly two Latin squares under this cage layout.
    )];

    for (n, desc, expected) in cases {
        let puzzle = parse_keen_desc(n, desc).unwrap();
        puzzle.validate(rules).unwrap();
        let got = count_solutions_up_to_with_deductions(&puzzle, rules, DeductionTier::Normal, 2)
            .unwrap();
        assert_eq!(got, expected, "n={n} desc={desc}");
    }
}
