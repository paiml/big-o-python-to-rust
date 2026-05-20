//! M5 — Capstone: the 5-step playbook on the top-K hot fighters
//! problem (UFC/BJJ edition).
//!
//! Given a slice of `(name, win_streak)` pairs, return the K fighters
//! with the longest current streaks. Course lesson 5.1.1 — the
//! five-step playbook: **depyler transpile → inspect & idiomatize →
//! author contract YAML → criterion bench → proptest property**.
//!
//! | Function | Strategy | Complexity | Bound contract |
//! |---|---|---|---|
//! | [`top_k_naive`]  | nested pairwise max | O(n²) | `complexity-quadratic-v1` |
//! | [`top_k_sort`]   | sort then take | O(n log n) | `complexity-linearithmic-v1` |
//! | [`top_k_heap`]   | min-heap of size K | O(n log K) which for fixed K is O(n) | `complexity-linear-v1` |
//!
//! All three must produce **equivalent** outputs on the same input
//! (proptest exercises that invariant; the unit tests pin a known
//! UFC roster). Lesson 5.2.1 — [`should_translate`] gives the
//! when-NOT-to-translate heuristic: skip Python→Rust when the hot
//! path is already C-accelerated, the bottleneck is I/O, or the team
//! cannot own Rust.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use m2_empirical::ComplexityClass;

/// O(n²) top-K via repeated linear scans of the fighter roster.
///
/// Bound to: `contracts/complexity-quadratic-v1.yaml`. Picks the
/// longest-streak fighter, removes them, repeats — `k` linear scans
/// over a vector whose size starts at `n`.
#[must_use]
pub fn top_k_naive(fighters: &[(String, u64)], k: usize) -> Vec<(String, u64)> {
    if k == 0 || fighters.is_empty() {
        return Vec::new();
    }
    let mut remaining: Vec<(String, u64)> = fighters.to_vec();
    let mut out: Vec<(String, u64)> = Vec::with_capacity(k.min(remaining.len()));
    while !remaining.is_empty() && out.len() < k {
        let mut best_idx = 0_usize;
        for i in 1..remaining.len() {
            if remaining[i].1 > remaining[best_idx].1 {
                best_idx = i;
            }
        }
        out.push(remaining.swap_remove(best_idx));
    }
    out
}

/// O(n log n) top-K via full sort + take.
///
/// Bound to: `contracts/complexity-linearithmic-v1.yaml`.
#[must_use]
pub fn top_k_sort(fighters: &[(String, u64)], k: usize) -> Vec<(String, u64)> {
    if k == 0 || fighters.is_empty() {
        return Vec::new();
    }
    let mut buf: Vec<(String, u64)> = fighters.to_vec();
    buf.sort_by_key(|entry| Reverse(entry.1));
    buf.truncate(k);
    buf
}

/// O(n log K) top-K via a min-heap of size K — for fixed K this is
/// linear in n.
///
/// Bound to: `contracts/complexity-linear-v1.yaml` (with K fixed).
#[must_use]
pub fn top_k_heap(fighters: &[(String, u64)], k: usize) -> Vec<(String, u64)> {
    if k == 0 || fighters.is_empty() {
        return Vec::new();
    }
    let mut heap: BinaryHeap<Reverse<(u64, String)>> = BinaryHeap::with_capacity(k + 1);
    for (name, streak) in fighters {
        heap.push(Reverse((*streak, name.clone())));
        if heap.len() > k {
            heap.pop();
        }
    }
    let mut out: Vec<(String, u64)> = heap.into_iter().map(|Reverse((c, n))| (n, c)).collect();
    out.sort_by_key(|entry| Reverse(entry.1));
    out
}

/// Documented complexity class for each implementation, keyed by name.
///
/// Bound to: `contracts/binding.yaml` — used by the capstone demo
/// binary to print the contract each implementation answers to.
///
/// Returns [`ComplexityClass::Constant`] for any unknown implementation
/// name (the safe default, since unknown == "we cannot bound").
#[must_use]
pub fn complexity_class(impl_name: &str) -> ComplexityClass {
    match impl_name {
        "naive" => ComplexityClass::Quadratic,
        "sort" => ComplexityClass::Linearithmic,
        "heap" => ComplexityClass::Linear,
        _ => ComplexityClass::Constant,
    }
}

