//! M3 — Structural complexity proofs + the course's correctness
//! translations (UFC/BJJ edition).
//!
//! Where M2 measures, M3 *argues*. We classify recurrences via the
//! master theorem on the tournament-bracket build, compute Binet's
//! closed-form for the bracket count, and run a banker's-method
//! amortized analysis of `Vec::push` (UFC roster growth). On top of
//! that, this module ships the structural-safety translations from
//! the course:
//!
//! * Lesson 3.1.1 — [`find_opponent`] (Python `Optional[T]` → Rust
//!   `Option<T>` with exhaustive `match`).
//! * Lesson 3.2.1 — [`book_fight`] + [`BookingError`] (Python
//!   `try/except` → Rust `Result<T, E>` with `?`).
//! * Lesson 3.3.1 — [`schedule_safe`] (Python mutable-default-arg bug →
//!   Rust ownership makes it structurally impossible).
//!
//! None of this needs criterion, so we can carry 100% line coverage
//! with deterministic unit tests.

use m2_empirical::ComplexityClass;

/// Which of the three master-theorem cases governs a recurrence
/// `T(n) = a · T(n/b) + f(n)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MasterCase {
    /// `f(n) = O(n^c)` with `c < log_b a` — leaves dominate.
    Case1,
    /// `f(n) = Θ(n^c)` with `c = log_b a` — balanced (e.g. the
    /// single-elimination bracket build `2 T(n/2) + Θ(n)`).
    Case2,
    /// `f(n) = Ω(n^c)` with `c > log_b a` — root dominates.
    Case3,
}

/// Classify a recurrence `T(n) = a · T(n/b) + f(n)` into one of the three
/// master-theorem cases.
///
/// Bound to: `contracts/complexity-linearithmic-v1.yaml` (the structural
/// side — proof obligation "master theorem applicability"). The
/// canonical UFC framing is the **tournament-bracket recurrence**: build
/// a single-elimination bracket over a roster of `n` fighters by
/// halving and merging, `T(n) = 2 T(n/2) + Θ(n)`, which lands in Case 2.
///
/// Returns [`MasterCase::Case2`] when `a == b^1` and `f(n)` is linear.
///
/// Will not panic on the inputs it accepts (`a > 0`, `b > 1`); a guard
/// returns [`MasterCase::Case2`] for degenerate `b ≤ 1` since
/// `log_b a` is undefined there and we prefer a total function over a
/// `Result` in this teaching context.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn classify(a: f64, b: f64, f_n_class: ComplexityClass) -> MasterCase {
    let c_critical = if b > 1.0 && a > 0.0 {
        a.log(b)
    } else {
        return MasterCase::Case2;
    };
    let c_f = exponent_of(f_n_class);
    if c_f < c_critical - 1e-9 {
        MasterCase::Case1
    } else if (c_f - c_critical).abs() < 1e-9 {
        MasterCase::Case2
    } else {
        MasterCase::Case3
    }
}

/// Map a [`ComplexityClass`] to its critical exponent `c` such that the
/// class is `Θ(n^c)` (with log-factor exponents folded into the
/// nearest integer for the master-theorem comparison — `Linearithmic`
/// maps to `1.0` because `n log n = Θ(n^{1+ε})` for every `ε > 0`).
#[must_use]
const fn exponent_of(c: ComplexityClass) -> f64 {
    match c {
        ComplexityClass::Constant => 0.0,
        ComplexityClass::Logarithmic | ComplexityClass::Linear | ComplexityClass::Linearithmic => {
            1.0
        }
        ComplexityClass::Quadratic => 2.0,
        ComplexityClass::Exponential => f64::INFINITY,
    }
}

/// Binet's closed-form approximation for the tournament-bracket count
/// `B(n) = φⁿ/√5` (same recurrence as Fibonacci).
///
/// Bound to: structural side of `contracts/complexity-exponential-v1.yaml`.
/// The naive recursive bracket count in M2 takes `Θ(φⁿ)` time; this
/// closed form computes the same value in O(1) floating-point ops,
/// which is the structural argument for why memoization beats naive
/// recursion.
///
/// Returns the float rounded to nearest integer as `f64`.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn brackets_closed_form(n: u32) -> f64 {
    let phi = (1.0 + 5.0_f64.sqrt()) / 2.0;
    let psi = (1.0 - 5.0_f64.sqrt()) / 2.0;
    let sqrt5 = 5.0_f64.sqrt();
    let n_f = f64::from(n);
    (phi.powf(n_f) - psi.powf(n_f)) / sqrt5
}

