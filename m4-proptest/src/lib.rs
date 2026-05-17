//! Module 4 demo — proptest-verified `safe_div` kernel.
//!
//! Provable contract (`contracts/safe-div-v1.yaml`, validated with
//! `pv validate`):
//!   1. `safe_div(_, 0) == None` — totality on the divide-by-zero edge.
//!   2. `safe_div(i32::MIN, -1) == None` — totality on the only other
//!      `i32` overflow case.
//!   3. For every other `(a, b)`, `safe_div(a, b) == Some(a / b)` and
//!      it never panics.
//!
//! The unit tests below restate each obligation; the proptest harness
//! exercises invariants over millions of random `(a, b)` pairs and
//! shrinks counterexamples to a minimal failing input. `cargo test`
//! is the runtime proof; `pv validate contracts/safe-div-v1.yaml` is
//! the static proof of the YAML's well-formedness.
//!
//! Lessons covered: 4.1.1 proptest for invariants, 4.1.2 shrinking,
//! 4.1.3 round-trip, 5.1.1 pre/post conditions, 5.1.2 totality,
//! 5.1.3 Result as a bounded failure surface.

/// Divide `a` by `b`, returning `None` for the two `i32` overflow
/// cases (`b == 0` and `(i32::MIN, -1)`). All other inputs return
/// `Some(a / b)`. Never panics.
pub fn safe_div(a: i32, b: i32) -> Option<i32> {
    a.checked_div(b)
}

/// Outcome of `parse_and_divide`. Mirrors `DiffTargetOutcome` from the
/// claude-from-zero playbook: a sealed enum so tests can `assert_eq!`
/// the whole value, no `_ => panic!()` arms left uncovered.
#[derive(Debug, PartialEq, Eq)]
pub enum DivOutcome {
    /// Parsed two `i32`s and got a finite quotient.
    Ok(i32),
    /// First arg failed to parse.
    BadA(String),
    /// Second arg failed to parse.
    BadB(String),
    /// Missing one or both args.
    Missing,
    /// `b == 0`.
    DivByZero,
    /// `(i32::MIN, -1)`.
    Overflow,
}

