//! M2 — One canonical example per complexity class, each bound to its
//! `complexity-*-v1.yaml` contract. UFC/BJJ roster operations replace
//! the generic abstractions, and each function is anchored to one of
//! the course's Python→Rust translation lessons:
//!
//! | Complexity | Operation | Lesson |
//! |---|---|---|
//! | O(1)        | [`hashmap_get_constant`] (Elo lookup by fighter name) | 2.2.1 (dict → `HashMap`) |
//! | O(log n)    | [`weight_class_boundary`] (binary search) | — |
//! | O(n)        | [`unbeaten_count`] (count zero-loss fighters) | 2.1.1 (list comp → iterator) |
//! | O(n log n)  | [`rank_then_total`] (sort Elos then sum) | 2.3.1 (sorted → `sort_unstable`) |
//! | O(n²)       | [`rivalry_pair_count`] (every (i,j) pair) | — |
//! | O(2ⁿ)       | [`naive_brackets`] (single-elimination brackets) | — |
//!
//! All public functions here are designed to be both (a) the cheapest
//! correct realization of their class and (b) tractable to unit-test for
//! 100% line coverage. Criterion benches in `benches/criterion_demo.rs`
//! exercise the empirical ratio gates declared in each contract; the
//! `proptest` and unit tests below verify *correctness*, which is what
//! a complexity-class proof actually rests on (the wrong answer in
//! constant time is not in `O(1)`, it's in `O(undefined)`).

use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::{BuildHasher, Hash};

// =====================================================================
// complexity-constant-v1
// =====================================================================

/// O(1) constant-time `HashMap` lookup — "look up a fighter's Elo by
/// name". Course lesson 2.2.1 (Python `dict[key]` → Rust `HashMap::get`).
///
/// Bound to: `contracts/complexity-constant-v1.yaml`. The criterion bench
/// in `benches/criterion_demo.rs` proves the doubling-ratio invariant
/// `mean(t(2n)) / mean(t(n)) ∈ [0.85, 1.15]`. This unit-level entry
/// point simply forwards to `HashMap::get` so the contract gate measures
/// the underlying stdlib primitive.
///
/// In Python: `roster["Khabib"]`. In Rust: `roster.get(&"Khabib")`. Same
/// O(1) class; Rust gets you compile-time guarantees on the key type.
#[must_use]
pub fn hashmap_get_constant<K, V, S>(map: &HashMap<K, V, S>, key: &K) -> Option<V>
where
    K: Hash + Eq,
    V: Clone,
    S: BuildHasher,
{
    map.get(key).cloned()
}

// =====================================================================
// complexity-logarithmic-v1
// =====================================================================

/// O(log n) binary search over a sorted weight roster — locates the
/// index of `target` (a fighter weight in pounds, e.g. 155 for the
/// lightweight boundary) and reports the comparison count.
///
/// Returns `(Some(index), comparisons)` if `target` is found, otherwise
/// `(None, comparisons)`. The counter is the empirical witness for
/// proof-obligation `KANI-CL-001` / `FALSIFY-CL-002`: it must remain
/// bounded by `ceil(log2(n)) + 1` for every roster size.
///
/// Bound to: `contracts/complexity-logarithmic-v1.yaml`. The Python
/// analog is `bisect.bisect_left`; the Rust analog is the stdlib
/// `slice::binary_search` (which we re-implement here so we can count
/// comparisons as the empirical witness).
#[must_use]
pub fn weight_class_boundary(sorted_weights: &[i64], target: i64) -> (Option<usize>, u32) {
    let mut ops: u32 = 0;
    if sorted_weights.is_empty() {
        return (None, ops);
    }
    let mut lo: usize = 0;
    let mut hi: usize = sorted_weights.len();
    while lo < hi {
        ops = ops.saturating_add(1);
        let mid = lo + (hi - lo) / 2;
        let probe = sorted_weights[mid];
        match probe.cmp(&target) {
            Ordering::Equal => return (Some(mid), ops),
            Ordering::Less => lo = mid + 1,
            Ordering::Greater => hi = mid,
        }
    }
    (None, ops)
}

// =====================================================================
// complexity-linear-v1
// =====================================================================

