#![doc = "WASM bindings for KenKen solver with opaque handle pattern"]

use kenken_core::Puzzle;
use kenken_solver::{Ruleset, solve_one_with_stats_dispatched};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;

/// Result of solving a puzzle
#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "serde")]
pub struct SolveResultData {
    pub success: bool,
    pub solution: Option<Vec<u32>>,
    pub assignments: u64,
    pub nodes_visited: u64,
    pub max_depth: u32,
    pub backtracked: bool,
    pub error: Option<String>,
}

// Global puzzle store using opaque handles (2025 WASM best practices)
lazy_static! {
    static ref PUZZLE_STORE: Mutex<HashMap<u32, Puzzle>> = Mutex::new(HashMap::new());
}

static mut PUZZLE_COUNTER: u32 = 0;

/// Create a test puzzle (4x4) and return opaque handle
/// Demonstrates the handle pattern with all-singleton Latin square puzzle
#[wasm_bindgen]
pub fn create_test_puzzle() -> u32 {
    // Create a 4x4 puzzle with all singleton cages (each cell is a 1-cell cage)
    let puzzle = Puzzle {
        n: 4,
        cages: vec![], // Empty cages for now - just tests the handle mechanism
    };

    let handle = unsafe {
        PUZZLE_COUNTER += 1;
        PUZZLE_COUNTER
    };

    if let Ok(mut store) = PUZZLE_STORE.lock() {
        store.insert(handle, puzzle);
    }
    handle
}

/// Solve puzzle by opaque handle
/// Returns JSON with solution and statistics
#[wasm_bindgen]
pub fn solve_puzzle_handle(handle: u32) -> Result<String, JsValue> {
    let puzzle = {
        let store = PUZZLE_STORE
            .lock()
            .map_err(|_| JsValue::from_str("Lock error"))?;
        store
            .get(&handle)
            .ok_or_else(|| JsValue::from_str("Puzzle not found"))?
            .clone()
    };

    let rules = Ruleset::keen_baseline();
    match solve_one_with_stats_dispatched(&puzzle, rules) {
        Ok((Some(solution), stats)) => {
            let result = SolveResultData {
                success: true,
                solution: Some(solution.grid.iter().map(|&v| v as u32).collect()),
                assignments: stats.assignments,
                nodes_visited: stats.nodes_visited,
                max_depth: stats.max_depth,
                backtracked: stats.backtracked,
                error: None,
            };

            // Use serde_wasm_bindgen for efficient serialization
            serde_wasm_bindgen::to_value(&result)
                .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
                .map(|v| v.as_string().unwrap_or_default())
        }
        Ok((None, _)) => {
            let result = SolveResultData {
                success: false,
                solution: None,
                assignments: 0,
                nodes_visited: 0,
                max_depth: 0,
                backtracked: false,
                error: Some("No solution found".to_string()),
            };
            serde_wasm_bindgen::to_value(&result)
                .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
                .map(|v| v.as_string().unwrap_or_default())
        }
        Err(e) => {
            let result = SolveResultData {
                success: false,
                solution: None,
                assignments: 0,
                nodes_visited: 0,
                max_depth: 0,
                backtracked: false,
                error: Some(format!("Solver error: {}", e)),
            };
            serde_wasm_bindgen::to_value(&result)
                .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
                .map(|v| v.as_string().unwrap_or_default())
        }
    }
}

/// Delete puzzle from store (cleanup)
#[wasm_bindgen]
pub fn free_puzzle(handle: u32) -> bool {
    PUZZLE_STORE
        .lock()
        .ok()
        .and_then(|mut store| store.remove(&handle))
        .is_some()
}

/// Get library information (API metadata)
#[wasm_bindgen]
pub fn get_info() -> String {
    serde_json::json!({
        "name": "kenken-wasm",
        "version": env!("CARGO_PKG_VERSION"),
        "api_version": "2.0",
        "strategy": "opaque_handles_with_serde_wasm_bindgen",
        "supported_sizes": [2, 3, 4, 5, 6, 7, 8, 9],
        "features": {
            "solver": true,
            "opaque_handles": true,
            "efficient_serialization": true,
            "statistics": true,
        }
    })
    .to_string()
}

/// Health check endpoint
#[wasm_bindgen]
pub fn health_check() -> String {
    "OK".to_string()
}