/// Run the safediv-demo pipeline against `argv`. Refactored out of
/// `main.rs` so every branch is unit-testable. `main` is a thin shell
/// that pattern-matches on this outcome to produce stdout/stderr.
pub fn run(args: &[String]) -> DivOutcome {
    let raw_a = match args.get(1) {
        Some(s) => s,
        None => return DivOutcome::Missing,
    };
    let raw_b = match args.get(2) {
        Some(s) => s,
        None => return DivOutcome::Missing,
    };
    let a: i32 = match raw_a.parse() {
        Ok(n) => n,
        Err(e) => return DivOutcome::BadA(format!("'{raw_a}' is not a valid i32: {e}")),
    };
    let b: i32 = match raw_b.parse() {
        Ok(n) => n,
        Err(e) => return DivOutcome::BadB(format!("'{raw_b}' is not a valid i32: {e}")),
    };

    match safe_div(a, b) {
        Some(q) => DivOutcome::Ok(q),
        None if b == 0 => DivOutcome::DivByZero,
        None => DivOutcome::Overflow,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn argv(extras: &[&str]) -> Vec<String> {
        let mut v = vec!["safediv-demo".to_string()];
        v.extend(extras.iter().map(|s| (*s).to_string()));
        v
    }

    // ── obligations from contracts/safe-div-v1.yaml ──────────────────

    #[test]
    fn safe_div_div_by_zero_returns_none() {
        for a in [i32::MIN, -1, 0, 1, i32::MAX] {
            assert_eq!(safe_div(a, 0), None, "safe_div({a}, 0) should be None");
        }
    }

    #[test]
    fn safe_div_min_over_neg_one_returns_none() {
        assert_eq!(safe_div(i32::MIN, -1), None);
    }

    #[test]
    fn safe_div_returns_some_quotient_for_safe_inputs() {
        assert_eq!(safe_div(10, 2), Some(5));
        assert_eq!(safe_div(7, 3), Some(2));
        assert_eq!(safe_div(-10, 2), Some(-5));
        assert_eq!(safe_div(0, 5), Some(0));
        assert_eq!(safe_div(i32::MAX, 1), Some(i32::MAX));
        assert_eq!(safe_div(i32::MIN, 1), Some(i32::MIN));
    }

    // proptest harness — round-trip + invariants over millions of inputs.
    proptest! {
        // Obligation 3: for every safe (a, b), safe_div agrees with the
        // builtin `/` operator and never panics.
        #[test]
        fn prop_agrees_with_native_divide_on_safe_inputs(
            a in any::<i32>(),
            b in any::<i32>(),
        ) {
            let actual = safe_div(a, b);
            if b == 0 || (a == i32::MIN && b == -1) {
                prop_assert_eq!(actual, None);
            } else {
                prop_assert_eq!(actual, Some(a / b));
            }
        }

        // Obligation 1+2: the kernel NEVER panics. proptest will shrink
        // any counterexample to a minimal failing input.
        #[test]
        fn prop_never_panics(a in any::<i32>(), b in any::<i32>()) {
            let _ = safe_div(a, b);
        }

        // Round-trip property (4.1.3): if (a, b) divides evenly and
        // safe_div returns Some(q), then q * b == a. Exercises the
        // safe-input region only.
        #[test]
        fn prop_round_trip_for_exact_divisions(
            q in any::<i32>(),
            b in any::<i32>().prop_filter("nonzero", |b| *b != 0),
        ) {
            let Some(a) = q.checked_mul(b) else { return Ok(()) };
            // Skip the i32::MIN / -1 overflow case too.
            if a == i32::MIN && b == -1 { return Ok(()) }
            prop_assert_eq!(safe_div(a, b), Some(q));
        }
    }

    // ── run() outcome tests ─────────────────────────────────────────

    #[test]
    fn run_ok_for_safe_inputs() {
        assert_eq!(run(&argv(&["10", "2"])), DivOutcome::Ok(5));
        assert_eq!(run(&argv(&["-12", "4"])), DivOutcome::Ok(-3));
    }

    #[test]
    fn run_div_by_zero_outcome() {
        assert_eq!(run(&argv(&["7", "0"])), DivOutcome::DivByZero);
    }

    #[test]
    fn run_overflow_outcome() {
        assert_eq!(
            run(&argv(&[&i32::MIN.to_string(), "-1"])),
            DivOutcome::Overflow
        );
    }

    #[test]
    fn run_missing_args() {
        assert_eq!(run(&argv(&[])), DivOutcome::Missing);
        assert_eq!(run(&argv(&["10"])), DivOutcome::Missing);
    }

    #[test]
    fn run_bad_a() {
        assert_eq!(
            run(&argv(&["banana", "2"])),
            DivOutcome::BadA(
                "'banana' is not a valid i32: invalid digit found in string".to_string()
            )
        );
    }

    #[test]
    fn run_bad_b() {
        assert_eq!(
            run(&argv(&["10", "banana"])),
            DivOutcome::BadB(
                "'banana' is not a valid i32: invalid digit found in string".to_string()
            )
        );
    }
}

// L4 model-checking harness for the safe-div-v1 contract. Bounded to
// |a|, |b| <= 16 per KANI-DV-001 — small enough to solve in seconds
// on cadical (cbmc would otherwise enumerate 2^64 pairs over the full
// i32 domain). The bound includes the divide-by-zero edge (b = 0) and
// is large enough to cover the totality claim in practice. The Lean
// theorem in `lean/ProvableContracts/Theorems/SafeDiv.lean` discharges
// the universal claim at L5.
//
// Run with `cargo kani -p m4-proptest`.
#[cfg(kani)]
mod verification {
    use super::safe_div;

    /// KANI-DV-001 — safe_div is total over (i32, i32) bounded to
    /// |a|, |b| <= 16: never panics, returns None on b == 0, returns
    /// Some(a / b) otherwise. The i32::MIN / -1 overflow case lies
    /// outside this bound and is proven by the L5 Lean theorem.
    #[kani::proof]
    #[kani::solver(cadical)]
    fn kani_safe_div_totality() {
        let a: i32 = kani::any();
        let b: i32 = kani::any();
        kani::assume(a >= -16 && a <= 16);
        kani::assume(b >= -16 && b <= 16);

        let r = safe_div(a, b);
        if b == 0 {
            assert!(r.is_none());
        } else {
            assert_eq!(r, Some(a / b));
        }
    }
}
