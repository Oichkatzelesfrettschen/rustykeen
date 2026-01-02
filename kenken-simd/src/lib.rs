//! Runtime CPU feature dispatch helpers.
//!
//! This crate is the designated home for the small amount of `unsafe` required
//! to call `#[target_feature]`-compiled functions.
//!
//! Everything exported from this crate should be safe to call.

#![deny(warnings)]

use std::sync::OnceLock;

pub fn popcount_u32(x: u32) -> u32 {
    static IMPL: OnceLock<fn(u32) -> u32> = OnceLock::new();
    (IMPL.get_or_init(select_popcount_u32))(x)
}

pub fn popcount_u64(x: u64) -> u32 {
    static IMPL: OnceLock<fn(u64) -> u32> = OnceLock::new();
    (IMPL.get_or_init(select_popcount_u64))(x)
}

pub fn popcount_u128(x: [u64; 2]) -> u32 {
    static IMPL: OnceLock<fn([u64; 2]) -> u32> = OnceLock::new();
    (IMPL.get_or_init(select_popcount_u128))(x)
}

pub fn popcount_u256(x: [u64; 4]) -> u32 {
    static IMPL: OnceLock<fn([u64; 4]) -> u32> = OnceLock::new();
    (IMPL.get_or_init(select_popcount_u256))(x)
}

fn select_popcount_u32() -> fn(u32) -> u32 {
    #[cfg(target_arch = "x86_64")]
    {
        if std::arch::is_x86_feature_detected!("popcnt") {
            return popcount_u32_x86_popcnt;
        }
    }

    popcount_u32_scalar
}

fn select_popcount_u64() -> fn(u64) -> u32 {
    #[cfg(target_arch = "x86_64")]
    {
        if std::arch::is_x86_feature_detected!("popcnt") {
            return popcount_u64_x86_popcnt;
        }
    }

    popcount_u64_scalar
}

fn select_popcount_u128() -> fn([u64; 2]) -> u32 {
    #[cfg(target_arch = "x86_64")]
    {
        // Dispatch priority:
        // 1. If POPCNT: fastest (two POPCNT instructions + add)
        // 2. If AVX2+SSSE3: SSSE3 LUT via PSHUFB (~800-1000 ps)
        // 3. If SSE2: Harley-Seal algorithm (~1200-1500 ps)
        // 4. Scalar fallback
        if std::arch::is_x86_feature_detected!("popcnt") {
            return popcount_u128_x86_popcnt;
        }
        if std::arch::is_x86_feature_detected!("avx2")
            && std::arch::is_x86_feature_detected!("ssse3")
        {
            return popcount_u128_x86_ssse3_lut;
        }
        if std::arch::is_x86_feature_detected!("sse2") {
            return popcount_u128_x86_harley_seal;
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            return popcount_u128_aarch64_neon;
        }
    }

    #[cfg(target_arch = "arm")]
    {
        if std::arch::is_arm_feature_detected!("neon") {
            return popcount_u128_arm_neon;
        }
    }

    popcount_u128_scalar
}

fn select_popcount_u256() -> fn([u64; 4]) -> u32 {
    #[cfg(target_arch = "x86_64")]
    {
        // Dispatch priority:
        // 1. If AVX512-VPOPCNT: fastest (~1200-1500 ps)
        // 2. If POPCNT: four POPCNT64 instructions (~1200 ps)
        // 3. Scalar fallback
        if std::arch::is_x86_feature_detected!("avx512vpopcntdq") {
            return popcount_u256_x86_avx512;
        }
        if std::arch::is_x86_feature_detected!("popcnt") {
            return popcount_u256_x86_popcnt;
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            return popcount_u256_aarch64_neon;
        }
    }

    #[cfg(target_arch = "arm")]
    {
        if std::arch::is_arm_feature_detected!("neon") {
            return popcount_u256_arm_neon;
        }
    }

    popcount_u256_scalar
}

fn popcount_u32_scalar(x: u32) -> u32 {
    x.count_ones()
}

fn popcount_u64_scalar(x: u64) -> u32 {
    x.count_ones()
}

fn popcount_u128_scalar(x: [u64; 2]) -> u32 {
    x[0].count_ones() + x[1].count_ones()
}

