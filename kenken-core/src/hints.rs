//! Branch prediction hints for performance optimization.
//!
//! Provides `likely!()` and `unlikely!()` macros that hint to the compiler about
//! expected branch outcomes, potentially improving performance through better
//! instruction scheduling and branch prediction.
//!
//! ## Usage
//!
//! ```rust
//! use kenken_core::hints::{likely, unlikely};
//!
//! if likely(x > 0) {
//!     // Hot path - expected to be taken most of the time
//! }
//!
//! if unlikely(error_condition) {
//!     // Cold path - error handling, rarely taken
//! }
//! ```
//!
//! ## Implementation Notes
//!
//! These functions use `core::intrinsics::likely` and `core::intrinsics::unlikely`
//! on nightly Rust, which map to LLVM's `llvm.expect` intrinsic. The hints may
//! improve performance by:
//! - Reducing branch misprediction penalties
//! - Improving instruction cache utilization
//! - Enabling better code layout
//!
//! On stable Rust or when intrinsics are unavailable, these are no-ops that
//! simply return the input boolean unchanged.

/// Hint that a boolean condition is likely to be true.
///
/// This provides a branch prediction hint to the compiler that the condition
/// is expected to be true in most cases. Use this for hot paths and common cases.
///
/// # Examples
///
/// ```rust
/// use kenken_core::hints::likely;
///
/// if likely(cache_hit) {
///     // Fast path - cache hit is expected
/// } else {
///     // Slow path - cache miss is rare
/// }
/// ```
#[inline(always)]
pub fn likely(b: bool) -> bool {
    #[cfg(all(target_arch = "x86_64", not(miri)))]
    {
        // Use core::hint::unlikely for the negation, which is stable
        // This is equivalent to likely(b) = !unlikely(!b)
        !core::hint::unlikely(!b)
    }
    #[cfg(not(all(target_arch = "x86_64", not(miri))))]
    {
        b
    }
}

/// Hint that a boolean condition is unlikely to be true.
///
/// This provides a branch prediction hint to the compiler that the condition
/// is expected to be false in most cases. Use this for error paths and rare cases.
///
/// # Examples
///
/// ```rust
/// use kenken_core::hints::unlikely;
///
/// if unlikely(out_of_memory) {
///     // Error path - rarely taken
///     panic!("Out of memory");
/// }
/// ```
#[inline(always)]
pub fn unlikely(b: bool) -> bool {
    #[cfg(all(target_arch = "x86_64", not(miri)))]
    {
        core::hint::unlikely(b)
    }
    #[cfg(not(all(target_arch = "x86_64", not(miri))))]
    {
        b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn likely_returns_true_when_true() {
        assert!(likely(true));
    }

    #[test]
    fn likely_returns_false_when_false() {
        assert!(!likely(false));
    }

    #[test]
    fn unlikely_returns_true_when_true() {
        assert!(unlikely(true));
    }

    #[test]
    fn unlikely_returns_false_when_false() {
        assert!(!unlikely(false));
    }

    #[test]
    fn likely_unlikely_are_inverses() {
        // likely(x) and unlikely(!x) should have same truthiness
        assert_eq!(likely(true), !unlikely(false));
        assert_eq!(likely(false), !unlikely(true));
    }
}
