#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::float_cmp
)]
//! Integration test: confirm structural proofs and correctness
//! translations behave end-to-end on the UFC roster.

use m2_empirical::ComplexityClass;
use m3_structural::{
    amortized_vec_push_cost, book_fight, brackets_closed_form, classify, find_opponent,
    schedule_safe, BookingError, MasterCase,
};

#[test]
fn canonical_recurrence_table() {
    let table = [
        (
            "tournament bracket",
            2.0,
            2.0,
            ComplexityClass::Linear,
            MasterCase::Case2,
        ),
        (
            "binary search",
            1.0,
            2.0,
            ComplexityClass::Constant,
            MasterCase::Case2,
        ),
        (
            "matrix mult naive",
            8.0,
            2.0,
            ComplexityClass::Quadratic,
            MasterCase::Case1,
        ),
        (
            "root-dominated",
            2.0,
            2.0,
            ComplexityClass::Quadratic,
            MasterCase::Case3,
        ),
    ];
    for (label, a, b, fc, expected) in table {
        let got = classify(a, b, fc);
        assert_eq!(got, expected, "{label}: got {got:?}, expected {expected:?}");
    }
}

#[test]
fn brackets_closed_form_agrees_with_known_values() {
    assert_eq!(brackets_closed_form(10).round() as i64, 55);
    assert_eq!(brackets_closed_form(20).round() as i64, 6765);
}

#[test]
fn amortized_push_is_constant_per_op() {
    let a = amortized_vec_push_cost(10_000);
    assert!((a.amortized_per_op - 3.0).abs() < 1e-9);
}

#[test]
fn optional_to_option_translation() {
    let roster = ["Khabib", "McGregor", "Adesanya"];
    assert_eq!(find_opponent(&roster, "Khabib"), Some("McGregor"));
    let empty: [&str; 0] = [];
    assert_eq!(find_opponent(&empty, "anyone"), None);
}

#[test]
fn try_except_to_result_translation() {
    let roster = ["Khabib", "McGregor"];
    assert!(book_fight(&roster, "Khabib", "McGregor").is_ok());
    assert_eq!(
        book_fight(&roster, "Khabib", "Khabib"),
        Err(BookingError::CannotFightSelf)
    );
}

#[test]
fn mutable_default_to_ownership_translation() {
    let a = schedule_safe("a", None);
    let b = schedule_safe("b", None);
    // The Python bug would have b == ["a", "b"]; ownership prevents it.
    assert_eq!(a, vec!["a".to_owned()]);
    assert_eq!(b, vec!["b".to_owned()]);
}
