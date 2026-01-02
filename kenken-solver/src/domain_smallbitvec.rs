//! Inline-optimized smallbitvec domain representation
//!
//! Uses `smallbitvec` crate for grids with n ≤ 8, where inline storage avoids allocations.
//! For small KenKen grids (2x2 through 8x8), this eliminates heap allocation overhead.
//!
//! Enabled via `solver-smallbitvec` feature.

use crate::domain_ops::DomainOps;
use core::fmt::Debug;
use smallbitvec::SmallBitVec;

/// Inline-optimized domain representation using smallbitvec
///
/// Uses stack-allocated inline storage for small bitvectors (n ≤ 8),
/// eliminating heap allocation overhead for small grids.
/// Falls back to heap storage for larger grids if needed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SmallBitDomain {
    bits: SmallBitVec,
    n: u8,
}

impl DomainOps for SmallBitDomain {
    fn empty() -> Self {
        Self {
            bits: SmallBitVec::new(),
            n: 8,
        }
    }

    fn full(n: u8) -> Self {
        let mut bits = SmallBitVec::new();
        // Set all bits from 0 to n-1 (representing values 1..=n)
        for _ in 0..n {
            bits.push(true);
        }
        Self { bits, n }
    }

    fn insert(&mut self, value: u8) {
        debug_assert!(value >= 1 && (value as usize) <= self.n as usize);
        let idx = (value - 1) as usize;
        if idx >= self.bits.len() {
            self.bits.resize(idx + 1, false);
        }
        self.bits.set(idx, true);
    }

    fn remove(&mut self, value: u8) {
        debug_assert!(value >= 1 && (value as usize) <= self.n as usize);
        let idx = (value - 1) as usize;
        if idx < self.bits.len() {
            self.bits.set(idx, false);
        }
    }

    fn contains(&self, value: u8) -> bool {
        debug_assert!(value >= 1 && (value as usize) <= self.n as usize);
        let idx = (value - 1) as usize;
        idx < self.bits.len() && self.bits[idx]
    }

    fn count(&self) -> u32 {
        self.bits.iter().filter(|&b| b).count() as u32
    }

    fn min(&self) -> Option<u8> {
        self.bits
            .iter()
            .position(|b| b)
            .map(|i| (i + 1) as u8)
    }

    fn max(&self) -> Option<u8> {
        self.bits
            .iter()
            .rposition(|b| b)
            .map(|i| (i + 1) as u8)
    }

    fn and(&self, other: &Self) -> Self {
        let mut result = self.clone();
        // Clear bits beyond other's length
        while result.bits.len() > other.bits.len() {
            result.bits.pop();
        }
        // AND with other's bits
        for (i, bit) in other.bits.iter().enumerate() {
            if i < result.bits.len() {
                let new_bit = result.bits[i] && bit;
                result.bits.set(i, new_bit);
            }
        }
        result
    }

    fn or(&self, other: &Self) -> Self {
        let mut result = self.clone();
        for (i, bit) in other.bits.iter().enumerate() {
            if i >= result.bits.len() {
                result.bits.push(bit);
            } else {
                let new_bit = result.bits[i] || bit;
                result.bits.set(i, new_bit);
            }
        }
        result
    }

    fn xor(&self, other: &Self) -> Self {
        let mut result = self.clone();
        for (i, bit) in other.bits.iter().enumerate() {
            if i >= result.bits.len() {
                result.bits.push(bit);
            } else {
                let new_bit = result.bits[i] != bit;
                result.bits.set(i, new_bit);
            }
        }
        result
    }

    fn complement(&self, n: u8) -> Self {
        let mut result = Self::empty();
        result.n = n;
        for i in 0..n as usize {
            result.bits.push(i >= self.bits.len() || !self.bits[i]);
        }
        result
    }

    fn iter_values(&self) -> Box<dyn Iterator<Item = u8> + '_> {
        Box::new(
            self.bits
                .iter()
                .enumerate()
                .filter_map(|(i, b)| if b { Some((i + 1) as u8) } else { None }),
        )
    }

    fn clear(&mut self) {
        self.bits.clear();
    }

    fn to_string(&self, n: u8) -> String {
        let mut bits_str = String::new();
        for i in (0..n as usize).rev() {
            bits_str.push(if i < self.bits.len() && self.bits[i] {
                '1'
            } else {
                '0'
            });
        }
        format!("SmallBitDomain({})", bits_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smallbitvec_basic() {
        let mut d = SmallBitDomain::empty();
        d.n = 3;
        d.insert(1);
        assert!(d.contains(1));
        assert_eq!(d.count(), 1);
    }

    #[test]
    fn test_smallbitvec_full() {
        let d = SmallBitDomain::full(3);
        assert_eq!(d.count(), 3);
        assert!(d.contains(1) && d.contains(2) && d.contains(3));
    }

    #[test]
    fn test_smallbitvec_min_max() {
        let d = SmallBitDomain::full(5);
        assert_eq!(d.min(), Some(1));
        assert_eq!(d.max(), Some(5));
    }

    #[test]
    fn test_smallbitvec_and() {
        let d1 = SmallBitDomain::full(4);
        let mut d2 = SmallBitDomain::empty();
        d2.n = 4;
        d2.insert(2);
        d2.insert(3);
        let result = d1.and(&d2);
        assert_eq!(result.count(), 2);
        assert!(result.contains(2) && result.contains(3));
    }

    #[test]
    fn test_smallbitvec_complement() {
        let mut d = SmallBitDomain::empty();
        d.n = 3;
        d.insert(1);
        let comp = d.complement(3);
        assert!(comp.contains(2));
        assert!(comp.contains(3));
        assert!(!comp.contains(1));
    }
}
