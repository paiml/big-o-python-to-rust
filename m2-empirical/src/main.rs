#![allow(clippy::print_stdout, clippy::print_stderr)]
//! M2 demo binary: exercise each canonical UFC-roster example once and
//! print a contract-by-contract OK marker stream. Useful for `make demo`
//! to confirm the wiring at runtime; the empirical proof itself lives in
//! `benches/criterion_demo.rs`.

use std::collections::HashMap;

use m2_empirical::{
    contract_marker, fused_chain_sum, hand_loop_sum, hashmap_get_constant, make_report,
    naive_brackets, rank_then_total, results_equal, rivalry_pair_count, unbeaten_count,
    weight_class_boundary, ComplexityClass,
};

fn main() {
    println!("M2 · Criterion-gated complexity contracts (UFC/BJJ correctness witnesses)\n");

    // O(1) — Elo lookup by fighter name (lesson 2.2.1).
    let mut roster: HashMap<&str, i32> = HashMap::new();
    roster.insert("Khabib", 2200);
    roster.insert("McGregor", 2050);
    let hit = hashmap_get_constant(&roster, &"Khabib");
    println!("  complexity-constant-v1     : HashMap::get(\"Khabib\") = {hit:?} (lesson 2.2.1)");

    // O(log n) — weight-class boundary search.
    let weights: Vec<i64> = (115..=265).collect();
    let (idx, ops) = weight_class_boundary(&weights, 155);
    println!("  complexity-logarithmic-v1  : weight_class_boundary(155 lb) = {idx:?}, ops = {ops}");

    // O(n) — unbeaten count over a 1024-fighter roster (lesson 2.1.1).
    let losses: Vec<u32> = (0..1024).map(|i| u32::from(i % 4 != 0)).collect();
    let unbeaten = unbeaten_count(&losses);
    println!(
        "  complexity-linear-v1       : unbeaten_count of 1024 fighters = {unbeaten} (lesson 2.1.1)"
    );

    // O(n log n) — rank-then-total (lesson 2.3.1).
    let elos: Vec<i64> = (1800..1800 + 1024).collect();
    let total = rank_then_total(&elos);
    println!(
        "  complexity-linearithmic-v1 : rank_then_total = {total} (order-invariant, lesson 2.3.1)"
    );

    // O(n²) — rivalry pair count.
    let pairs = rivalry_pair_count(32);
    println!("  complexity-quadratic-v1    : rivalry_pair_count(n=32) = {pairs}");

    // O(2ⁿ) — naive bracket count.
    let b20 = naive_brackets(20);
    println!("  complexity-exponential-v1  : naive_brackets(20) = {b20}");

    // iterator-fusion.
    let streaks = [1_i64, -2, 3, -4, 5];
    println!(
        "  iterator-fusion-v1         : fused = {}, hand = {}, equal = {}",
        fused_chain_sum(&streaks),
        hand_loop_sum(&streaks),
        results_equal(&streaks),
    );

    // transpile-preservation — Khabib's win-streak prediction, Python vs Rust.
    let report = make_report(
        100.0,
        25.0,
        ComplexityClass::Linear,
        ComplexityClass::Linear,
    );
    println!(
        "  preserved-across-transpile : speedup = {:.2}x, class_preserved = {}",
        report.speedup, report.class_preserved
    );

    eprintln!("{}", contract_marker());
}
