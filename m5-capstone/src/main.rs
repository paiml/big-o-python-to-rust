#![allow(clippy::print_stdout, clippy::print_stderr)]
//! M5 capstone demo: run all three top-K hot-fighter implementations on
//! the canonical UFC roster, print the resulting top-3, and confirm the
//! count multisets agree. Then demonstrate the lesson 5.2.1 when-not-to-
//! translate signals.

use m5_capstone::{
    complexity_class, contract_marker, count_multisets_equal, should_translate, top_k_heap,
    top_k_naive, top_k_sort,
};

fn main() {
    println!("M5 — Top-K hot fighters, three ways (the 5-step playbook)\n");

    let roster: Vec<(String, u64)> = vec![
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
    ];
    let k = 3;
    let n = top_k_naive(&roster, k);
    let s = top_k_sort(&roster, k);
    let h = top_k_heap(&roster, k);

    let print_one = |label: &str, v: &[(String, u64)]| {
        let class = complexity_class(label);
        print!("  top-{k} ({label}, {class:?})\t-> [");
        for (i, (name, streak)) in v.iter().enumerate() {
            if i > 0 {
                print!(", ");
            }
            print!("{name}:{streak}");
        }
        println!("]");
    };
    print_one("naive", &n);
    print_one("sort", &s);
    print_one("heap", &h);

    let equal_ns = count_multisets_equal(&n, &s);
    let equal_sh = count_multisets_equal(&s, &h);
    println!("\n  count-multiset equality: naive==sort: {equal_ns}, sort==heap: {equal_sh}");

    println!("\n  when-not-to-translate signals:");
    let label_for = |b: bool| if b { "TRANSLATE" } else { "skip" };
    println!(
        "    numpy hot path?         {}",
        label_for(should_translate(true, false, false))
    );
    println!(
        "    HTTP-bound scraper?     {}",
        label_for(should_translate(false, true, false))
    );
    println!(
        "    no Rust capacity?       {}",
        label_for(should_translate(false, false, true))
    );
    println!(
        "    real CPU bottleneck?    {}",
        label_for(should_translate(false, false, false))
    );

    eprintln!("{}", contract_marker());
}
