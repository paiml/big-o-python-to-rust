#![allow(clippy::print_stdout, clippy::print_stderr)]
//! M1 demo binary: tour the three modes of proof on a UFC roster.
//!
//! Prints the empirical ratio on a synthetic O(n) roster scan, the
//! structural master-theorem verdict on the tournament-bracket recurrence,
//! the formal proof status, the falsifiable-claim shape check, and the
//! depyler transpile-preservation receipt. Finishes with the runtime
//! marker line that `make demo` greps for.

use m1_modes::{
    applicable_mode, classify_recurrence, contract_marker, empirical_doubling_ratio, formal_status,
    is_well_formed_claim, speedup, transpile_class_preserved, MasterCase,
};

fn main() {
    println!("M1 · Three modes of complexity proof (UFC/BJJ edition)\n");

    // Empirical: synthetic O(n) sample — doubling roster sizes, doubling times.
    let synthetic_on = [1.0_f64, 2.0, 4.0, 8.0, 16.0, 32.0];
    let r = empirical_doubling_ratio(&synthetic_on);
    println!("  empirical    : max doubling ratio = {r:.3} (expected ~ 2.0 for O(n))");

    // Structural: tournament-bracket recurrence T(n) = 2 T(n/2) + Θ(n), case 2.
    let bracket = classify_recurrence(2.0, 2.0, 1.0);
    let bracket_is_case_2 = matches!(bracket, MasterCase::Case2);
    println!("  structural   : tournament-bracket recurrence in case 2? {bracket_is_case_2}");

    // Formal: status as data; route empirical-class contracts vs formal-class.
    let f = formal_status();
    println!("  formal       : status = {f:?}");
    println!(
        "  formal       : applicable_mode(\"master-theorem-case-2\") = {:?}",
        applicable_mode("master-theorem-case-2")
    );

    // Lesson 1.2.1 — falsifiable claim shape.
    let wf = is_well_formed_claim(&[1024, 2048, 4096, 8192], &[2.01, 1.99, 2.02]);
    println!("  falsifiable  : well-formed claim shape = {wf}");

    // Lesson 1.3.1 — depyler transpile-preservation receipt.
    let preserved = transpile_class_preserved("O(n)", "O(n)");
    let speed = speedup(40.0, 10.0);
    println!("  depyler      : class preserved = {preserved}, expected speedup = {speed:.2}x");

    eprintln!("{}", contract_marker());
}
