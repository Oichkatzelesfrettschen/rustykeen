const CONTRACT_DOC: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../docs/cross_target_api_parity_contract.md"
));
const UNIFFI_UDL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../kenken-uniffi/src/keen.udl"
));
const UNIFFI_LIB: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../kenken-uniffi/src/lib.rs"
));
const WASM_LIB: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs"));

fn assert_contains(haystack: &str, needle: &str, subject: &str) {
    assert!(
        haystack.contains(needle),
        "expected `{subject}` to contain `{needle}`"
    );
}

#[test]
fn contract_doc_covers_core_operations() {
    assert_contains(CONTRACT_DOC, "Required parity operations", "contract doc");
    assert_contains(
        CONTRACT_DOC,
        "Accepted intentional divergences",
        "contract doc",
    );
    assert_contains(CONTRACT_DOC, "Error-shape expectations", "contract doc");

    for operation in [
        "solve_sgt_desc",
        "count_solutions_sgt_desc",
        "generate_sgt_desc",
    ] {
        assert_contains(CONTRACT_DOC, operation, "contract doc");
    }
}

#[test]
fn uniffi_exports_core_operation_family() {
    for signature in [
        "Grid? solve_sgt_desc(u8 n, string desc, DeductionTier tier);",
        "u32 count_solutions_sgt_desc(u8 n, string desc, DeductionTier tier, u32 limit);",
        "Generated? generate_sgt_desc(u8 n, u64 seed, DeductionTier tier);",
    ] {
        assert_contains(UNIFFI_UDL, signature, "kenken-uniffi/src/keen.udl");
    }
}

#[test]
fn wasm_surface_and_documented_divergences_stay_aligned() {
    assert_contains(
        WASM_LIB,
        "pub fn solve_puzzle_handle(handle: u32) -> Result<String, JsValue>",
        "kenken-wasm/src/lib.rs",
    );
    assert!(
        !WASM_LIB.contains("pub fn count_solutions_sgt_desc("),
        "kenken-wasm now exposes count_solutions_sgt_desc; update contract and parity tests"
    );
    assert!(
        !WASM_LIB.contains("pub fn generate_sgt_desc("),
        "kenken-wasm now exposes generate_sgt_desc; update contract and parity tests"
    );

    assert_contains(
        CONTRACT_DOC,
        "`count_solutions_sgt_desc` is not currently exported by `kenken-wasm`.",
        "contract doc",
    );
    assert_contains(
        CONTRACT_DOC,
        "`generate_sgt_desc` is not currently exported by `kenken-wasm`.",
        "contract doc",
    );
}

#[test]
fn error_shape_contract_matches_source_shapes() {
    assert_contains(
        UNIFFI_LIB,
        "pub fn solve_sgt_desc(n: u8, desc: String, tier: DeductionTier) -> Option<Grid>",
        "kenken-uniffi/src/lib.rs",
    );
    assert_contains(
        UNIFFI_LIB,
        "pub fn generate_sgt_desc(n: u8, seed: u64, tier: DeductionTier) -> Option<Generated>",
        "kenken-uniffi/src/lib.rs",
    );
    assert_contains(
        UNIFFI_LIB,
        "pub fn count_solutions_sgt_desc(n: u8, desc: String, tier: DeductionTier, limit: u32) -> u32",
        "kenken-uniffi/src/lib.rs",
    );

    assert_contains(
        WASM_LIB,
        "pub struct SolveResultData",
        "kenken-wasm/src/lib.rs",
    );
    for field in [
        "pub success: bool",
        "pub solution: Option<Vec<u32>>",
        "pub assignments: u64",
        "pub nodes_visited: u64",
        "pub max_depth: u32",
        "pub backtracked: bool",
        "pub error: Option<String>",
    ] {
        assert_contains(WASM_LIB, field, "kenken-wasm/src/lib.rs");
    }

    assert_contains(CONTRACT_DOC, "Option<Grid>", "contract doc");
    assert_contains(CONTRACT_DOC, "Option<Generated>", "contract doc");
    assert_contains(CONTRACT_DOC, "Result<String, JsValue>", "contract doc");
    assert_contains(CONTRACT_DOC, "SolveResultData", "contract doc");
}
