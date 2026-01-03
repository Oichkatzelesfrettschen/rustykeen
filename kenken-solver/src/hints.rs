//! Branch prediction hints for hot paths
//!
//! Provides `likely()` and `unlikely()` functions for branch prediction hints.
//! These compile to LLVM intrinsics that guide the optimizer to favor likely branches.

/// Branch prediction hint: this condition is likely to be true
///
/// Uses cold path marking to hint the optimizer about branch probability.
/// This is an internalized version of the `likely_stable` crate.
#[inline(always)]
pub fn likely(b: bool) -> bool {
    if !b {
        cold();
    }
    b
}

/// Branch prediction hint: this condition is unlikely to be true
///
/// Uses cold path marking to hint the optimizer about branch probability.
#[allow(dead_code)]
#[inline(always)]
pub fn unlikely(b: bool) -> bool {
    if b {
        cold();
    }
    b
}

/// Marker function for cold (unlikely) branches
#[cold]
#[inline(always)]
fn cold() {}