/// Return `true` if the multisets of win-streak counts in `a` and `b`
/// are equal.
///
/// Top-K outputs are equivalent up to permutation of ties on the
/// boundary; comparing the multiset of streaks is the cleanest
/// invariant that holds across all three implementations.
#[must_use]
pub fn count_multisets_equal(a: &[(String, u64)], b: &[(String, u64)]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut left: Vec<u64> = a.iter().map(|(_, c)| *c).collect();
    let mut right: Vec<u64> = b.iter().map(|(_, c)| *c).collect();
    left.sort_unstable();
    right.sort_unstable();
    left == right
}

/// Course lesson 5.2.1 — when NOT to translate Python → Rust.
///
/// Three skip signals fire as `true`:
///
/// 1. `c_accelerated` — the hot path is already C-accelerated (numpy,
///    pandas, scikit-learn). The inner loop is already native;
///    translating the orchestration won't move the needle.
/// 2. `io_bound` — a web scraper waiting on HTTP doesn't get faster
///    in Rust.
/// 3. `owns_python_only` — translation is a 1-2 quarter investment.
///    Spend it on bottlenecks, not on convenience.
///
/// Returns `true` only when none of the three signals fires — i.e.,
/// you have a real CPU bottleneck AND your team can own Rust.
#[must_use]
pub const fn should_translate(c_accelerated: bool, io_bound: bool, owns_python_only: bool) -> bool {
    !(c_accelerated || io_bound || owns_python_only)
}

