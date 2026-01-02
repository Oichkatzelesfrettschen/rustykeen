//! Domain256: 256-bit bitmask domain for n ≤ 255 with multi-platform SIMD support
//!
//! Supports grids up to 255×255 (65,025 cells) using 256-bit bitmask representation.
//!
//! **SIMD Implementations** (via kenken-simd runtime dispatch):
//! - **x86_64**:
//!   - AVX512-VPOPCNT (fastest, native instruction, ~1200-1500 ps)
//!   - POPCNT fallback (four POPCNT64 instructions, ~1200 ps)
//! - **ARM aarch64**: NEON (vcntq_u8 applied to 32 bytes)
//! - **Fallback**: Scalar (all platforms)
//!
//! **Performance**:
//! - AVX512-VPOPCNT: ~1200-1500 ps (2-3x Domain64)
//! - POPCNT fallback: ~1200 ps (4x POPCNT64)
//! - Scalar: ~1000-1200 ps
//! - Acceptable for n > 128 (rare use case)
//!
//! **Layout**:
//! ```text
//! [u64; 4] = [bits 0-63, bits 64-127, bits 128-191, bits 192-255]
//!             [values 1-64, values 65-128, values 129-192, values 193-255]
//! ```

use crate::domain_ops::DomainOps;

#[cfg(feature = "simd-dispatch")]
use kenken_simd::popcount_u256;

/// 256-bit bitmask domain for n ≤ 255
///
/// Stores four u64 values: 64 bits for each 64-value range
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Domain256([u64; 4]);

impl DomainOps for Domain256 {
    fn empty() -> Self {
        Domain256([0, 0, 0, 0])
    }

    fn full(n: u8) -> Self {
        let mut limbs = [u64::MAX; 4];

        // Zero out unused limbs
        for i in 0..4 {
            let limb_start = (i * 64) as u8;
            if limb_start >= n {
                limbs[i] = 0;
            } else {
                let limb_end = ((i + 1) * 64).min(n as usize) as u8;
                let bits_in_limb = limb_end - limb_start;
                if bits_in_limb < 64 {
                    limbs[i] = (1u64 << bits_in_limb) - 1;
                }
            }
        }

        Domain256(limbs)
    }

    fn insert(&mut self, value: u8) {
        debug_assert!(value > 0, "Value must be >= 1");

        let bit_pos = (value - 1) as usize;
        let limb_idx = bit_pos / 64;
        let bit_in_limb = bit_pos % 64;
        self.0[limb_idx] |= 1u64 << bit_in_limb;
    }

    fn remove(&mut self, value: u8) {
        debug_assert!(value > 0, "Value must be >= 1");

        let bit_pos = (value - 1) as usize;
        let limb_idx = bit_pos / 64;
        let bit_in_limb = bit_pos % 64;
        self.0[limb_idx] &= !(1u64 << bit_in_limb);
    }

    fn contains(&self, value: u8) -> bool {
        debug_assert!(value > 0, "Value must be >= 1");

        let bit_pos = (value - 1) as usize;
        let limb_idx = bit_pos / 64;
        let bit_in_limb = bit_pos % 64;
        (self.0[limb_idx] & (1u64 << bit_in_limb)) != 0
    }

    fn count(&self) -> u32 {
        // Use SIMD-optimized popcount via runtime dispatch
        #[cfg(feature = "simd-dispatch")]
        {
            popcount_u256(self.0)
        }
        #[cfg(not(feature = "simd-dispatch"))]
        {
            self.0[0].count_ones() + self.0[1].count_ones() + self.0[2].count_ones() + self.0[3].count_ones()
        }
    }

    fn min(&self) -> Option<u8> {
        // Find the first non-zero limb and then trailing_zeros
        for (i, &limb) in self.0.iter().enumerate() {
            if limb != 0 {
                let bit_pos = limb.trailing_zeros() as u8;
                return Some(1 + (i as u8 * 64) + bit_pos);
            }
        }
        None
    }

    fn max(&self) -> Option<u8> {
        // Find the last non-zero limb and then leading_zeros
        for i in (0..4).rev() {
            if self.0[i] != 0 {
                let bit_pos = 63 - self.0[i].leading_zeros() as u8;
                return Some(1 + (i as u8 * 64) + bit_pos);
            }
        }
        None
    }

    fn and(&self, other: &Self) -> Self {
        Domain256([
            self.0[0] & other.0[0],
            self.0[1] & other.0[1],
            self.0[2] & other.0[2],
            self.0[3] & other.0[3],
        ])
    }

