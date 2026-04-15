#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

pub mod error;

#[cfg(feature = "io-rkyv")]
pub mod rkyv_snapshot;

#[cfg(feature = "format-sgt-desc")]
pub mod sgt_desc_snapshot;