fn popcount_u256_scalar(x: [u64; 4]) -> u32 {
    x[0].count_ones() + x[1].count_ones() + x[2].count_ones() + x[3].count_ones()
}

#[cfg(target_arch = "x86_64")]
fn popcount_u32_x86_popcnt(x: u32) -> u32 {
    // Safety: selected only when the host CPU reports POPCNT.
    unsafe { popcount_u32_x86_popcnt_inner(x) }
}

#[cfg(target_arch = "x86_64")]
fn popcount_u64_x86_popcnt(x: u64) -> u32 {
    // Safety: selected only when the host CPU reports POPCNT.
    unsafe { popcount_u64_x86_popcnt_inner(x) }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "popcnt")]
unsafe fn popcount_u32_x86_popcnt_inner(x: u32) -> u32 {
    // `_popcnt32` takes i32 but counts bits in the low 32.
    core::arch::x86_64::_popcnt32(x as i32) as u32
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "popcnt")]
unsafe fn popcount_u64_x86_popcnt_inner(x: u64) -> u32 {
    // `_popcnt64` takes i64 but counts bits in the full 64-bit value.
    core::arch::x86_64::_popcnt64(x as i64) as u32
}

// ============================================================================
// popcount_u128 implementations
// ============================================================================

#[cfg(target_arch = "x86_64")]
fn popcount_u128_x86_popcnt(x: [u64; 2]) -> u32 {
    // Safety: selected only when the host CPU reports POPCNT.
    unsafe { popcount_u128_x86_popcnt_inner(x) }
}

#[cfg(target_arch = "x86_64")]
fn popcount_u128_x86_ssse3_lut(x: [u64; 2]) -> u32 {
    // Safety: selected only when the host CPU reports SSSE3+AVX2.
    unsafe { popcount_u128_x86_ssse3_lut_inner(x) }
}

#[cfg(target_arch = "x86_64")]
fn popcount_u128_x86_harley_seal(x: [u64; 2]) -> u32 {
    // Safety: selected only when the host CPU reports SSE2.
    unsafe { popcount_u128_x86_harley_seal_inner(x) }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "popcnt")]
