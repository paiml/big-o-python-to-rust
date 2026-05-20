#![allow(clippy::print_stdout, clippy::print_stderr)]
//! M4 demo: print cache-miss estimates for fighter-roster scans,
//! Brent-theorem parallel speedup curves for fight prediction, the I/O
//! cost of sorting a 1M-fight history database, and the generator →
//! iterator translation (lesson 4.1.1).

use m4_systems::{
    cache_misses, contract_marker, count_winners, external_sort_io_cost, parallel_speedup,
    CacheModel, WorkDepth,
};

fn main() {
    println!("M4 · Cache + parallel complexity (UFC/BJJ edition)\n");

    let cache = CacheModel {
        lines: 1024,
        line_size: 64,
    };
    println!(
        "Cache miss estimates ({}-byte lines, fighter roster):",
        cache.line_size
    );
    for n in [64_u32, 256, 1024, 65_536] {
        println!("  n = {n:>6} -> {:>6} misses", cache_misses(n, cache));
    }

    let wd = WorkDepth {
        work: 1024,
        depth: 8,
    };
    println!(
        "\nBrent-theorem speedup (rayon parallel fight prediction, work={}, depth={}):",
        wd.work, wd.depth
    );
    for p in [1_u32, 8, 256] {
        let s = parallel_speedup(wd, p);
        println!("  p = {p:>4} -> {s:>6.2}x");
    }

    println!("\nExternal-sort I/O cost (1M-fight history database):");
    let cost = external_sort_io_cost(1 << 20, 4096, 64);
    println!("  n=1M, M=4096, B=64 -> {cost} I/O ops");

    println!("\ngenerator -> iterator (streaming fight history):");
    let history: &[(&str, &str)] = &[
        ("Khabib", "McGregor"),
        ("Adesanya", "Pereira"),
        ("Jones", "Cormier"),
    ];
    let winners = count_winners(history);
    println!("  count_winners over 3 fights = {winners}");

    eprintln!("{}", contract_marker());
}