/// O(n) linear scan: count the number of unbeaten fighters (entries
/// equal to zero in the losses slice).
///
/// Course lesson 2.1.1 (Python list comprehension → Rust iterator —
/// `[f for f in roster if f.losses == 0]` becomes
/// `roster.iter().filter(|f| f.losses == 0).count()`).
///
/// Bound to: `contracts/complexity-linear-v1.yaml`. The empirical
/// doubling ratio gate is `[1.8, 2.2]`. Counts in a single pass with no
/// intermediate allocation — the iterator-fusion compiler optimization
/// applies here.
#[must_use]
pub fn unbeaten_count(losses: &[u32]) -> u32 {
    let mut acc: u32 = 0;
    for &l in losses {
        if l == 0 {
            acc = acc.saturating_add(1);
        }
    }
    acc
}

// =====================================================================
// complexity-linearithmic-v1
// =====================================================================

/// O(n log n) rank-then-total: clone the Elo vector, `sort_unstable`,
/// then sum.
///
/// Course lesson 2.3.1 (Python `sorted()` → Rust `sort_unstable` —
/// Timsort vs pdqsort, same Big O class, better constants on Rust).
///
/// Bound to: `contracts/complexity-linearithmic-v1.yaml`. The sort
/// dominates; the linear scan keeps the function returning a useful
/// value while remaining in the n·log n class.
///
/// The order-invariant correctness property: `rank_then_total(xs)`
/// equals `linear_sum_of_elos(xs)` for any permutation — this is what
/// the proptest exercises.
#[must_use]
pub fn rank_then_total(elos: &[i64]) -> i64 {
    let mut buf = elos.to_vec();
    buf.sort_unstable();
    let mut acc: i64 = 0;
    for &x in &buf {
        acc = acc.wrapping_add(x);
    }
    acc
}

// =====================================================================
// complexity-quadratic-v1
// =====================================================================

/// O(n²) nested pair count: visits every (i, j) pair of fighters in a
/// roster of size `n` exactly once. Returns `n²` — the head-to-head
/// rivalry pair count.
///
/// Bound to: `contracts/complexity-quadratic-v1.yaml`. The pair-visit
/// count is `n²` by construction (proof obligation `FALSIFY-QD-002`).
/// This is the witness loop, not a closed-form `n * n` shortcut — we
/// want the empirical bench to measure the actual double-loop work.
#[must_use]
pub fn rivalry_pair_count(n: u32) -> u64 {
    let mut acc: u64 = 0;
    for _ in 0..n {
        for _ in 0..n {
            acc = acc.saturating_add(1);
        }
    }
    acc
}

// =====================================================================
// complexity-exponential-v1
// =====================================================================

/// O(φⁿ) ≈ O(2ⁿ) naive bracket count: number of single-elimination
/// tournament brackets for `n` fighters.
///
/// Grows like Fibonacci. **No memoization** — the whole point is to be
/// the witness for the exponential-time gate.
///
/// Bound to: `contracts/complexity-exponential-v1.yaml`. The unit-step
/// ratio gate is `[1.8, 2.2]`. Saturates on overflow so it remains
/// total over all `u32` inputs.
#[must_use]
pub fn naive_brackets(n: u32) -> u64 {
    if n < 2 {
        return u64::from(n);
    }
    naive_brackets(n - 1).saturating_add(naive_brackets(n - 2))
}

// =====================================================================
// iterator-fusion-v1
// =====================================================================

/// Iterator-fusion contract: chained `.iter().filter(...).map(...).sum()`
/// — the canonical Rust translation of a Python generator expression.
///
/// Bound to: `contracts/iterator-fusion-v1.yaml`. Asm-level proof of
/// "no intermediate Vec alloc" is deferred to an external
/// `cargo-show-asm` gate (see README "FALSIFY-IF-001 deferral"); this
/// crate proves *result equivalence* with the hand-loop equivalent
/// via [`results_equal`] under proptest.
///
/// Filters out negatives, doubles the rest, sums to `i64` with wrapping
/// arithmetic for totality. The semantic frame: "for every fighter
/// with a positive win streak, double the streak and total it up".
#[must_use]
pub fn fused_chain_sum(xs: &[i64]) -> i64 {
    xs.iter()
        .filter(|&&x| x >= 0)
        .map(|&x| x.wrapping_mul(2))
        .fold(0_i64, i64::wrapping_add)
}

/// Hand-written imperative equivalent of [`fused_chain_sum`].
///
/// Bound to: `contracts/iterator-fusion-v1.yaml` — the parity-of-result
/// half of the contract. Wall-clock parity is exercised by criterion.
/// The lesson: chained adapters and a hand loop compile to the same
/// tight inner loop in release mode.
#[must_use]
pub fn hand_loop_sum(xs: &[i64]) -> i64 {
    let mut acc: i64 = 0;
    for &x in xs {
        if x >= 0 {
            acc = acc.wrapping_add(x.wrapping_mul(2));
        }
    }
    acc
}

