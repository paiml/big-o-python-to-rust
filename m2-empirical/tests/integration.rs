#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Integration test: exercise every public function from outside the
//! crate. Mirrors `make demo`'s contract walk-through on the UFC roster.

use std::collections::HashMap;

use m2_empirical::{
    fused_chain_sum, hand_loop_sum, hashmap_get_constant, make_report, naive_brackets,
    rank_then_total, results_equal, rivalry_pair_count, unbeaten_count, weight_class_boundary,
    ComplexityClass,
};

#[test]
fn contract_walkthrough() {
    // O(1) — Elo lookup.
    let mut roster: HashMap<&str, u32> = HashMap::new();
    roster.insert("Khabib", 2200);
    assert_eq!(hashmap_get_constant(&roster, &"Khabib"), Some(2200));

    // O(log n) — weight-class boundary.
    let weights: Vec<i64> = (0..256).collect();
    let (idx, ops) = weight_class_boundary(&weights, 99);
    assert_eq!(idx, Some(99));
    assert!(ops <= 9);

    // O(n) — unbeaten count.
    let losses: Vec<u32> = (0..256).map(|i| u32::from(i % 4 != 0)).collect();
    assert_eq!(unbeaten_count(&losses), 64);

    // O(n log n) — rank-then-total over Elo slice.
    let elos: Vec<i64> = (0..256).collect();
    let direct: i64 = elos.iter().copied().sum();
    assert_eq!(rank_then_total(&elos), direct);

    // O(n²) — rivalry pair count.
    assert_eq!(rivalry_pair_count(8), 64);

    // O(2ⁿ) — naive brackets.
    assert_eq!(naive_brackets(10), 55);

    // iterator-fusion.
    let probe = [1_i64, -1, 2, -2, 3, -3];
    assert_eq!(fused_chain_sum(&probe), hand_loop_sum(&probe));
    assert!(results_equal(&probe));

    // transpile-preservation.
    let report = make_report(10.0, 2.5, ComplexityClass::Linear, ComplexityClass::Linear);
    assert!(report.class_preserved);
    assert!((report.speedup - 4.0).abs() < 1e-9);
}
