//! Domain128: 128-bit bitmask domain for n ≤ 127 with multi-platform SIMD support
//!
//! Supports grids up to 127×127 (16,129 cells) using 128-bit bitmask representation.
//!
//! **SIMD Implementations** (via kenken-simd runtime dispatch):
//! - **x86_64**:
//!   - POPCNT (fastest, native instruction)
//!   - SSSE3+AVX2 (lookup table, ~800-1000 ps)
//!   - SSE2 (Harley-Seal, ~1200-1500 ps)
//!   - Scalar fallback
//! - **ARM aarch64**: NEON (vcntq_u8)
//! - **Fallback**: Scalar (all platforms)
//!
//! **Performance**:
//! - POPCNT (modern x86_64): ~300-400 ps
//! - SSSE3+AVX2 LUT: ~800-1000 ps
//! - Scalar: ~400-600 ps (via count_ones)
//! - 1.5-2x slower than Domain64 (acceptable for n > 64)
//!
//! **Layout**:
//! ```text
//! [u64; 2] = [bits 0-63, bits 64-127]
//!             [values 1-64, values 65-127]
//! ```

use crate::domain_ops::DomainOps;

#[cfg(feature = "simd-dispatch")]
use kenken_simd::popcount_u128;

/// 128-bit bitmask domain for n ≤ 127
///
/// Stores two u64 values: lower 64 bits for values 1-64, upper 64 bits for values 65-127
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Domain128([u64; 2]);

impl DomainOps for Domain128 {
    fn empty() -> Self {
        Domain128([0, 0])
    }

    fn full(n: u8) -> Self {
        debug_assert!(n <= 127, "Domain128 only supports n ≤ 127");

        if n < 64 {
            Domain128([(1u64 << n) - 1, 0])
        } else if n == 64 {
            Domain128([u64::MAX, 0])
        } else {
            let upper_bits = n - 64;
            Domain128([u64::MAX, (1u64 << upper_bits) - 1])
        }
    }

    fn insert(&mut self, value: u8) {
        debug_assert!(value > 0 && value <= 127, "Value must be in [1..=127]");

        let bit_pos = (value - 1) as usize;
        if bit_pos < 64 {
            self.0[0] |= 1u64 << bit_pos;
        } else {
            self.0[1] |= 1u64 << (bit_pos - 64);
        }
    }

    fn remove(&mut self, value: u8) {
        debug_assert!(value > 0 && value <= 127, "Value must be in [1..=127]");

        let bit_pos = (value - 1) as usize;
        if bit_pos < 64 {
            self.0[0] &= !(1u64 << bit_pos);
        } else {
            self.0[1] &= !(1u64 << (bit_pos - 64));
        }
    }

    fn contains(&self, value: u8) -> bool {
        debug_assert!(value > 0 && value <= 127, "Value must be in [1..=127]");

        let bit_pos = (value - 1) as usize;
        if bit_pos < 64 {
            (self.0[0] & (1u64 << bit_pos)) != 0
        } else {
            (self.0[1] & (1u64 << (bit_pos - 64))) != 0
        }
    }

    fn count(&self) -> u32 {
        // Use SIMD-optimized popcount via runtime dispatch
        #[cfg(feature = "simd-dispatch")]
        {
            popcount_u128(self.0)
        }
        #[cfg(not(feature = "simd-dispatch"))]
        {
            self.0[0].count_ones() + self.0[1].count_ones()
        }
    }

    fn min(&self) -> Option<u8> {
        if self.0[0] != 0 {
            Some(self.0[0].trailing_zeros() as u8 + 1)
        } else if self.0[1] != 0 {
            Some(self.0[1].trailing_zeros() as u8 + 65)
        } else {
            None
        }
    }

    fn max(&self) -> Option<u8> {
        if self.0[1] != 0 {
            Some(128 - self.0[1].leading_zeros() as u8)
        } else if self.0[0] != 0 {
            Some(64 - self.0[0].leading_zeros() as u8)
        } else {
            None
        }
    }