/// Proof-of-equivalence predicate. Returns `true` iff [`fused_chain_sum`]
/// and [`hand_loop_sum`] agree on this input.
///
/// Bound to: `contracts/iterator-fusion-v1.yaml`, falsification test
/// `FALSIFY-IF-003`.
#[must_use]
pub fn results_equal(xs: &[i64]) -> bool {
    fused_chain_sum(xs) == hand_loop_sum(xs)
}

// =====================================================================
// complexity-preserved-across-transpile-v1
// =====================================================================

/// The six complexity classes this workspace tracks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComplexityClass {
    /// O(1).
    Constant,
    /// O(log n).
    Logarithmic,
    /// O(n).
    Linear,
    /// O(n log n).
    Linearithmic,
    /// O(n²).
    Quadratic,
    /// O(2ⁿ).
    Exponential,
}

/// Result of comparing a Python source program S against its
/// depyler-transpiled Rust output R = depyler(S).
///
/// For example, Khabib's win-streak prediction in Python
/// (list-comprehension form) versus its depyler-translated Rust form
/// (iterator form) — same class, ~4x speedup.
///
/// Bound to: `contracts/complexity-preserved-across-transpile-v1.yaml`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TranspileReport {
    /// `mean(t_py) / mean(t_rs)`. A value > 1.0 means Rust is faster.
    pub speedup: f64,
    /// Whether `class_emp(depyler(S)) == class_emp(S)`.
    pub class_preserved: bool,
}

/// Construct a [`TranspileReport`] from paired Python and Rust timings
/// and their fitted complexity classes.
///
/// Returns `speedup = 0.0` and `class_preserved = false` if `rs_time`
/// is non-positive or non-finite — that's a measurement bug, not a
/// passing contract.
///
/// Bound to: `contracts/complexity-preserved-across-transpile-v1.yaml`.
#[must_use]
pub fn make_report(
    py_time: f64,
    rs_time: f64,
    py_class: ComplexityClass,
    rs_class: ComplexityClass,
) -> TranspileReport {
    if rs_time <= 0.0 || !rs_time.is_finite() || !py_time.is_finite() {
        return TranspileReport {
            speedup: 0.0,
            class_preserved: false,
        };
    }
    TranspileReport {
        speedup: py_time / rs_time,
        class_preserved: py_class == rs_class,
    }
}

