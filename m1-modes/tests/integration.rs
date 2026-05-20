#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Integration test: confirm the three modes of proof speak the same
//! language from a downstream caller's perspective. UFC roster framing.

use m1_modes::{
    applicable_mode, classify_recurrence, empirical_doubling_ratio, formal_status,
    is_well_formed_claim, speedup, transpile_class_preserved, MasterCase, ProofStatus,
};

#[test]
fn empirical_and_structural_agree_on_bracket_build() {
    // Synthetic O(n) sample → max consecutive ratio near 2.0.
    let ratio = empirical_doubling_ratio(&[1.0, 2.0, 4.0, 8.0]);
    assert!((ratio - 2.0).abs() < 1e-9);
    // Tournament-bracket recurrence (a=2, b=2, exp=1) lands in Case 2.
    assert_eq!(classify_recurrence(2.0, 2.0, 1.0), MasterCase::Case2);
}

#[test]
fn formal_status_is_distinct_from_other_modes() {
    let f = formal_status();
    assert_eq!(f, ProofStatus::Formal);
    assert_ne!(f, ProofStatus::Empirical);
    assert_ne!(f, ProofStatus::Structural);
}

#[test]
fn falsifiable_shape_and_depyler_receipt() {
    assert!(is_well_formed_claim(&[1024, 2048, 4096], &[2.0, 2.0]));
    assert!(transpile_class_preserved("O(n)", "O(n)"));
    assert!((speedup(40.0, 10.0) - 4.0).abs() < 1e-9);
}

#[test]
fn applicable_mode_dispatches_three_ways() {
    assert_eq!(
        applicable_mode("complexity-linear-v1"),
        ProofStatus::Empirical
    );
    assert_eq!(
        applicable_mode("master-theorem-case-2"),
        ProofStatus::Formal
    );
    assert_eq!(
        applicable_mode("banker-amortized-push"),
        ProofStatus::Structural
    );
}