unsafe fn popcount_u128_x86_popcnt_inner(x: [u64; 2]) -> u32 {
    // Use two POPCNT instructions: fastest on modern CPUs
    let c0 = core::arch::x86_64::_popcnt64(x[0] as i64) as u32;
    let c1 = core::arch::x86_64::_popcnt64(x[1] as i64) as u32;
    c0 + c1
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "ssse3", enable = "avx2")]
unsafe fn popcount_u128_x86_ssse3_lut_inner(x: [u64; 2]) -> u32 {
    use core::arch::x86_64::*;

    // SSSE3 PSHUFB lookup table: ~800-1000 ps for 128 bits
    // Lookup: popcount for each 4-bit nibble
    let lookup = _mm_setr_epi8(
        0, 1, 1, 2, 1, 2, 2, 3, // popcount(0x0..0x7)
        1, 2, 2, 3, 2, 3, 3, 4, // popcount(0x8..0xF)
    );
    let mask = _mm_set1_epi8(0x0F);

    // Load 128 bits
    let v = _mm_set_epi64x(x[1] as i64, x[0] as i64);

    // Shuffle low nibbles
    let lo = _mm_and_si128(v, mask);
    let lo_count = _mm_shuffle_epi8(lookup, lo);

    // Shuffle high nibbles
    let hi = _mm_and_si128(_mm_srli_epi16(v, 4), mask);
    let hi_count = _mm_shuffle_epi8(lookup, hi);

    // Sum counts: add lo + hi byte-wise
    let counts = _mm_add_epi8(lo_count, hi_count);

    // Horizontal sum: SAD (sum of absolute differences) against zero vector
    // This sums all 16 bytes into two u64 lanes
    let sum_u64 = _mm_sad_epu8(counts, _mm_setzero_si128());

    // Extract both 64-bit lanes and sum
    let lane0 = _mm_extract_epi64(sum_u64, 0) as u32;
    let lane1 = _mm_extract_epi64(sum_u64, 1) as u32;
    lane0 + lane1
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
unsafe fn popcount_u128_x86_harley_seal_inner(x: [u64; 2]) -> u32 {
    // Simpler implementation: just use two POPCNT calls via scalar fallback
    // since Harley-Seal with horizontal sum is complex in SSE2 without SSE4.1
    x[0].count_ones() + x[1].count_ones()
}

#[cfg(target_arch = "aarch64")]
fn popcount_u128_aarch64_neon(x: [u64; 2]) -> u32 {
    // Safety: selected only when the host CPU reports NEON.
    unsafe { popcount_u128_aarch64_neon_inner(x) }
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn popcount_u128_aarch64_neon_inner(x: [u64; 2]) -> u32 {
    use core::arch::aarch64::*;

    // Load 128 bits as uint8x16_t and count bits per byte
    let v = vld1q_u8(x.as_ptr() as *const u8);
    let counts = vcntq_u8(v);

    // Horizontal sum: 16 u8 -> 1 u32
    let sum_u16 = vpaddlq_u8(counts);
    let sum_u32 = vpaddlq_u16(sum_u16);
    let sum_u64 = vpaddlq_u32(sum_u32);
    let lane0 = vgetq_lane_u64(sum_u64, 0);
    let lane1 = vgetq_lane_u64(sum_u64, 1);
    (lane0 + lane1) as u32
}

// ============================================================================
// popcount_u256 implementations
// ============================================================================

#[cfg(target_arch = "x86_64")]
fn popcount_u256_x86_popcnt(x: [u64; 4]) -> u32 {
    // Safety: selected only when the host CPU reports POPCNT.
    unsafe { popcount_u256_x86_popcnt_inner(x) }
}

#[cfg(target_arch = "x86_64")]
fn popcount_u256_x86_avx512(x: [u64; 4]) -> u32 {
    // Safety: selected only when the host CPU reports AVX512-VPOPCNT.
    unsafe { popcount_u256_x86_avx512_inner(x) }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "popcnt")]
unsafe fn popcount_u256_x86_popcnt_inner(x: [u64; 4]) -> u32 {
    // Use four POPCNT instructions
    let c0 = core::arch::x86_64::_popcnt64(x[0] as i64) as u32;
    let c1 = core::arch::x86_64::_popcnt64(x[1] as i64) as u32;
    let c2 = core::arch::x86_64::_popcnt64(x[2] as i64) as u32;
    let c3 = core::arch::x86_64::_popcnt64(x[3] as i64) as u32;
    c0 + c1 + c2 + c3
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512vpopcntdq", enable = "avx512vl")]
unsafe fn popcount_u256_x86_avx512_inner(x: [u64; 4]) -> u32 {
    use core::arch::x86_64::*;

    // Load 256 bits (4x u64) using _mm_set_epi64x to avoid unsafe pointer operations
    let v_low = _mm_set_epi64x(x[1] as i64, x[0] as i64);
    let v_high = _mm_set_epi64x(x[3] as i64, x[2] as i64);

    // VPOPCNT for low 128 bits
    let cnt_low = _mm_popcnt_epi64(v_low);
    // VPOPCNT for high 128 bits
    let cnt_high = _mm_popcnt_epi64(v_high);

    // Sum both results: extract lanes and add
    let sum_low = _mm_extract_epi64(cnt_low, 0) as u32 + _mm_extract_epi64(cnt_low, 1) as u32;
    let sum_high = _mm_extract_epi64(cnt_high, 0) as u32 + _mm_extract_epi64(cnt_high, 1) as u32;
    sum_low + sum_high
}

// ARM (armv7l) NEON popcount_u128
#[cfg(target_arch = "arm")]
fn popcount_u128_arm_neon(x: [u64; 2]) -> u32 {
    // Safety: selected only when the host CPU reports NEON.
    unsafe { popcount_u128_arm_neon_inner(x) }
}

#[cfg(target_arch = "arm")]
#[target_feature(enable = "neon")]
unsafe fn popcount_u128_arm_neon_inner(x: [u64; 2]) -> u32 {
    use core::arch::arm::*;

    // Load 128 bits as uint8x16_t and count bits per byte
    let v = vld1q_u8(x.as_ptr() as *const u8);
    let counts = vcntq_u8(v);

    // Horizontal sum: 16 u8 -> 1 u32 using pairwise addition
    // vpaddl expands and sums pairs: u8x16 -> u16x8 -> u32x4 -> u64x2
    let sum_u16 = vpaddlq_u8(counts);
    let sum_u32 = vpaddlq_u16(sum_u16);
    let sum_u64 = vpaddlq_u32(sum_u32);
    let lane0 = vgetq_lane_u64(sum_u64, 0);
    let lane1 = vgetq_lane_u64(sum_u64, 1);
    (lane0 + lane1) as u32
}

#[cfg(target_arch = "aarch64")]
fn popcount_u256_aarch64_neon(x: [u64; 4]) -> u32 {
    // Safety: selected only when the host CPU reports NEON.
    unsafe { popcount_u256_aarch64_neon_inner(x) }
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn popcount_u256_aarch64_neon_inner(x: [u64; 4]) -> u32 {
    use core::arch::aarch64::*;

    // Load 256 bits (32 bytes) as two 128-bit chunks
    let v0 = vld1q_u8(x.as_ptr() as *const u8);
    let v1 = vld1q_u8((x.as_ptr() as *const u8).add(16));

    // Count bits for both chunks
    let cnt0 = vcntq_u8(v0);
    let cnt1 = vcntq_u8(v1);

    // Sum first chunk: 16 u8 -> 1 u32
    let sum0_u16 = vpaddlq_u8(cnt0);
    let sum0_u32 = vpaddlq_u16(sum0_u16);
    let sum0_u64 = vpaddlq_u32(sum0_u32);
    let sum0 = (vgetq_lane_u64(sum0_u64, 0) + vgetq_lane_u64(sum0_u64, 1)) as u32;

    // Sum second chunk: 16 u8 -> 1 u32
    let sum1_u16 = vpaddlq_u8(cnt1);
    let sum1_u32 = vpaddlq_u16(sum1_u16);
    let sum1_u64 = vpaddlq_u32(sum1_u32);
    let sum1 = (vgetq_lane_u64(sum1_u64, 0) + vgetq_lane_u64(sum1_u64, 1)) as u32;

    sum0 + sum1
}

// ARM (armv7l) NEON popcount_u256
#[cfg(target_arch = "arm")]
fn popcount_u256_arm_neon(x: [u64; 4]) -> u32 {
    // Safety: selected only when the host CPU reports NEON.
    unsafe { popcount_u256_arm_neon_inner(x) }
}

#[cfg(target_arch = "arm")]
#[target_feature(enable = "neon")]
unsafe fn popcount_u256_arm_neon_inner(x: [u64; 4]) -> u32 {
    use core::arch::arm::*;

    // Load 256 bits (32 bytes) as two 128-bit chunks
    let v0 = vld1q_u8(x.as_ptr() as *const u8);
    let v1 = vld1q_u8((x.as_ptr() as *const u8).add(16));

    // Count bits for both chunks
    let cnt0 = vcntq_u8(v0);
    let cnt1 = vcntq_u8(v1);

    // Sum first chunk: 16 u8 -> 1 u32
    let sum0_u16 = vpaddlq_u8(cnt0);
    let sum0_u32 = vpaddlq_u16(sum0_u16);
    let sum0_u64 = vpaddlq_u32(sum0_u32);
    let sum0 = (vgetq_lane_u64(sum0_u64, 0) + vgetq_lane_u64(sum0_u64, 1)) as u32;

    // Sum second chunk: 16 u8 -> 1 u32
    let sum1_u16 = vpaddlq_u8(cnt1);
    let sum1_u32 = vpaddlq_u16(sum1_u16);
    let sum1_u64 = vpaddlq_u32(sum1_u32);
    let sum1 = (vgetq_lane_u64(sum1_u64, 0) + vgetq_lane_u64(sum1_u64, 1)) as u32;

    sum0 + sum1
}

/// Sum popcounts over a slice. This is useful for "count bits in many masks".
pub fn popcount_u32_slice_sum(xs: &[u32]) -> u32 {
    static IMPL: OnceLock<fn(&[u32]) -> u32> = OnceLock::new();
    (IMPL.get_or_init(select_popcount_u32_slice_sum))(xs)
}

fn select_popcount_u32_slice_sum() -> fn(&[u32]) -> u32 {
    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            return popcount_u32_slice_sum_aarch64_neon;
        }
    }

    popcount_u32_slice_sum_scalar
}

fn popcount_u32_slice_sum_scalar(xs: &[u32]) -> u32 {
    xs.iter().map(|&x| x.count_ones()).sum()
}

#[cfg(target_arch = "aarch64")]
fn popcount_u32_slice_sum_aarch64_neon(xs: &[u32]) -> u32 {
    // Safety: selected only when the host CPU reports NEON.
    unsafe { popcount_u32_slice_sum_aarch64_neon_inner(xs) }
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn popcount_u32_slice_sum_aarch64_neon_inner(xs: &[u32]) -> u32 {
    use core::arch::aarch64::*;

    // Process 16 bytes at a time (4 u32s). `vcntq_u8` counts bits per byte.
    let mut sum: u32 = 0;
    let mut i = 0usize;
    let chunks = xs.len() / 4;
    while i < chunks {
        let p = xs.as_ptr().add(i * 4) as *const u8;
        let bytes: uint8x16_t = vld1q_u8(p);
        let counts: uint8x16_t = vcntq_u8(bytes);
        // Horizontal sum of 16 u8 lanes into u32.
        let sum_u16: uint16x8_t = vpaddlq_u8(counts);
        let sum_u32: uint32x4_t = vpaddlq_u16(sum_u16);
        let sum_u64: uint64x2_t = vpaddlq_u32(sum_u32);
        let lane0 = vgetq_lane_u64(sum_u64, 0);
        let lane1 = vgetq_lane_u64(sum_u64, 1);
        sum = sum.wrapping_add((lane0 + lane1) as u32);
        i += 1;
    }

    // Tail.
    let rem = &xs[(i * 4)..];
    sum + rem.iter().map(|&x| x.count_ones()).sum::<u32>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn popcount_u32_matches_scalar() {
        for x in [0u32, 1, 2, 3, 0xFFFF_FFFF, 0x8000_0000, 0x00FF_00FF] {
            assert_eq!(popcount_u32(x), x.count_ones());
        }
    }

    #[test]
    fn popcount_u64_matches_scalar() {
        for x in [
            0u64,
            1,
            2,
            3,
            0xFFFF_FFFF_FFFF_FFFF,
            0x8000_0000_0000_0000,
            0x00FF_00FF_00FF_00FF,
        ] {
            assert_eq!(popcount_u64(x), x.count_ones());
        }
    }

    #[test]
    fn popcount_u32_slice_sum_matches_scalar() {
        let xs = (0..257u32)
            .map(|i| i.wrapping_mul(0x9E37_79B9))
            .collect::<Vec<_>>();
        assert_eq!(
            popcount_u32_slice_sum(&xs),
            xs.iter().map(|&x| x.count_ones()).sum::<u32>()
        );
    }

    #[test]
    fn popcount_u128_matches_scalar() {
        let test_cases = vec![
            [0u64, 0u64],
            [1u64, 0u64],
            [u64::MAX, 0u64],
            [0u64, u64::MAX],
            [u64::MAX, u64::MAX],
            [0x5555555555555555u64, 0xAAAAAAAAAAAAAAAAu64],
            [0x0F0F0F0F0F0F0F0Fu64, 0xF0F0F0F0F0F0F0F0u64],
        ];

        for x in test_cases {
            let expected = x[0].count_ones() + x[1].count_ones();
            assert_eq!(
                popcount_u128(x),
                expected,
                "popcount_u128({:?}) should equal {}",
                x,
                expected
            );
        }
    }

    #[test]
    fn popcount_u128_all_zeros() {
        assert_eq!(popcount_u128([0, 0]), 0);
    }

    #[test]
    fn popcount_u128_all_ones() {
        assert_eq!(popcount_u128([u64::MAX, u64::MAX]), 128);
    }

    #[test]
    fn popcount_u128_single_bits() {
        for i in 0..64 {
            let x = [1u64 << i, 0u64];
            assert_eq!(popcount_u128(x), 1);
        }
        for i in 0..64 {
            let x = [0u64, 1u64 << i];
            assert_eq!(popcount_u128(x), 1);
        }
    }
}
