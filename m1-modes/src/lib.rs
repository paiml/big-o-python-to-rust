//! M1 — Tour of the three modes of complexity proof (UFC/BJJ edition).
//!
//! Every Big-O claim ships with three independent receipts; this crate
//! gives the on-ramp tour, anchored to course lessons 1.1.1 (what
//! complexity means), 1.2.1 (falsifiability), and 1.3.1 (depyler as the
//! transpiler shortcut). The running example throughout is a roster of
//! UFC fighters:
//!
//! * **Empirical** — measure mean times across doubling roster sizes
//!   (1024, 2048, 4096 fighters) and compute consecutive ratios. The
//!   shape of the ratio sequence is the empirical fingerprint of a
//!   complexity class. Lesson 1.1.1.
//! * **Structural** — argue about a recurrence. A single-elimination
//!   tournament-bracket build is `T(n) = 2 T(n/2) + Θ(n)` = master
//!   theorem Case 2 = O(n log n). Lesson 1.2.1.
//! * **Formal** — track the proof status as data so the rest of the
//!   workspace can route a contract to the appropriate gate. Lesson
//!   1.3.1 introduces depyler as a *shortcut* into this loop: you do not
//!   trust depyler's translation, you measure it through the three
//!   modes.
//!
//! Every public item is bound, in spirit, to `contracts/binding.yaml`
//! via the `ProofStatus` enum (which is what M3/M5 use to label their
//! own proofs).

/// Which mode of proof produced a complexity claim.
///
/// Bound informally to `contracts/binding.yaml` — every contract in this
/// workspace eventually labels itself with one of these three statuses so
/// downstream tooling (`pmat comply`, `pv status`) can route audits to the
/// correct gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProofStatus {
    /// Verified by `criterion` benches over doubling input sizes.
    Empirical,
    /// Verified by recurrence / amortized argument in code.
    Structural,
    /// Verified by a Lean 4 theorem under `lean/BigOFromZero/`.
    Formal,
}

/// Compute the maximum consecutive ratio across a sequence of mean times
/// taken at doubling input sizes (e.g. roster sizes 1024, 2048, 4096).
///
/// This is the empirical fingerprint used by every `complexity-*-v1`
/// contract: an O(1) Elo lookup yields ratios near 1.0, an O(n) roster
/// scan near 2.0, an O(n²) rivalry-pair sweep near 4.0.
///
/// Returns `0.0` when `times.len() < 2` (no pair exists to compare) or
/// when any earlier sample is non-positive (cannot form a valid ratio).
///
/// Bound to: the empirical-proof side of every `complexity-*-v1.yaml`.
/// Course lesson 1.1.1.
#[must_use]
pub fn empirical_doubling_ratio(times: &[f64]) -> f64 {
    if times.len() < 2 {
        return 0.0;
    }
    let mut max_ratio = 0.0_f64;
    for window in times.windows(2) {
        let prev = window[0];
        let next = window[1];
        if prev <= 0.0 || !prev.is_finite() || !next.is_finite() {
            return 0.0;
        }
        let ratio = next / prev;
        if ratio > max_ratio {
            max_ratio = ratio;
        }
    }
    max_ratio
}

/// Which of the three master-theorem cases governs a recurrence
/// `T(n) = a · T(n/b) + f(n)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MasterCase {
    /// `f(n) = O(n^c)` with `c < log_b a` — leaves dominate.
    Case1,
    /// `f(n) = Θ(n^c)` with `c = log_b a` — balanced (e.g. the
    /// tournament-bracket build `2 T(n/2) + Θ(n)`).
    Case2,
    /// `f(n) = Ω(n^c)` with `c > log_b a` — root dominates.
    Case3,
}

