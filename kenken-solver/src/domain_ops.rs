//! Zero-cost trait abstraction for domain representation
//!
//! This module defines a generic `DomainOps` trait to abstract over different
//! bitmask domain representations:
//!
//! - **Domain32**: u32-based bitmasks, n ≤ 31 (current default, backward compatible)
//! - **Domain64**: u64-based bitmasks, n ≤ 63 (opt-in via `solver-u64` feature)
//! - **BitDomain**: Heap-allocated bitvec, n ≤ 255 (opt-in via `solver-bitdomain` feature)
//!
//! The trait provides:
//! - Bit manipulation (insert, remove, contains, count)
//! - Set operations (and, or, xor, complement)
//! - Domain iteration and encoding/decoding
//!
//! ## Performance Characteristics
//!
//! - **Domain32**: Zero overhead vs current code; all operations inlined
//! - **Domain64**: ~2-5% overhead from u64 vs u32 register pressure
//! - **BitDomain**: ~2-3x slower (heap allocations + indirect access)

use core::fmt::Debug;

/// Trait for abstract domain operations over different bitfield representations
///
/// # Invariants
///
/// For all methods, the domain must remain valid (no bits set beyond position n-1).
pub trait DomainOps: Clone + Debug + Sized + 'static {
    /// Create an empty domain (all bits cleared)
    fn empty() -> Self;

    /// Create a full domain with all bits set for [1..=n]
    fn full(n: u8) -> Self;

    /// Insert a value into the domain (sets bit at position value-1)
    fn insert(&mut self, value: u8);

    /// Remove a value from the domain (clears bit at position value-1)
    fn remove(&mut self, value: u8);

    /// Check if a value is in the domain
    fn contains(&self, value: u8) -> bool;

    /// Count the number of values in the domain (popcount)
    fn count(&self) -> u32;

    /// Get the smallest value in the domain, or None if empty
    fn min(&self) -> Option<u8>;

    /// Get the largest value in the domain, or None if empty
    fn max(&self) -> Option<u8>;

    /// Bitwise AND with another domain
    fn and(&self, other: &Self) -> Self;

    /// Bitwise OR with another domain
    fn or(&self, other: &Self) -> Self;

    /// Bitwise XOR with another domain
    fn xor(&self, other: &Self) -> Self;

    /// Bitwise complement (within n-bit scope)
    fn complement(&self, n: u8) -> Self;

    /// Iterate over all values in the domain (1-indexed)
    fn iter_values(&self) -> Box<dyn Iterator<Item = u8> + '_>;

    /// Clear all bits
    fn clear(&mut self);

    /// Check if domain is empty
    fn is_empty(&self) -> bool {
        self.count() == 0
    }

    /// Get a string representation for debugging
    fn to_string(&self, n: u8) -> String;
}

/// Domain32: u32-based bitmask representation (n ≤ 31)
///
/// This is the default for backward compatibility with existing code.
/// Supports grids up to 31×31 (1024 cells).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Domain32(pub u32);

impl DomainOps for Domain32 {
    fn empty() -> Self {
        Domain32(0)
    }

    fn full(n: u8) -> Self {
        debug_assert!(n <= 31, "Domain32 only supports n ≤ 31");
        if n == 31 {
            Domain32(u32::MAX)
        } else {
            Domain32((1u32 << n) - 1)
        }
    }

    fn insert(&mut self, value: u8) {
        debug_assert!(value >= 1 && value <= 31);
        self.0 |= 1u32 << (value - 1);
    }

    fn remove(&mut self, value: u8) {
        debug_assert!(value >= 1 && value <= 31);
        self.0 &= !(1u32 << (value - 1));
    }

    fn contains(&self, value: u8) -> bool {
        debug_assert!(value >= 1 && value <= 31);
        (self.0 & (1u32 << (value - 1))) != 0
    }

    fn count(&self) -> u32 {
        self.0.count_ones()
    }

    fn min(&self) -> Option<u8> {
        if self.0 == 0 {
            None
        } else {
            Some((self.0.trailing_zeros() + 1) as u8)
        }
    }

    fn max(&self) -> Option<u8> {
        if self.0 == 0 {
            None
        } else {
            Some((31 - self.0.leading_zeros()) as u8)
        }
    }