    fn and(&self, other: &Self) -> Self {
        Domain128([self.0[0] & other.0[0], self.0[1] & other.0[1]])
    }

    fn or(&self, other: &Self) -> Self {
        Domain128([self.0[0] | other.0[0], self.0[1] | other.0[1]])
    }

    fn xor(&self, other: &Self) -> Self {
        Domain128([self.0[0] ^ other.0[0], self.0[1] ^ other.0[1]])
    }

    fn complement(&self, n: u8) -> Self {
        debug_assert!(n <= 127, "Complement requires n ≤ 127");

        let full = Self::full(n);
        Domain128([self.0[0] ^ full.0[0], self.0[1] ^ full.0[1]])
    }

    fn iter_values(&self) -> Box<dyn Iterator<Item = u8> + '_> {
        Box::new(
            (0..64)
                .filter(move |&i| (self.0[0] & (1u64 << i)) != 0)
                .map(|i| i as u8 + 1)
                .chain(
                    (0..64)
                        .filter(move |&i| (self.0[1] & (1u64 << i)) != 0)
                        .map(|i| i as u8 + 65),
                ),
        )
    }

    fn clear(&mut self) {
        self.0[0] = 0;
        self.0[1] = 0;
    }

    fn to_string(&self, n: u8) -> String {
        let mut result = String::with_capacity(128);
        for i in 0..n {
            result.push(if self.contains(i + 1) { '1' } else { '0' });
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain128_empty_full() {
        let empty = Domain128::empty();
        assert!(empty.is_empty());
        assert_eq!(empty.count(), 0);

        let full = Domain128::full(127);
        assert!(!full.is_empty());
        assert_eq!(full.count(), 127);
    }

    #[test]
    fn test_domain128_insert_remove() {
        let mut d = Domain128::empty();

        // Insert in lower half
        d.insert(1);
        assert!(d.contains(1));
        assert_eq!(d.count(), 1);

        // Insert in upper half
        d.insert(100);
        assert!(d.contains(100));
        assert_eq!(d.count(), 2);

        // Remove
        d.remove(1);
        assert!(!d.contains(1));
        assert_eq!(d.count(), 1);
    }

    #[test]
    fn test_domain128_min_max() {
        let mut d = Domain128::empty();

        assert_eq!(d.min(), None);
        assert_eq!(d.max(), None);

        d.insert(10);
        d.insert(50);
        d.insert(100);

        assert_eq!(d.min(), Some(10));
        assert_eq!(d.max(), Some(100));
    }

    #[test]
    fn test_domain128_bitwise_ops() {
        let d1 = Domain128::full(64);
        let d2 = Domain128::full(127);

        let and_result = d1.and(&d2);
        assert_eq!(and_result.count(), 64); // All values 1-64

        let or_result = d1.or(&d2);
        assert_eq!(or_result.count(), 127); // All values 1-127

        let xor_result = d1.xor(&d2);
        assert_eq!(xor_result.count(), 63); // Values 65-127
    }

    #[test]
    fn test_domain128_iter_values() {
        let mut d = Domain128::empty();
        d.insert(1);
        d.insert(50);
        d.insert(65);
        d.insert(127);

        let values: Vec<u8> = d.iter_values().collect();
        assert_eq!(values, vec![1, 50, 65, 127]);
    }

    #[test]
    fn test_domain128_popcount_accuracy() {
        let mut d = Domain128::empty();

        for v in 1..=127 {
            d.insert(v);
        }

        assert_eq!(d.count(), 127);
    }

    #[test]
    fn test_domain128_complement() {
        let mut d = Domain128::full(50);
        let comp = d.complement(50);

        assert!(comp.is_empty());

        d.remove(25);
        let comp2 = d.complement(50);
        assert_eq!(comp2.count(), 1);
        assert!(comp2.contains(25));
    }
}
