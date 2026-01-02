/// SIMD Popcount Effectiveness Benchmarks
///
/// Measures the performance of SIMD-accelerated popcount operations
/// vs scalar implementations across different data sizes and patterns.
///
/// Tests:
/// - Single value popcount (u32 and u64)
/// - Slice popcount with various patterns
/// - Performance impact of SIMD dispatch vs direct scalar
///
/// Flamegraph Output:
/// - CPU flamegraphs generated to target/criterion/*/profile/flamegraph.svg
/// - Run with `cargo bench --bench simd_effectiveness` to generate profiling data

use criterion::{criterion_group, criterion_main, Criterion};
use pprof::criterion::{Output, PProfProfiler};
use kenken_simd::{popcount_u32, popcount_u64, popcount_u32_slice_sum};

fn benchmark_popcount_u32_single(c: &mut Criterion) {
    let val = std::hint::black_box(0xDEADBEEFu32);

    c.bench_function("popcount_u32_single", |b| {
        b.iter(|| popcount_u32(val))
    });
}

fn benchmark_popcount_u64_single(c: &mut Criterion) {
    let val = std::hint::black_box(0xDEADBEEFDEADBEEFu64);

    c.bench_function("popcount_u64_single", |b| {
        b.iter(|| popcount_u64(val))
    });
}

fn benchmark_popcount_u32_slice_small(c: &mut Criterion) {
    let data: Vec<u32> = (0..8u32).map(|i| i.wrapping_mul(0x9E3779B9)).collect();

    c.bench_function("popcount_u32_slice_small_8", |b| {
        b.iter(|| popcount_u32_slice_sum(std::hint::black_box(&data)))
    });
}

fn benchmark_popcount_u32_slice_medium(c: &mut Criterion) {
    let data: Vec<u32> = (0..256u32).map(|i| i.wrapping_mul(0x9E3779B9)).collect();

    c.bench_function("popcount_u32_slice_medium_256", |b| {
        b.iter(|| popcount_u32_slice_sum(std::hint::black_box(&data)))
    });
}

fn benchmark_popcount_u32_slice_large(c: &mut Criterion) {
    let data: Vec<u32> = (0..4096u32).map(|i| i.wrapping_mul(0x9E3779B9)).collect();

    c.bench_function("popcount_u32_slice_large_4096", |b| {
        b.iter(|| popcount_u32_slice_sum(std::hint::black_box(&data)))
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets =
        benchmark_popcount_u32_single,
        benchmark_popcount_u64_single,
        benchmark_popcount_u32_slice_small,
        benchmark_popcount_u32_slice_medium,
        benchmark_popcount_u32_slice_large,
}

criterion_main!(benches);