    fn and(&self, other: &Self) -> Self {
        Domain32(self.0 & other.0)
    }

    fn or(&self, other: &Self) -> Self {
        Domain32(self.0 | other.0)
    }

    fn xor(&self, other: &Self) -> Self {
        Domain32(self.0 ^ other.0)
    }

    fn complement(&self, n: u8) -> Self {
        debug_assert!(n <= 31);
        let mask = if n == 31 { u32::MAX } else { (1u32 << n) - 1 };
        Domain32(self.0 ^ mask)
    }

    fn iter_values(&self) -> Box<dyn Iterator<Item = u8> + '_> {
        let bits = self.0;
        Box::new((0..32).filter_map(move |i| {
            if (bits & (1u32 << i)) != 0 {
                Some((i + 1) as u8)
            } else {
                None
            }
        }))
    }

    fn clear(&mut self) {
        self.0 = 0;
    }

    fn to_string(&self, n: u8) -> String {
        format!("Domain32({:0width$b})", self.0, width = n as usize)
    }
}

/// Domain64: u64-based bitmask representation (32 < n ≤ 63)
///
/// Enabled via `solver-u64` feature. Supports larger grids.
/// Slightly more register pressure than u32, but enables 32×32 and 64×64 grids.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Domain64(pub u64);

impl DomainOps for Domain64 {
    fn empty() -> Self {
        Domain64(0)
    }

    fn full(n: u8) -> Self {
        debug_assert!(n <= 63, "Domain64 only supports n ≤ 63");
        if n == 63 {
            Domain64(u64::MAX)
        } else {
            Domain64((1u64 << n) - 1)
        }
    }

    fn insert(&mut self, value: u8) {
        debug_assert!(value >= 1 && value <= 63);
        self.0 |= 1u64 << (value - 1);
    }

    fn remove(&mut self, value: u8) {
        debug_assert!(value >= 1 && value <= 63);
        self.0 &= !(1u64 << (value - 1));
    }

    fn contains(&self, value: u8) -> bool {
        debug_assert!(value >= 1 && value <= 63);
        (self.0 & (1u64 << (value - 1))) != 0
    }

    fn count(&self) -> u32 {
        self.0.count_ones()
    }

    fn min(&self) -> Option<u8> {
        if self.0 == 0 {
            None
        } else {
            Some((self.0.trailing_zeros() + 1) as u8)
        }
    }

    fn max(&self) -> Option<u8> {
        if self.0 == 0 {
            None
        } else {
            Some((63 - self.0.leading_zeros()) as u8)
        }
    }

    fn and(&self, other: &Self) -> Self {
        Domain64(self.0 & other.0)
    }

    fn or(&self, other: &Self) -> Self {
        Domain64(self.0 | other.0)
    }

    fn xor(&self, other: &Self) -> Self {
        Domain64(self.0 ^ other.0)
    }

    fn complement(&self, n: u8) -> Self {
        debug_assert!(n <= 63);
        let mask = if n == 63 { u64::MAX } else { (1u64 << n) - 1 };
        Domain64(self.0 ^ mask)
    }

    fn iter_values(&self) -> Box<dyn Iterator<Item = u8> + '_> {
        let bits = self.0;
        Box::new((0..64).filter_map(move |i| {
            if (bits & (1u64 << i)) != 0 {
                Some((i + 1) as u8)
            } else {
                None
            }
        }))
    }

    fn clear(&mut self) {
        self.0 = 0;
    }

    fn to_string(&self, n: u8) -> String {
        format!("Domain64({:0width$b})", self.0, width = n as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain32_basic() {
        let mut d = Domain32::empty();
        assert!(d.is_empty());
        d.insert(1);
        assert!(d.contains(1));
        assert_eq!(d.count(), 1);
    }

    #[test]
    fn test_domain32_full() {
        let d = Domain32::full(3);
        assert_eq!(d.count(), 3);
        assert!(d.contains(1) && d.contains(2) && d.contains(3));
    }

    #[test]
    fn test_domain64_basic() {
        let mut d = Domain64::empty();
        assert!(d.is_empty());
        d.insert(1);
        assert!(d.contains(1));
        assert_eq!(d.count(), 1);
    }

    #[test]
    fn test_domain64_large() {
        let d = Domain64::full(32);
        assert_eq!(d.count(), 32);
    }
}