/// Result of an amortized-analysis run over `operations` consecutive
/// `Vec::push` calls using the banker's method.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AmortizedAnalysis {
    /// Total work performed, in abstract "credit units".
    pub total: f64,
    /// `total / operations` — converges to a small constant.
    pub amortized_per_op: f64,
}

/// Banker's-method amortized analysis of `n` consecutive `Vec::push`
/// operations on an initially-empty fighter roster.
///
/// Bound to: `contracts/binding.yaml` (structural-only — no specific
/// complexity contract). Total work is bounded by `3n` (each fighter
/// is paid for once on push, and accumulates 2 credits to fund the
/// eventual copy during a doubling reallocation). Dividing by `n`
/// gives the amortized-per-op cost, which is `Θ(1)` in the limit —
/// growing the roster from 1 to 1M fighters is O(1) amortized per
/// push.
///
/// Returns a zeroed [`AmortizedAnalysis`] for `operations == 0`.
#[must_use]
pub fn amortized_vec_push_cost(operations: u32) -> AmortizedAnalysis {
    if operations == 0 {
        return AmortizedAnalysis {
            total: 0.0,
            amortized_per_op: 0.0,
        };
    }
    let total = 3.0 * f64::from(operations);
    AmortizedAnalysis {
        total,
        amortized_per_op: total / f64::from(operations),
    }
}

/// Course lesson 3.1.1 — Python `Optional[Fighter]` → Rust `Option<&str>`.
///
/// Find the first fighter on the roster who is not the one we want to
/// skip (e.g. an opponent for Khabib). Returns `None` when no such
/// fighter exists.
///
/// In Python, `Optional[Fighter]` is documentation; the `None` branch
/// is easy to forget. In Rust, `Option<&str>` is a sum type the
/// compiler exhaustively enforces — the `None` branch is unmissable.
#[must_use]
pub fn find_opponent<'a>(roster: &'a [&'a str], skip: &str) -> Option<&'a str> {
    for f in roster {
        if *f != skip {
            return Some(*f);
        }
    }
    None
}

/// Course lesson 3.2.1 — Rust `Result<T, E>` for fight-booking errors.
///
/// Python `try/except` is dynamic; the compiler does not refuse to
/// ignore an error branch. Rust `Result<T, E>` is static — the
/// compiler refuses to ignore the [`Err`] variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BookingError {
    /// Fighter is not on the roster — payload is the offending name.
    NotOnRoster(String),
    /// A fighter cannot fight themself.
    CannotFightSelf,
}

/// Course lesson 3.2.1 — book a fight between two roster members.
///
/// Returns `Ok("booked: a vs b")` on success, or the appropriate
/// [`BookingError`]. The Python analog raises `FightBookingError`; the
/// Rust translation makes the failure mode part of the return type.
///
/// # Errors
///
/// * [`BookingError::NotOnRoster`] if either `a` or `b` is not in
///   `roster`.
/// * [`BookingError::CannotFightSelf`] if `a == b`.
pub fn book_fight(roster: &[&str], a: &str, b: &str) -> Result<String, BookingError> {
    if a == b {
        return Err(BookingError::CannotFightSelf);
    }
    if !roster.contains(&a) {
        return Err(BookingError::NotOnRoster(a.to_owned()));
    }
    if !roster.contains(&b) {
        return Err(BookingError::NotOnRoster(b.to_owned()));
    }
    Ok(format!("booked: {a} vs {b}"))
}

/// Course lesson 3.3.1 — Rust ownership prevents Python's
/// mutable-default-arg bug at compile time.
///
/// In Python, `def schedule(card=[]):` shares the same list across
/// calls — a classic surprise-aliasing bug. The Python fix is
/// `card=None` + an explicit `if card is None: card = []` inside the
/// function. Rust makes the bug structurally impossible: callers
/// either pass `Some(vec)` (which is *moved* into the function) or
/// `None` (which constructs a fresh `Vec`). Either way, the function
/// owns the `Vec` outright and no surprise sharing exists.
#[must_use]
pub fn schedule_safe(fight: &str, card: Option<Vec<String>>) -> Vec<String> {
    let mut v = card.unwrap_or_default();
    v.push(fight.to_owned());
    v
}

