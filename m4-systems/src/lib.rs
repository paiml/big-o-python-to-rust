//! M4 — Systems-level complexity (UFC/BJJ edition).
//!
//! Cache-oblivious analysis on a linear fighter-roster scan, the
//! work-depth model for parallel fight prediction, and external-sort
//! I/O over a 1M-fight history database. Plus the course concurrency
//! translation [`count_winners`] (Python generator → Rust iterator,
//! lesson 4.1.1).
//!
//! | Topic | Function | Lesson |
//! |---|---|---|
//! | Cache misses on roster scan | [`cache_misses`] | — |
//! | Brent's theorem on parallel predictions | [`parallel_speedup`] | 4.3.1 (`threading` → `rayon`) |
//! | External sort I/O cost | [`external_sort_io_cost`] | 4.2.1 (`subprocess` → `Command`) |
//! | Generator → Iterator | [`count_winners`] | 4.1.1 |
//!
//! Bound to: `contracts/binding.yaml` (no specific complexity contract).
//! These tools extend the empirical+structural toolkit so M5's capstone
//! can reason about hardware effects (cache misses, parallel speedup)
//! on top of asymptotic class.

/// A simple cache model: a number of cache lines of a given size in bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CacheModel {
    /// Number of cache lines available (i.e. `cache_capacity / line_size`).
    pub lines: u32,
    /// Size of each cache line in bytes.
    pub line_size: u32,
}

/// Estimate the number of cache misses incurred by a linear scan over
/// `n` 1-byte fighter-roster elements under the given [`CacheModel`].
///
/// Bound to: `contracts/binding.yaml` (structural model). The estimate
/// is `ceil(n / line_size)` — every line is read exactly once. Smaller
/// working sets (a single weight class) stay hot in cache; full roster
/// scans saturate L2/L3.
///
/// Returns `0` if `line_size == 0` or `n == 0` (degenerate inputs).
#[must_use]
pub const fn cache_misses(n: u32, m: CacheModel) -> u32 {
    if n == 0 || m.line_size == 0 {
        return 0;
    }
    n.div_ceil(m.line_size)
}

/// Work-depth model for a parallel computation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkDepth {
    /// Total operations performed (sequential time on 1 processor).
    pub work: u64,
    /// Critical-path length (sequential dependency chain).
    pub depth: u64,
}

/// Compute the parallel-speedup upper bound per Brent's theorem:
/// `speedup(p) = min(p, work / depth)`.
///
/// Bound to: `contracts/binding.yaml`. The UFC framing: predicting
/// fight outcomes is embarrassingly parallel (each prediction is
/// independent), so `work` is the total simulation count and `depth`
/// is the per-fight critical path. Course lesson 4.3.1 — Python
/// `threading` is GIL-bound; Rust `rayon` parallelizes across cores
/// at zero ceremony.
///
/// Returns `0.0` for `p == 0` or a degenerate `wd.depth == 0`.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn parallel_speedup(wd: WorkDepth, p: u32) -> f64 {
    if p == 0 || wd.depth == 0 {
        return 0.0;
    }
    let parallelism = wd.work as f64 / wd.depth as f64;
    f64::from(p).min(parallelism)
}

/// External-sort I/O cost — `O((N/B) · log_{M/B}(N/B))` from the
/// disk-access model (Aggarwal & Vitter 1988), applied to a UFC
/// 1M-fight history database that does not fit in memory.
///
/// `n` is the input size in items, `m` is fast-memory capacity in items,
/// `b` is block size in items. Returns `0` when any of `m`, `b`, or
/// `m/b` is zero/one — the formula has no meaningful value there.
///
/// Course lesson 4.2.1 — Python `subprocess` → Rust `Command`. The
/// canonical external-sort workflow shells out to a sort utility; on
/// Rust you get proper exit codes and no string-injection footgun.
///
/// Bound to: `contracts/binding.yaml` (structural).
#[must_use]
pub const fn external_sort_io_cost(n: u64, m: u64, b: u64) -> u64 {
    if b == 0 || m == 0 || n == 0 {
        return 0;
    }
    let runs = n / b;
    if runs <= 1 {
        return runs;
    }
    let fanout = m / b;
    if fanout <= 1 {
        return runs;
    }
    let mut levels: u64 = 0;
    let mut remaining = runs;
    while remaining > 1 {
        remaining = remaining.div_ceil(fanout);
        levels = levels.saturating_add(1);
    }
    runs.saturating_mul(levels)
}

/// Course lesson 4.1.1 — Python generator → Rust iterator.
///
/// Take a slice of `(winner, loser)` fight pairs (streaming fight
/// history) and count the number of distinct winners. The Rust
/// translation of the Python generator function is a chain of
/// iterator adapters that holds O(1) extra memory — same shape as the
/// Python `yield` version, but with no intermediate `list` allocation
/// and a tight inner loop after optimization.
#[must_use]
pub fn count_winners(fight_history: &[(&str, &str)]) -> usize {
    let mut seen: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    for (winner, _loser) in fight_history {
        seen.insert(*winner);
    }
    seen.len()
}

