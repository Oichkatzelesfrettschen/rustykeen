#![no_main]

//! Fuzz target for sgt-desc format parser.
//!
//! Tests that `parse_keen_desc` handles arbitrary input without panicking.
//! Valid parse results are further validated for consistency.

use libfuzzer_sys::fuzz_target;

use kenken_core::format::sgt_desc::{encode_keen_desc, parse_keen_desc};
use kenken_core::rules::Ruleset;

fuzz_target!(|data: &[u8]| {
    // Convert to string, skip if not valid UTF-8
    let Ok(input) = std::str::from_utf8(data) else {
        return;
    };

    // Test various grid sizes
    for n in 2u8..=9 {
        // Parser should not panic on any input
        let result = parse_keen_desc(n, input);

        // If parse succeeded, validate the puzzle structure
        if let Ok(puzzle) = result {
            let rules = Ruleset::keen_baseline();

            // Validation should not panic
            let _ = puzzle.validate(rules);

            // If valid, roundtrip should work
            if puzzle.validate(rules).is_ok() {
                if let Ok(encoded) = encode_keen_desc(&puzzle, rules) {
                    // Re-parse should succeed
                    let _ = parse_keen_desc(n, &encoded);
                }
            }
        }
    }
});