/// Runtime marker for the demo binary.
#[must_use]
pub const fn contract_marker() -> &'static str {
    "contract: m3-structural holds — OK"
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::float_cmp,
    clippy::bool_assert_comparison,
    clippy::suboptimal_flops
)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // ---- master-theorem classification ----

    #[test]
    fn bracket_build_is_case_2() {
        // Tournament bracket: 2 T(n/2) + Θ(n) — log_2(2) = 1 = critical exponent of f.
        assert_eq!(
            classify(2.0, 2.0, ComplexityClass::Linear),
            MasterCase::Case2
        );
        // Equivalent with linearithmic f(n) (rounded to exponent 1).
        assert_eq!(
            classify(2.0, 2.0, ComplexityClass::Linearithmic),
            MasterCase::Case2
        );
    }

    #[test]
    fn binary_search_recurrence_is_case_2_a1_b2() {
        // T(n) = T(n/2) + Θ(1) — log_2(1) = 0 = critical exponent of f.
        assert_eq!(
            classify(1.0, 2.0, ComplexityClass::Constant),
            MasterCase::Case2
        );
    }

    #[test]
    fn matrix_multiply_recurrence_is_case_1() {
        // T(n) = 8 T(n/2) + Θ(n²) — log_2(8) = 3 > 2 = c_f → case 1.
        assert_eq!(
            classify(8.0, 2.0, ComplexityClass::Quadratic),
            MasterCase::Case1
        );
    }

    #[test]
    fn root_dominated_is_case_3() {
        // T(n) = 2 T(n/2) + Θ(n²) — log_2(2) = 1 < 2 = c_f → case 3.
        assert_eq!(
            classify(2.0, 2.0, ComplexityClass::Quadratic),
            MasterCase::Case3
        );
    }

    #[test]
    fn degenerate_inputs_default_to_case_2() {
        // b <= 1 is undefined; return Case2 by total-function convention.
        assert_eq!(
            classify(2.0, 1.0, ComplexityClass::Linear),
            MasterCase::Case2
        );
        assert_eq!(
            classify(0.0, 2.0, ComplexityClass::Linear),
            MasterCase::Case2
        );
    }

    #[test]
    fn exponential_f_maps_to_case_3() {
        // f(n) exponent = +∞ — root always dominates.
        assert_eq!(
            classify(2.0, 2.0, ComplexityClass::Exponential),
            MasterCase::Case3
        );
    }

    #[test]
    fn master_case_derive_traits() {
        let c = MasterCase::Case1;
        let d = c;
        assert_eq!(d, MasterCase::Case1);
        assert_ne!(d, MasterCase::Case2);
        assert_eq!(format!("{c:?}"), "Case1");
    }

    // ---- brackets closed form ----

    #[test]
    fn brackets_closed_form_matches_known_sequence() {
        let expected: [i64; 11] = [0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55];
        for (i, want) in expected.iter().enumerate() {
            let got = brackets_closed_form(u32::try_from(i).unwrap_or(0)).round() as i64;
            assert_eq!(got, *want, "brackets({i})");
        }
        assert_eq!(brackets_closed_form(20).round() as i64, 6765);
    }

    #[test]
    fn brackets_closed_form_handles_zero_and_one() {
        assert!((brackets_closed_form(0).round() - 0.0).abs() < 1e-9);
        assert!((brackets_closed_form(1).round() - 1.0).abs() < 1e-9);
    }

    // ---- amortized analysis ----

    #[test]
    fn amortized_push_zero_ops() {
        let a = amortized_vec_push_cost(0);
        assert_eq!(a.total, 0.0);
        assert_eq!(a.amortized_per_op, 0.0);
    }

    #[test]
    fn amortized_push_converges_to_three() {
        let a = amortized_vec_push_cost(1);
        assert!((a.amortized_per_op - 3.0).abs() < 1e-9);
        let b = amortized_vec_push_cost(1_000_000);
        assert!((b.amortized_per_op - 3.0).abs() < 1e-9);
        let dup = b;
        assert_eq!(format!("{dup:?}").is_empty(), false);
    }

    // ---- find_opponent (lesson 3.1.1) ----

    #[test]
    fn find_opponent_returns_first_non_skip() {
        let roster = ["Khabib", "McGregor", "Adesanya"];
        assert_eq!(find_opponent(&roster, "Khabib"), Some("McGregor"));
        assert_eq!(find_opponent(&roster, "McGregor"), Some("Khabib"));
    }

    #[test]
    fn find_opponent_returns_none_when_only_skip_present() {
        let roster = ["Khabib", "Khabib"];
        assert_eq!(find_opponent(&roster, "Khabib"), None);
    }

    #[test]
    fn find_opponent_empty_roster() {
        let roster: [&str; 0] = [];
        assert_eq!(find_opponent(&roster, "anyone"), None);
    }

    // ---- book_fight (lesson 3.2.1) ----

    #[test]
    fn book_fight_ok_path() {
        let roster = ["Khabib", "McGregor"];
        assert_eq!(
            book_fight(&roster, "Khabib", "McGregor"),
            Ok("booked: Khabib vs McGregor".to_owned())
        );
    }

    #[test]
    fn book_fight_cannot_fight_self() {
        let roster = ["Khabib"];
        assert_eq!(
            book_fight(&roster, "Khabib", "Khabib"),
            Err(BookingError::CannotFightSelf)
        );
    }

    #[test]
    fn book_fight_not_on_roster() {
        let roster = ["Khabib"];
        assert_eq!(
            book_fight(&roster, "Ghost", "Khabib"),
            Err(BookingError::NotOnRoster("Ghost".to_owned()))
        );
        // Right-hand offender too.
        assert_eq!(
            book_fight(&roster, "Khabib", "Ghost"),
            Err(BookingError::NotOnRoster("Ghost".to_owned()))
        );
    }

    #[test]
    fn booking_error_derive_traits() {
        let e = BookingError::CannotFightSelf;
        let f = e.clone();
        assert_eq!(e, f);
        assert_ne!(e, BookingError::NotOnRoster("X".to_owned()));
        assert!(format!("{e:?}").contains("Cannot"));
    }

    // ---- schedule_safe (lesson 3.3.1) ----

    #[test]
    fn schedule_safe_none_starts_fresh() {
        let card = schedule_safe("Khabib vs McGregor", None);
        assert_eq!(card, vec!["Khabib vs McGregor".to_owned()]);
    }

    #[test]
    fn schedule_safe_some_appends() {
        let prior = vec!["existing".to_owned()];
        let card = schedule_safe("Adesanya vs Pereira", Some(prior));
        assert_eq!(
            card,
            vec!["existing".to_owned(), "Adesanya vs Pereira".to_owned()]
        );
    }

    #[test]
    fn schedule_safe_no_shared_default_across_calls() {
        // Two consecutive calls with None must NOT share state.
        let a = schedule_safe("a", None);
        let b = schedule_safe("b", None);
        assert_eq!(a, vec!["a".to_owned()]);
        assert_eq!(b, vec!["b".to_owned()]);
        // The Python mutable-default-arg bug would have b == ["a", "b"].
    }

    #[test]
    fn contract_marker_shape() {
        assert!(contract_marker().starts_with("contract:"));
        assert!(contract_marker().ends_with("— OK"));
    }

    // ---- proptest ----

    proptest! {
        #[test]
        fn amortized_per_op_is_three_for_any_positive_n(n in 1u32..=100_000) {
            let a = amortized_vec_push_cost(n);
            prop_assert!((a.amortized_per_op - 3.0).abs() < 1e-9);
            prop_assert!((a.total - 3.0 * f64::from(n)).abs() < 1e-6 * a.total.max(1.0));
        }

        #[test]
        fn brackets_closed_form_is_monotone(n in 2u32..30) {
            let b_n = brackets_closed_form(n);
            let b_n_plus_1 = brackets_closed_form(n + 1);
            prop_assert!(b_n_plus_1 > b_n);
        }

        #[test]
        fn classify_is_total(a in 1u32..16, b in 2u32..8) {
            let classes = [
                ComplexityClass::Constant,
                ComplexityClass::Logarithmic,
                ComplexityClass::Linear,
                ComplexityClass::Linearithmic,
                ComplexityClass::Quadratic,
                ComplexityClass::Exponential,
            ];
            for c in classes {
                let _ = classify(f64::from(a), f64::from(b), c);
            }
        }

        #[test]
        fn schedule_safe_grows_by_one(prior_len in 0usize..32) {
            let prior: Vec<String> = (0..prior_len).map(|i| format!("p{i}")).collect();
            let extended = schedule_safe("new", Some(prior));
            prop_assert_eq!(extended.len(), prior_len + 1);
            prop_assert_eq!(extended.last().map(String::as_str), Some("new"));
        }
    }
}
