//! SIMD-optimized fixedbitset domain representation
//!
//! Wraps the `fixedbitset` crate's FixedBitSet for CSP/SAT solver optimization.
//! FixedBitSet uses SSE2/AVX/AVX2 SIMD operations for batch bit operations,
//! making it faster than Domain32/Domain64 for large grids (n > 16).
//!
//! Enabled via `solver-fixedbitset` feature.

use crate::domain_ops::DomainOps;
use core::fmt::Debug;
use fixedbitset::FixedBitSet;

/// SIMD-optimized domain representation using fixedbitset
///
/// Heap-allocated bitset with SIMD batch operations.
/// Excellent for large grids (n > 16) where SIMD benefits outweigh allocation overhead.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixedBitDomain {
    bits: FixedBitSet,
    n: u8,
}

impl DomainOps for FixedBitDomain {
    fn empty() -> Self {
        Self {
            bits: FixedBitSet::with_capacity(64),
            n: 64,
        }
    }

    fn full(n: u8) -> Self {
        let mut bits = FixedBitSet::with_capacity(n as usize);
        // Set all bits from 0 to n-1 (representing values 1..=n)
        bits.insert_range(0..n as usize);
        Self { bits, n }
    }

    fn insert(&mut self, value: u8) {
        debug_assert!(value >= 1 && (value as usize) <= self.n as usize);
        self.bits.insert((value - 1) as usize);
    }

    fn remove(&mut self, value: u8) {
        debug_assert!(value >= 1 && (value as usize) <= self.n as usize);
        self.bits.set((value - 1) as usize, false);
    }

    fn contains(&self, value: u8) -> bool {
        debug_assert!(value >= 1 && (value as usize) <= self.n as usize);
        self.bits.contains((value - 1) as usize)
    }

    fn count(&self) -> u32 {
        self.bits.count_ones(..) as u32
    }

    fn min(&self) -> Option<u8> {
        self.bits.ones().next().map(|i| (i + 1) as u8)
    }

    fn max(&self) -> Option<u8> {
        self.bits.ones().next_back().map(|i| (i + 1) as u8)
    }

    fn and(&self, other: &Self) -> Self {
        let mut result = self.clone();
        result.bits.intersect_with(&other.bits);
        result
    }

    fn or(&self, other: &Self) -> Self {
        let mut result = self.clone();
        result.bits.union_with(&other.bits);
        result
    }

    fn xor(&self, other: &Self) -> Self {
        let mut result = self.clone();
        result.bits.symmetric_difference_with(&other.bits);
        result
    }

    fn complement(&self, n: u8) -> Self {
        let mut result = Self::full(n);
        result.bits.difference_with(&self.bits);
        result
    }

    fn iter_values(&self) -> Box<dyn Iterator<Item = u8> + '_> {
        Box::new(self.bits.ones().map(|i| (i + 1) as u8))
    }

    fn clear(&mut self) {
        self.bits.clear();
    }

    fn to_string(&self, n: u8) -> String {
        let mut bits_str = String::new();
        for i in (0..n as usize).rev() {
            bits_str.push(if self.bits.contains(i) { '1' } else { '0' });
        }
        format!("FixedBitDomain({})", bits_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixedbitset_basic() {
        let mut d = FixedBitDomain::empty();
        d.n = 3;
        d.insert(1);
        assert!(d.contains(1));
        assert_eq!(d.count(), 1);
    }

    #[test]
    fn test_fixedbitset_full() {
        let d = FixedBitDomain::full(3);
        assert_eq!(d.count(), 3);
        assert!(d.contains(1) && d.contains(2) && d.contains(3));
    }

    #[test]
    fn test_fixedbitset_min_max() {
        let d = FixedBitDomain::full(5);
        assert_eq!(d.min(), Some(1));
        assert_eq!(d.max(), Some(5));
    }

    #[test]
    fn test_fixedbitset_and() {
        let d1 = FixedBitDomain::full(4);
        let mut d2 = FixedBitDomain::empty();
        d2.n = 4;
        d2.insert(2);
        d2.insert(3);
        let result = d1.and(&d2);
        assert_eq!(result.count(), 2);
        assert!(result.contains(2) && result.contains(3));
    }

    #[test]
    fn test_fixedbitset_complement() {
        let mut d = FixedBitDomain::empty();
        d.n = 3;
        d.insert(1);
        let comp = d.complement(3);
        assert!(comp.contains(2));
        assert!(comp.contains(3));
        assert!(!comp.contains(1));
    }
}