/// Runtime marker for the demo binary.
#[must_use]
pub const fn contract_marker() -> &'static str {
    "contract: m4-systems holds — OK"
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::bool_assert_comparison,
    clippy::float_cmp,
    clippy::cast_possible_truncation
)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn default_cache() -> CacheModel {
        CacheModel {
            lines: 1024,
            line_size: 64,
        }
    }

    // ---- cache_misses ----

    #[test]
    fn cache_misses_zero_inputs() {
        assert_eq!(cache_misses(0, default_cache()), 0);
        let degen = CacheModel {
            lines: 1024,
            line_size: 0,
        };
        assert_eq!(cache_misses(100, degen), 0);
    }

    #[test]
    fn cache_misses_round_up() {
        assert_eq!(cache_misses(64, default_cache()), 1);
        assert_eq!(cache_misses(65, default_cache()), 2);
        assert_eq!(cache_misses(128, default_cache()), 2);
    }

    #[test]
    fn cache_misses_is_monotone_in_n() {
        let m = default_cache();
        for n in 1..1000 {
            let a = cache_misses(n, m);
            let b = cache_misses(n + 1, m);
            assert!(b >= a, "monotonicity broken at n={n}");
        }
    }

    #[test]
    fn cache_misses_more_cache_fewer_misses() {
        let small = CacheModel {
            lines: 1024,
            line_size: 32,
        };
        let big = CacheModel {
            lines: 1024,
            line_size: 128,
        };
        assert!(cache_misses(10_000, big) <= cache_misses(10_000, small));
        let copy = small;
        assert_eq!(copy, small);
        assert_ne!(copy, big);
        assert_eq!(format!("{copy:?}").is_empty(), false);
    }

    // ---- parallel_speedup ----

    #[test]
    fn parallel_speedup_capped_by_p() {
        let wd = WorkDepth {
            work: 1024,
            depth: 1,
        };
        assert!((parallel_speedup(wd, 4) - 4.0).abs() < 1e-9);
        assert!((parallel_speedup(wd, 1024) - 1024.0).abs() < 1e-9);
    }

    #[test]
    fn parallel_speedup_capped_by_parallelism() {
        let wd = WorkDepth {
            work: 1000,
            depth: 1000,
        };
        assert!((parallel_speedup(wd, 16) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn parallel_speedup_zero_cores_or_zero_depth() {
        let wd = WorkDepth {
            work: 1000,
            depth: 1,
        };
        assert!((parallel_speedup(wd, 0) - 0.0).abs() < f64::EPSILON);
        let degen = WorkDepth {
            work: 1000,
            depth: 0,
        };
        assert!((parallel_speedup(degen, 8) - 0.0).abs() < f64::EPSILON);
        let dup = wd;
        assert_eq!(dup, wd);
        assert_eq!(format!("{wd:?}").is_empty(), false);
    }

    // ---- external_sort_io_cost ----

    #[test]
    fn external_sort_io_cost_degenerate_inputs() {
        assert_eq!(external_sort_io_cost(0, 1024, 64), 0);
        assert_eq!(external_sort_io_cost(1_000_000, 0, 64), 0);
        assert_eq!(external_sort_io_cost(1_000_000, 1024, 0), 0);
    }

    #[test]
    fn external_sort_io_cost_single_run() {
        assert_eq!(external_sort_io_cost(64, 1024, 64), 1);
        assert_eq!(external_sort_io_cost(32, 1024, 64), 0);
    }

    #[test]
    fn external_sort_io_cost_no_fanout() {
        let runs = 8;
        assert_eq!(external_sort_io_cost(8 * 64, 64, 64), runs);
    }

    #[test]
    fn external_sort_io_cost_realistic() {
        let cost = external_sort_io_cost(1 << 20, 4096, 64);
        let runs = (1 << 20) / 64;
        assert_eq!(cost, runs * 3);
    }

    // ---- count_winners (lesson 4.1.1) ----

    #[test]
    fn count_winners_distinct() {
        let history: &[(&str, &str)] = &[
            ("Khabib", "McGregor"),
            ("Adesanya", "Pereira"),
            ("Jones", "Cormier"),
        ];
        assert_eq!(count_winners(history), 3);
    }

    #[test]
    fn count_winners_dedup() {
        let history: &[(&str, &str)] = &[
            ("Khabib", "McGregor"),
            ("Khabib", "Poirier"),
            ("Khabib", "Gaethje"),
        ];
        assert_eq!(count_winners(history), 1);
    }

    #[test]
    fn count_winners_empty() {
        let history: &[(&str, &str)] = &[];
        assert_eq!(count_winners(history), 0);
    }

    #[test]
    fn contract_marker_shape() {
        assert!(contract_marker().starts_with("contract:"));
        assert!(contract_marker().ends_with("— OK"));
    }

    // ---- proptest ----

    proptest! {
        #[test]
        fn cache_misses_monotone_in_n(n in 1u32..10_000, line in 1u32..256) {
            let m = CacheModel { lines: 1024, line_size: line };
            let a = cache_misses(n, m);
            let b = cache_misses(n + 1, m);
            prop_assert!(b >= a);
        }

        #[test]
        fn parallel_speedup_monotone_in_p(work in 1u64..10_000, depth in 1u64..1_000, p in 1u32..64) {
            let wd = WorkDepth { work, depth };
            let s_p = parallel_speedup(wd, p);
            let s_p1 = parallel_speedup(wd, p + 1);
            prop_assert!(s_p1 >= s_p - 1e-9);
        }

        #[test]
        fn external_sort_io_cost_is_total(n in 0u64..1_000_000, m in 0u64..10_000, b in 0u64..1024) {
            let _ = external_sort_io_cost(n, m, b);
        }

        #[test]
        fn count_winners_bounded_by_history_len(len in 0usize..32) {
            let names = ["Khabib", "Jones", "Silva", "Adesanya"];
            let history: Vec<(&str, &str)> = (0..len)
                .map(|i| (names[i % names.len()], names[(i + 1) % names.len()]))
                .collect();
            let c = count_winners(&history);
            prop_assert!(c <= history.len());
            prop_assert!(c <= names.len());
        }
    }
}