/// Classify a recurrence `T(n) = a · T(n/b) + n^{f_exponent}` into one of
/// the three master-theorem cases.
///
/// The classic UFC bracket build — repeatedly halve the roster, then do
/// O(n) merge work — is `2 T(n/2) + Θ(n)`, i.e. `(a=2, b=2, exp=1)`,
/// which lands in Case 2 = O(n log n).
///
/// Bound to: structural side of `contracts/complexity-linearithmic-v1.yaml`.
/// Course lesson 1.2.1 (falsifiable claim shape).
///
/// Returns [`MasterCase::Case2`] for degenerate `b <= 1` since `log_b a`
/// is undefined there and we prefer a total function over a `Result` in
/// this teaching context.
#[must_use]
pub fn classify_recurrence(a: f64, b: f64, f_exponent: f64) -> MasterCase {
    let c_critical = if b > 1.0 && a > 0.0 {
        a.log(b)
    } else {
        // Degenerate: treat as case 2 (balanced) — the safest "bracket-build" default.
        return MasterCase::Case2;
    };
    if f_exponent < c_critical - 1e-9 {
        MasterCase::Case1
    } else if (f_exponent - c_critical).abs() < 1e-9 {
        MasterCase::Case2
    } else {
        MasterCase::Case3
    }
}

/// Pick the natural proof mode for a given contract slug.
///
/// Master-theorem and Fibonacci-closed-form contracts are genuinely
/// Lean-provable (`status: proved`); recurrence/amortized contracts are
/// structural; everything else is empirical (criterion + statistical
/// CIs). Course lesson 1.3.1.
///
/// Bound to: `contracts/binding.yaml`.
#[must_use]
pub fn applicable_mode(contract: &str) -> ProofStatus {
    if contract.contains("master-theorem") || contract.contains("fibonacci-closed-form") {
        return ProofStatus::Formal;
    }
    if contract.contains("recurrence") || contract.contains("amortized") {
        return ProofStatus::Structural;
    }
    ProofStatus::Empirical
}

/// Return the proof status that the formal-mode tour case produces: by
/// construction, the formal track always lands at [`ProofStatus::Formal`]
/// when the corresponding Lean theorem builds.
///
/// Bound to: `contracts/binding.yaml` — used by the demo binary to print
/// the three proof statuses one after another.
#[must_use]
pub const fn formal_status() -> ProofStatus {
    ProofStatus::Formal
}

/// Check that a complexity claim is *well-formed* (and therefore
/// falsifiable): input sizes form a doubling sequence and the ratio
/// table has the right length (one ratio per consecutive pair).
///
/// A claim that fails this check is not wrong — it's structurally
/// untestable. A bench that does not double its input, or that doesn't
/// publish a ratio table, cannot be falsified the way Big O wants.
///
/// Bound to: course lesson 1.2.1 (falsifiability). Used by the demo
/// binary to gate the empirical receipt.
#[must_use]
pub fn is_well_formed_claim(input_sizes: &[u64], ratios: &[f64]) -> bool {
    if input_sizes.len() < 2 {
        return false;
    }
    if ratios.len() != input_sizes.len() - 1 {
        return false;
    }
    for window in input_sizes.windows(2) {
        let lo = window[0];
        let hi = window[1];
        if lo == 0 || hi != lo.saturating_mul(2) {
            return false;
        }
    }
    true
}

/// Depyler's transpile-preservation contract: same complexity class on
/// both sides of the Python→Rust translation.
///
/// Course lesson 1.3.1 — depyler produces a starting-point Rust
/// translation; you do not trust it, you measure it. This predicate is
/// the gate the empirical receipt checks.
#[must_use]
pub fn transpile_class_preserved(py_class: &str, rs_class: &str) -> bool {
    py_class == rs_class
}

/// Constant-factor speedup ratio `python_time / rust_time`.
///
/// Course lesson 1.3.1. A 4x speedup is typical for naive Python
/// list-comp → Rust iterator translations; the class is preserved, the
/// constants improve.
///
/// Returns `0.0` if `rs_time` is non-positive or non-finite.
#[must_use]
pub fn speedup(py_time: f64, rs_time: f64) -> f64 {
    if rs_time <= 0.0 || !rs_time.is_finite() || !py_time.is_finite() {
        return 0.0;
    }
    py_time / rs_time
}

