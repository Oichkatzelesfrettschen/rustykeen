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

fn popcount_u32_scalar(x: u32) -> u32 {
    x.count_ones()
}

fn popcount_u64_scalar(x: u64) -> u32 {
    x.count_ones()
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

/// Sum popcounts over a slice. This is useful for “count bits in many masks”.
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
}