/// Runtime marker for the demo binary.
#[must_use]
pub const fn contract_marker() -> &'static str {
    "contract: m2-empirical holds — OK"
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::bool_assert_comparison,
    clippy::float_cmp,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // ---- complexity-constant-v1 ----

    #[test]
    fn hashmap_get_returns_fighter_elo() {
        let mut roster: HashMap<String, i32> = HashMap::new();
        roster.insert("Khabib".to_owned(), 2200);
        roster.insert("McGregor".to_owned(), 2050);
        roster.insert("Adesanya".to_owned(), 2150);
        assert_eq!(
            hashmap_get_constant(&roster, &"Khabib".to_owned()),
            Some(2200)
        );
        assert_eq!(
            hashmap_get_constant(&roster, &"McGregor".to_owned()),
            Some(2050)
        );
        assert_eq!(hashmap_get_constant(&roster, &"Unknown".to_owned()), None);
    }

    // ---- complexity-logarithmic-v1 ----

    #[test]
    fn weight_class_boundary_finds_lightweight_cutoff() {
        // Weights 115..=265 — UFC weight-class spectrum.
        let weights: Vec<i64> = (115..=265).collect();
        let (idx, ops) = weight_class_boundary(&weights, 155);
        assert_eq!(idx, Some(40)); // 155 - 115 = 40
                                   // ceil(log2(151)) + 1 = 9, comfortably bounded.
        assert!(ops <= 9, "ops={ops}, expected ≤ 9");
    }

    #[test]
    fn weight_class_boundary_returns_none_for_missing() {
        let weights: Vec<i64> = (0..1024).step_by(2).collect();
        let (idx, ops) = weight_class_boundary(&weights, 999_999);
        assert_eq!(idx, None);
        assert!(ops <= 11);
    }

    #[test]
    fn weight_class_boundary_empty_slice() {
        let (idx, ops) = weight_class_boundary(&[], 155);
        assert_eq!(idx, None);
        assert_eq!(ops, 0);
    }

    #[test]
    fn weight_class_boundary_target_below_range() {
        let weights: Vec<i64> = (10..20).collect();
        let (idx, _) = weight_class_boundary(&weights, 0);
        assert_eq!(idx, None);
    }

    #[test]
    fn weight_class_boundary_target_above_range() {
        let weights: Vec<i64> = (10..20).collect();
        let (idx, _) = weight_class_boundary(&weights, 100);
        assert_eq!(idx, None);
    }

    #[test]
    fn weight_class_boundary_doubles_only_adds_one_op() {
        // Doubling roster size adds at most one comparison.
        let small: Vec<i64> = (0..1024).collect();
        let big: Vec<i64> = (0..2048).collect();
        let (_, ops_small) = weight_class_boundary(&small, 999);
        let (_, ops_big) = weight_class_boundary(&big, 1999);
        assert!(ops_big <= ops_small + 1);
    }

    // ---- complexity-linear-v1 ----

    #[test]
    fn unbeaten_count_known() {
        // 1024 fighters, every 4th is unbeaten → 256 unbeaten.
        let losses: Vec<u32> = (0..1024).map(|i| u32::from(i % 4 != 0)).collect();
        assert_eq!(unbeaten_count(&losses), 256);
        // Empty roster → 0.
        let empty: Vec<u32> = Vec::new();
        assert_eq!(unbeaten_count(&empty), 0);
        // All beaten.
        assert_eq!(unbeaten_count(&[1, 2, 3]), 0);
        // All unbeaten.
        assert_eq!(unbeaten_count(&[0, 0, 0]), 3);
    }

    #[test]
    fn unbeaten_count_doubles_with_roster() {
        let small: Vec<u32> = (0..1024).map(|i| u32::from(i % 4 != 0)).collect();
        let big: Vec<u32> = (0..2048).map(|i| u32::from(i % 4 != 0)).collect();
        assert_eq!(unbeaten_count(&small), 256);
        assert_eq!(unbeaten_count(&big), 512);
    }

    // ---- complexity-linearithmic-v1 ----

    #[test]
    fn rank_then_total_independent_of_order() {
        // Sorting then summing is order-invariant.
        let a = [3_i64, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5];
        let mut b = a;
        b.reverse();
        assert_eq!(rank_then_total(&a), rank_then_total(&b));
        // Empty slice → 0.
        assert_eq!(rank_then_total(&[]), 0);
    }

    // ---- complexity-quadratic-v1 ----

    #[test]
    fn rivalry_pair_count_is_n_squared() {
        // The pair count is exactly n² by construction.
        assert_eq!(rivalry_pair_count(0), 0);
        assert_eq!(rivalry_pair_count(1), 1);
        assert_eq!(rivalry_pair_count(32), 32 * 32);
        assert_eq!(rivalry_pair_count(64), 64 * 64);
    }

    #[test]
    fn rivalry_pair_count_doubles_quadruples() {
        // Doubling n quadruples the pair count.
        let a = rivalry_pair_count(32);
        let b = rivalry_pair_count(64);
        assert_eq!(b, 4 * a);
    }

    // ---- complexity-exponential-v1 ----

    #[test]
    fn naive_brackets_small_cases() {
        // 0, 1, 1, 2, 3, 5, 8, 13, 21, 34
        let expected = [0_u64, 1, 1, 2, 3, 5, 8, 13, 21, 34];
        for (i, want) in expected.iter().enumerate() {
            let got = naive_brackets(u32::try_from(i).unwrap_or(0));
            assert_eq!(got, *want, "brackets({i})");
        }
    }

    #[test]
    fn naive_brackets_at_20() {
        assert_eq!(naive_brackets(20), 6765);
    }

    // ---- iterator-fusion-v1 ----

    #[test]
    fn fused_chain_and_hand_loop_match() {
        let xs = [1_i64, -2, 3, -4, 5, -6];
        let fused = fused_chain_sum(&xs);
        let hand = hand_loop_sum(&xs);
        assert_eq!(fused, hand);
        // 2*1 + 2*3 + 2*5 = 18.
        assert_eq!(fused, 18);
        assert!(results_equal(&xs));
    }

    #[test]
    fn fused_chain_empty() {
        assert_eq!(fused_chain_sum(&[]), 0);
        assert_eq!(hand_loop_sum(&[]), 0);
        assert!(results_equal(&[]));
    }

    // ---- complexity-preserved-across-transpile-v1 ----

    #[test]
    fn report_records_speedup_and_preservation() {
        let r = make_report(
            100.0,
            25.0,
            ComplexityClass::Linear,
            ComplexityClass::Linear,
        );
        assert!((r.speedup - 4.0).abs() < 1e-9);
        assert!(r.class_preserved);
        let copy = r;
        assert_eq!(format!("{copy:?}").is_empty(), false);
    }

    #[test]
    fn report_flags_class_change() {
        let r = make_report(
            100.0,
            25.0,
            ComplexityClass::Linear,
            ComplexityClass::Logarithmic,
        );
        assert!(!r.class_preserved);
    }

    #[test]
    fn report_rejects_nonpositive_rs_time() {
        let r = make_report(100.0, 0.0, ComplexityClass::Linear, ComplexityClass::Linear);
        assert!((r.speedup - 0.0).abs() < f64::EPSILON);
        assert!(!r.class_preserved);
        let r2 = make_report(
            100.0,
            -1.0,
            ComplexityClass::Linear,
            ComplexityClass::Linear,
        );
        assert!(!r2.class_preserved);
    }

    #[test]
    fn report_rejects_nonfinite_times() {
        let r = make_report(
            f64::NAN,
            10.0,
            ComplexityClass::Linear,
            ComplexityClass::Linear,
        );
        assert!(!r.class_preserved);
        let r2 = make_report(
            10.0,
            f64::INFINITY,
            ComplexityClass::Linear,
            ComplexityClass::Linear,
        );
        assert!(!r2.class_preserved);
    }

    #[test]
    fn complexity_class_equality() {
        assert_eq!(ComplexityClass::Linear, ComplexityClass::Linear);
        assert_ne!(ComplexityClass::Linear, ComplexityClass::Quadratic);
        let c = ComplexityClass::Exponential;
        let d = c;
        assert_eq!(format!("{d:?}"), "Exponential");
        let mut set: std::collections::HashSet<ComplexityClass> = std::collections::HashSet::new();
        set.insert(ComplexityClass::Constant);
        set.insert(ComplexityClass::Constant);
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn contract_marker_shape() {
        assert!(contract_marker().starts_with("contract:"));
        assert!(contract_marker().ends_with("— OK"));
    }

    // ---- proptest properties ----

    proptest! {
        #[test]
        fn fused_chain_equals_hand_loop_for_all_inputs(xs in proptest::collection::vec(any::<i64>(), 0..64)) {
            prop_assert_eq!(fused_chain_sum(&xs), hand_loop_sum(&xs));
            prop_assert!(results_equal(&xs));
        }

        #[test]
        fn weight_class_boundary_none_for_random_missing(mut sorted in proptest::collection::vec(any::<i64>(), 0..256)) {
            sorted.sort_unstable();
            // Find a value that is definitely NOT in the slice.
            let probe: i64 = i64::MIN;
            if !sorted.contains(&probe) {
                let (idx, ops) = weight_class_boundary(&sorted, probe);
                prop_assert_eq!(idx, None);
                let n = sorted.len() as u64;
                let log2: u32 = if n == 0 { 0 } else { u64::BITS - n.leading_zeros() };
                prop_assert!(ops <= log2 + 1);
            }
        }

        #[test]
        fn weight_class_boundary_finds_when_present(mut xs in proptest::collection::vec(any::<i64>(), 1..256)) {
            xs.sort_unstable();
            let target = xs[xs.len() / 2];
            let (idx, _) = weight_class_boundary(&xs, target);
            prop_assert!(idx.is_some());
            if let Some(i) = idx {
                prop_assert_eq!(xs[i], target);
            }
        }

        #[test]
        fn unbeaten_count_is_total_over_any_slice(xs in proptest::collection::vec(0u32..100, 0..256)) {
            let c = unbeaten_count(&xs);
            // Count is bounded by slice length.
            prop_assert!(c as usize <= xs.len());
        }

        #[test]
        fn rank_then_total_matches_naive_sum(xs in proptest::collection::vec(-10_000_i64..10_000, 0..64)) {
            let direct: i64 = xs.iter().fold(0_i64, |a, b| a.wrapping_add(*b));
            prop_assert_eq!(rank_then_total(&xs), direct);
        }

        #[test]
        fn rivalry_pair_count_equals_n_squared(n in 0u32..32) {
            let count = rivalry_pair_count(n);
            prop_assert_eq!(count, u64::from(n) * u64::from(n));
        }
    }
}
