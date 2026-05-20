#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Integration test: the three top-K implementations agree on a UFC
//! roster fixture that exercises ties, zero streaks, and k > n.

use m5_capstone::{
    complexity_class, count_multisets_equal, should_translate, top_k_heap, top_k_naive, top_k_sort,
};

fn fixture() -> Vec<(String, u64)> {
    vec![
        ("Diaz".to_owned(), 0),
        ("Jones".to_owned(), 9),
        ("Silva".to_owned(), 9),
        ("Adesanya".to_owned(), 3),
        ("Khabib".to_owned(), 9),
    ]
}

#[test]
fn three_impls_agree_with_ties() {
    let xs = fixture();
    for k in 0..=10 {
        let n = top_k_naive(&xs, k);
        let s = top_k_sort(&xs, k);
        let h = top_k_heap(&xs, k);
        assert!(count_multisets_equal(&n, &s), "naive vs sort at k={k}");
        assert!(count_multisets_equal(&s, &h), "sort vs heap at k={k}");
    }
}

#[test]
fn class_lookup_lines_up_with_implementation_strategy() {
    assert_eq!(
        complexity_class("naive"),
        m2_empirical::ComplexityClass::Quadratic
    );
    assert_eq!(
        complexity_class("sort"),
        m2_empirical::ComplexityClass::Linearithmic
    );
    assert_eq!(
        complexity_class("heap"),
        m2_empirical::ComplexityClass::Linear
    );
}

#[test]
fn should_translate_skip_signals() {
    assert!(!should_translate(true, false, false));
    assert!(!should_translate(false, true, false));
    assert!(!should_translate(false, false, true));
    assert!(should_translate(false, false, false));
}