    fn or(&self, other: &Self) -> Self {
        Domain256([
            self.0[0] | other.0[0],
            self.0[1] | other.0[1],
            self.0[2] | other.0[2],
            self.0[3] | other.0[3],
        ])
    }

    fn xor(&self, other: &Self) -> Self {
        Domain256([
            self.0[0] ^ other.0[0],
            self.0[1] ^ other.0[1],
            self.0[2] ^ other.0[2],
            self.0[3] ^ other.0[3],
        ])
    }

    fn complement(&self, n: u8) -> Self {

        let full = Self::full(n);
        Domain256([
            self.0[0] ^ full.0[0],
            self.0[1] ^ full.0[1],
            self.0[2] ^ full.0[2],
            self.0[3] ^ full.0[3],
        ])
    }

    fn iter_values(&self) -> Box<dyn Iterator<Item = u8> + '_> {
        Box::new(
            (0..4)
                .flat_map(move |i| {
                    (0..64)
                        .filter(move |&j| (self.0[i] & (1u64 << j)) != 0)
                        .map(move |j| 1 + (i as u8 * 64) + (j as u8))
                })
        )
    }

    fn clear(&mut self) {
        self.0[0] = 0;
        self.0[1] = 0;
        self.0[2] = 0;
        self.0[3] = 0;
    }

    fn to_string(&self, n: u8) -> String {
        let mut result = String::with_capacity(256);
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
    fn test_domain256_empty_full() {
        let empty = Domain256::empty();
        assert!(empty.is_empty());
        assert_eq!(empty.count(), 0);

        let full = Domain256::full(255);
        assert!(!full.is_empty());
        assert_eq!(full.count(), 255);
    }

    #[test]
    fn test_domain256_insert_remove() {
        let mut d = Domain256::empty();

        // Insert in first limb
        d.insert(1);
        assert!(d.contains(1));
        assert_eq!(d.count(), 1);

        // Insert in last limb
        d.insert(255);
        assert!(d.contains(255));
        assert_eq!(d.count(), 2);

        // Remove
        d.remove(1);
        assert!(!d.contains(1));
        assert_eq!(d.count(), 1);
    }

    #[test]
    fn test_domain256_min_max() {
        let mut d = Domain256::empty();

        assert_eq!(d.min(), None);
        assert_eq!(d.max(), None);

        d.insert(10);
        d.insert(100);
        d.insert(200);

        assert_eq!(d.min(), Some(10));
        assert_eq!(d.max(), Some(200));
    }

    #[test]
    fn test_domain256_bitwise_ops() {
        let d1 = Domain256::full(128);
        let d2 = Domain256::full(255);

        let and_result = d1.and(&d2);
        assert_eq!(and_result.count(), 128); // All values 1-128

        let or_result = d1.or(&d2);
        assert_eq!(or_result.count(), 255); // All values 1-255

        let xor_result = d1.xor(&d2);
        assert_eq!(xor_result.count(), 127); // Values 129-255
    }

    #[test]
    fn test_domain256_iter_values() {
        let mut d = Domain256::empty();
        d.insert(1);
        d.insert(64);
        d.insert(128);
        d.insert(200);
        d.insert(255);

        let values: Vec<u8> = d.iter_values().collect();
        assert_eq!(values, vec![1, 64, 128, 200, 255]);
    }

    #[test]
    fn test_domain256_popcount_accuracy() {
        let mut d = Domain256::empty();

        for v in 1..=255 {
            d.insert(v);
        }

        assert_eq!(d.count(), 255);
    }

    #[test]
    fn test_domain256_complement() {
        let mut d = Domain256::full(100);
        let comp = d.complement(100);

        assert!(comp.is_empty());

        d.remove(50);
        let comp2 = d.complement(100);
        assert_eq!(comp2.count(), 1);
        assert!(comp2.contains(50));
    }

    #[test]
    fn test_domain256_cross_limb_operations() {
        let mut d = Domain256::empty();

        // Test values across different limbs
        d.insert(1);     // Limb 0
        d.insert(65);    // Limb 1
        d.insert(129);   // Limb 2
        d.insert(193);   // Limb 3

        assert_eq!(d.count(), 4);
        assert_eq!(d.min(), Some(1));
        assert_eq!(d.max(), Some(193));

        let values: Vec<u8> = d.iter_values().collect();
        assert_eq!(values, vec![1, 65, 129, 193]);
    }
}