/// Runtime marker for the capstone demo binary.
#[must_use]
pub const fn contract_marker() -> &'static str {
    "contract: m5-capstone holds — OK"
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::bool_assert_comparison
)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    /// The canonical UFC roster fixture used across the playbook tests.
    fn ufc_roster() -> Vec<(String, u64)> {
        vec![
            ("Khabib".to_owned(), 29),
            ("Silva".to_owned(), 16),
            ("Jones".to_owned(), 27),
            ("GSP".to_owned(), 13),
            ("Adesanya".to_owned(), 6),
            ("McGregor".to_owned(), 2),
            ("Diaz".to_owned(), 1),
            ("Volkanovski".to_owned(), 19),
            ("Cormier".to_owned(), 4),
            ("Holloway".to_owned(), 7),
        ]
    }

    #[test]
    fn three_impls_agree_on_ufc_fixture() {
        let xs = ufc_roster();
        for k in [0_usize, 1, 2, 3, 5, 7, 10, 12] {
            let n = top_k_naive(&xs, k);
            let s = top_k_sort(&xs, k);
            let h = top_k_heap(&xs, k);
            assert!(count_multisets_equal(&n, &s), "naive vs sort at k={k}");
            assert!(count_multisets_equal(&s, &h), "sort vs heap at k={k}");
            assert_eq!(n.len(), k.min(xs.len()));
        }
    }

    #[test]
    fn top_3_ufc_are_khabib_jones_volkanovski() {
        // The hand-checked answer: Khabib (29), Jones (27), Volkanovski (19).
        let xs = ufc_roster();
        let s = top_k_sort(&xs, 3);
        assert_eq!(s.len(), 3);
        let streaks: Vec<u64> = s.iter().map(|(_, c)| *c).collect();
        assert_eq!(streaks, vec![29, 27, 19]);
        let names: Vec<&str> = s.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, vec!["Khabib", "Jones", "Volkanovski"]);
    }

    #[test]
    fn empty_input() {
        let empty: Vec<(String, u64)> = Vec::new();
        assert!(top_k_naive(&empty, 5).is_empty());
        assert!(top_k_sort(&empty, 5).is_empty());
        assert!(top_k_heap(&empty, 5).is_empty());
    }

    #[test]
    fn zero_k() {
        let xs = ufc_roster();
        assert!(top_k_naive(&xs, 0).is_empty());
        assert!(top_k_sort(&xs, 0).is_empty());
        assert!(top_k_heap(&xs, 0).is_empty());
    }

    #[test]
    fn top_one_returns_max() {
        let xs = ufc_roster();
        let n = top_k_naive(&xs, 1);
        let s = top_k_sort(&xs, 1);
        let h = top_k_heap(&xs, 1);
        assert_eq!(n.len(), 1);
        assert_eq!(n[0].1, 29);
        assert_eq!(s[0].1, 29);
        assert_eq!(h[0].1, 29);
        assert_eq!(n[0].0, "Khabib");
    }

    #[test]
    fn top_k_with_ties_keeps_count_multiset() {
        // Three fighters all with streak 5.
        let xs = vec![
            ("Khabib".to_owned(), 5),
            ("Jones".to_owned(), 5),
            ("Silva".to_owned(), 5),
        ];
        let n = top_k_naive(&xs, 2);
        let s = top_k_sort(&xs, 2);
        let h = top_k_heap(&xs, 2);
        assert!(count_multisets_equal(&n, &s));
        assert!(count_multisets_equal(&s, &h));
        assert_eq!(n.len(), 2);
    }

    #[test]
    fn count_multisets_equal_basic() {
        let a = vec![("Khabib".to_owned(), 1), ("Jones".to_owned(), 2)];
        let b = vec![("Silva".to_owned(), 2), ("GSP".to_owned(), 1)];
        assert!(count_multisets_equal(&a, &b));
        let c = vec![("Khabib".to_owned(), 1)];
        assert!(!count_multisets_equal(&a, &c));
        let d = vec![("Khabib".to_owned(), 1), ("Jones".to_owned(), 3)];
        assert!(!count_multisets_equal(&a, &d));
    }

    #[test]
    fn complexity_class_lookup() {
        assert_eq!(complexity_class("naive"), ComplexityClass::Quadratic);
        assert_eq!(complexity_class("sort"), ComplexityClass::Linearithmic);
        assert_eq!(complexity_class("heap"), ComplexityClass::Linear);
        assert_eq!(complexity_class("unknown"), ComplexityClass::Constant);
    }

    #[test]
    fn should_translate_three_skip_signals() {
        // numpy hot path → skip.
        assert!(!should_translate(true, false, false));
        // I/O-bound scraper → skip.
        assert!(!should_translate(false, true, false));
        // No Rust capacity → skip.
        assert!(!should_translate(false, false, true));
        // Real CPU bottleneck + Rust capacity → translate.
        assert!(should_translate(false, false, false));
    }

    #[test]
    fn contract_marker_shape() {
        assert!(contract_marker().starts_with("contract:"));
        assert!(contract_marker().ends_with("— OK"));
    }

    proptest! {
        #[test]
        fn three_impls_agree_on_random_rosters(
            counts in proptest::collection::vec(0u64..1000, 0..64),
            k in 0usize..16,
        ) {
            let names = ["Khabib", "Jones", "Silva", "Adesanya", "GSP", "McGregor"];
            let fighters: Vec<(String, u64)> = counts
                .iter()
                .enumerate()
                .map(|(i, c)| (format!("{}#{i}", names[i % names.len()]), *c))
                .collect();
            let n = top_k_naive(&fighters, k);
            let s = top_k_sort(&fighters, k);
            let h = top_k_heap(&fighters, k);
            prop_assert!(count_multisets_equal(&n, &s));
            prop_assert!(count_multisets_equal(&s, &h));
            let expected_len = k.min(fighters.len());
            prop_assert_eq!(n.len(), expected_len);
            prop_assert_eq!(s.len(), expected_len);
            prop_assert_eq!(h.len(), expected_len);
        }

        #[test]
        fn top_k_sort_is_descending(
            counts in proptest::collection::vec(0u64..1000, 1..64),
            k in 1usize..16,
        ) {
            let fighters: Vec<(String, u64)> = counts
                .iter()
                .enumerate()
                .map(|(i, c)| (format!("fighter{i}"), *c))
                .collect();
            let s = top_k_sort(&fighters, k);
            for w in s.windows(2) {
                prop_assert!(w[0].1 >= w[1].1);
            }
        }

        #[test]
        fn should_translate_only_true_for_all_false(c in any::<bool>(), io in any::<bool>(), o in any::<bool>()) {
            let r = should_translate(c, io, o);
            prop_assert_eq!(r, !c && !io && !o);
        }
    }
}