/// Runtime marker the demo binary asserts at exit so an operator can grep
/// for "contract: m1-modes-tour holds — OK" in stderr without needing to
/// open Coursera or read the Lean output.
#[must_use]
pub const fn contract_marker() -> &'static str {
    "contract: m1-modes-tour holds — OK"
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::float_cmp,
    clippy::bool_assert_comparison
)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn empirical_doubling_ratio_on_o1() {
        // O(1) Elo lookup — every sample takes the same time, ratio ≈ 1.
        let ratio = empirical_doubling_ratio(&[1.0, 1.0, 1.0, 1.0, 1.0]);
        assert!((ratio - 1.0).abs() < 1e-9);
    }

    #[test]
    fn empirical_doubling_ratio_on_on() {
        // O(n) roster scan — time doubles each step, max ratio ≈ 2.
        let ratio = empirical_doubling_ratio(&[1.0, 2.0, 4.0, 8.0, 16.0]);
        assert!((ratio - 2.0).abs() < 1e-9);
    }

    #[test]
    fn empirical_doubling_ratio_on_on2() {
        // O(n^2) rivalry sweep — time quadruples each step, max ratio ≈ 4.
        let ratio = empirical_doubling_ratio(&[1.0, 4.0, 16.0, 64.0]);
        assert!((ratio - 4.0).abs() < 1e-9);
    }

    #[test]
    fn empirical_doubling_ratio_handles_empty_and_singleton() {
        assert!((empirical_doubling_ratio(&[]) - 0.0).abs() < f64::EPSILON);
        assert!((empirical_doubling_ratio(&[1.0]) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn empirical_doubling_ratio_rejects_nonpositive_or_nonfinite() {
        assert!((empirical_doubling_ratio(&[0.0, 1.0]) - 0.0).abs() < f64::EPSILON);
        assert!((empirical_doubling_ratio(&[-1.0, 1.0]) - 0.0).abs() < f64::EPSILON);
        assert!((empirical_doubling_ratio(&[1.0, f64::NAN]) - 0.0).abs() < f64::EPSILON);
        assert!((empirical_doubling_ratio(&[1.0, f64::INFINITY]) - 0.0).abs() < f64::EPSILON);
        assert!((empirical_doubling_ratio(&[f64::NAN, 1.0]) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn classify_recurrence_bracket_build_is_case_2() {
        // Tournament bracket: 2 T(n/2) + Θ(n)  →  Case 2 = O(n log n).
        assert_eq!(classify_recurrence(2.0, 2.0, 1.0), MasterCase::Case2);
        // Binary search through ranked roster: 1 T(n/2) + Θ(1) → Case 2 = O(log n).
        assert_eq!(classify_recurrence(1.0, 2.0, 0.0), MasterCase::Case2);
        // Strassen-ish: 8 T(n/2) + Θ(n²) → Case 1.
        assert_eq!(classify_recurrence(8.0, 2.0, 2.0), MasterCase::Case1);
        // Root-dominated: 2 T(n/2) + Θ(n²) → Case 3.
        assert_eq!(classify_recurrence(2.0, 2.0, 2.0), MasterCase::Case3);
    }

    #[test]
    fn classify_recurrence_degenerate_inputs() {
        // b <= 1 is undefined; return Case2 by total-function convention.
        assert_eq!(classify_recurrence(2.0, 1.0, 1.0), MasterCase::Case2);
        assert_eq!(classify_recurrence(0.0, 2.0, 1.0), MasterCase::Case2);
        // Exercise derived traits on MasterCase.
        let c = MasterCase::Case1;
        let d = c;
        assert_eq!(d, MasterCase::Case1);
        assert_ne!(d, MasterCase::Case2);
        assert_eq!(format!("{c:?}"), "Case1");
    }

    #[test]
    fn applicable_mode_routes_contracts() {
        assert_eq!(
            applicable_mode("complexity-linear-v1"),
            ProofStatus::Empirical
        );
        assert_eq!(
            applicable_mode("master-theorem-case-2"),
            ProofStatus::Formal
        );
        assert_eq!(
            applicable_mode("fibonacci-closed-form"),
            ProofStatus::Formal
        );
        assert_eq!(
            applicable_mode("banker-amortized-push"),
            ProofStatus::Structural
        );
        assert_eq!(
            applicable_mode("bracket-recurrence"),
            ProofStatus::Structural
        );
    }

    #[test]
    fn formal_status_round_trips() {
        let s = formal_status();
        assert_eq!(s, ProofStatus::Formal);
        let copy = s;
        assert_eq!(format!("{copy:?}"), "Formal");
        assert_eq!(s, copy);
        assert_ne!(s, ProofStatus::Empirical);
        assert_ne!(s, ProofStatus::Structural);
    }

    #[test]
    fn well_formed_claim_doubling_sizes() {
        assert!(is_well_formed_claim(
            &[1024, 2048, 4096, 8192],
            &[2.01, 1.99, 2.02]
        ));
        // Sizes don't double.
        assert!(!is_well_formed_claim(&[1024, 1500, 2000], &[1.5, 1.3]));
        // Ratio length mismatch.
        assert!(!is_well_formed_claim(&[1024, 2048, 4096], &[2.0]));
        // Too few sizes.
        assert!(!is_well_formed_claim(&[1024], &[]));
        assert!(!is_well_formed_claim(&[], &[]));
        // Zero start.
        assert!(!is_well_formed_claim(&[0, 0], &[1.0]));
    }

    #[test]
    fn transpile_class_preserved_predicate() {
        assert!(transpile_class_preserved("O(n)", "O(n)"));
        assert!(!transpile_class_preserved("O(n)", "O(n^2)"));
    }

    #[test]
    fn speedup_typical_iterator_translation() {
        assert!((speedup(40.0, 10.0) - 4.0).abs() < 1e-9);
        // Non-positive rs_time → 0.0.
        assert!((speedup(40.0, 0.0) - 0.0).abs() < f64::EPSILON);
        assert!((speedup(40.0, -1.0) - 0.0).abs() < f64::EPSILON);
        // Non-finite inputs → 0.0.
        assert!((speedup(f64::NAN, 10.0) - 0.0).abs() < f64::EPSILON);
        assert!((speedup(10.0, f64::INFINITY) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn contract_marker_shape() {
        let m = contract_marker();
        assert!(m.starts_with("contract:"));
        assert!(m.ends_with("— OK"));
    }

    proptest! {
        #[test]
        fn ratio_is_nonnegative_on_positive_sequences(seed in 1u32..1_000_000) {
            let mut t = vec![1.0_f64];
            let bits = seed % 8 + 2; // 2..=9 samples
            for _ in 0..bits {
                let last = *t.last().unwrap_or(&1.0);
                t.push(last * 2.0);
            }
            let r = empirical_doubling_ratio(&t);
            prop_assert!(r >= 0.0);
            prop_assert!(r.is_finite());
        }

        #[test]
        fn classify_recurrence_total(a in 1u32..16, b in 2u32..8, exp in 0u32..4) {
            // Returns *some* case for any (a, b, exp) in this grid.
            let _ = classify_recurrence(f64::from(a), f64::from(b), f64::from(exp));
        }

        #[test]
        fn speedup_positive_inputs(py in 1u32..1000, rs in 1u32..1000) {
            let s = speedup(f64::from(py), f64::from(rs));
            prop_assert!(s > 0.0);
            prop_assert!(s.is_finite());
        }
    }
}
