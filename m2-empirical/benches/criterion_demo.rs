#![allow(missing_docs, clippy::unwrap_used, clippy::expect_used)]
//! Placeholder criterion harness — one benchmark group per contract, just
//! enough to prove the wiring. The actual contract gates (doubling
//! ratios, R² fits) are run by `pv` against the JSON criterion produces
//! in `target/criterion/`, not by the test runner — that's why this
//! file is excluded from coverage by `cargo llvm-cov --ignore-filename-regex 'benches/'`.
//!
//! FALSIFY-IF-001 asm inspection is deferred to an external
//! `cargo-show-asm` gate; this bench provides the wall-clock half of
//! `iterator-fusion-v1`.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use m2_empirical::{
    fused_chain_sum, hand_loop_sum, naive_brackets, rank_then_total, rivalry_pair_count,
    unbeaten_count, weight_class_boundary,
};

fn bench_constant(c: &mut Criterion) {
    use std::collections::HashMap;
    let mut roster: HashMap<u64, u64> = HashMap::new();
    for k in 0..1024_u64 {
        roster.insert(k, 1800 + k);
    }
    c.bench_function("hashmap_get_constant_elo", |b| {
        b.iter(|| m2_empirical::hashmap_get_constant(black_box(&roster), black_box(&512_u64)));
    });
}

fn bench_logarithmic(c: &mut Criterion) {
    let weights: Vec<i64> = (0..1_048_576_i64).collect();
    c.bench_function("weight_class_boundary", |b| {
        b.iter(|| weight_class_boundary(black_box(&weights), black_box(777)));
    });
}

fn bench_linear(c: &mut Criterion) {
    let losses: Vec<u32> = (0..65_536_u32).map(|i| u32::from(i % 4 != 0)).collect();
    c.bench_function("unbeaten_count", |b| {
        b.iter(|| unbeaten_count(black_box(&losses)));
    });
}

fn bench_linearithmic(c: &mut Criterion) {
    let elos: Vec<i64> = (0..16_384_i64).rev().collect();
    c.bench_function("rank_then_total", |b| {
        b.iter(|| rank_then_total(black_box(&elos)));
    });
}

fn bench_quadratic(c: &mut Criterion) {
    c.bench_function("rivalry_pair_count_512", |b| {
        b.iter(|| rivalry_pair_count(black_box(512)));
    });
}

fn bench_exponential(c: &mut Criterion) {
    c.bench_function("naive_brackets_20", |b| {
        b.iter(|| naive_brackets(black_box(20)));
    });
}

fn bench_iterator_fusion(c: &mut Criterion) {
    let xs: Vec<i64> = (-32_768_i64..32_768).collect();
    let mut group = c.benchmark_group("iterator_fusion_parity");
    group.bench_function("fused_chain_sum", |b| {
        b.iter(|| fused_chain_sum(black_box(&xs)));
    });
    group.bench_function("hand_loop_sum", |b| {
        b.iter(|| hand_loop_sum(black_box(&xs)));
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_constant,
    bench_logarithmic,
    bench_linear,
    bench_linearithmic,
    bench_quadratic,
    bench_exponential,
    bench_iterator_fusion,
);
criterion_main!(benches);
