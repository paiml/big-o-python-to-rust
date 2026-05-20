#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Integration test: monotonicity properties hold from outside the
//! crate. UFC framing.

use m4_systems::{
    cache_misses, count_winners, external_sort_io_cost, parallel_speedup, CacheModel, WorkDepth,
};

#[test]
fn more_cores_never_hurt() {
    let wd = WorkDepth {
        work: 10_000,
        depth: 10,
    };
    let mut prev = parallel_speedup(wd, 1);
    for p in 2_u32..32 {
        let s = parallel_speedup(wd, p);
        assert!(s >= prev - 1e-9, "speedup non-monotone at p={p}");
        prev = s;
    }
}

#[test]
fn bigger_lines_never_hurt() {
    let n = 1024_u32;
    let small = CacheModel {
        lines: 64,
        line_size: 16,
    };
    let big = CacheModel {
        lines: 64,
        line_size: 256,
    };
    assert!(cache_misses(n, big) <= cache_misses(n, small));
}

#[test]
fn external_sort_costs_reasonable() {
    let cost = external_sort_io_cost(1 << 20, 4096, 64);
    assert!(cost > 0);
    assert!(cost < (1 << 20) * 10);
}

#[test]
fn count_winners_distinct_set() {
    let history: &[(&str, &str)] = &[
        ("Khabib", "McGregor"),
        ("Khabib", "Poirier"),
        ("Adesanya", "Pereira"),
    ];
    assert_eq!(count_winners(history), 2);
}
