//! Module 2 demo — PhantomData currency tags.
//!
//! Provable contract: `Money<C>` can only be added to or compared with
//! another `Money<C>` of the same currency `C`. Mixing `Money<Usd>`
//! with `Money<Eur>` is a **compile error**, not a runtime check.
//!
//! The contract is in the type system itself — there is no
//! `if currency != other.currency { panic! }` anywhere in this crate.
//! `cargo test` is the runtime proof; `cargo build` (which type-checks)
//! is the compile-time proof. The binary `money-demo` exercises one
//! end-to-end USD ledger and prints the contract-success line.
//!
//! Lessons covered: 2.1.1 newtype unit-safe, 2.1.2 PhantomData zero-cost
//! tag, 2.2.1 currency unit bug caught at compile time.

use std::fmt;
use std::marker::PhantomData;
use std::ops::Add;

/// Marker trait for currency types. Implemented by zero-sized unit
/// structs like `Usd`, `Eur`, `Jpy` — they exist purely to discriminate
/// `Money<Usd>` from `Money<Eur>` at the type level.
pub trait Currency {
    /// Three-letter ISO-4217 code printed in `Display` output.
    fn code() -> &'static str;
}

/// US dollar tag. Zero-sized; only the type matters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Usd;
impl Currency for Usd {
    fn code() -> &'static str {
        "USD"
    }
}

/// Euro tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Eur;
impl Currency for Eur {
    fn code() -> &'static str {
        "EUR"
    }
}

/// Japanese yen tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Jpy;
impl Currency for Jpy {
    fn code() -> &'static str {
        "JPY"
    }
}

/// Money tagged with a currency at the type level. Mixing tags is a
/// compile error — see `cents`'s `Add` impl.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Money<C: Currency> {
    cents: i64,
    _currency: PhantomData<C>,
}

impl<C: Currency> Money<C> {
    /// Construct `Money` from a whole-cent integer amount.
    pub const fn from_cents(cents: i64) -> Self {
        Self {
            cents,
            _currency: PhantomData,
        }
    }

    /// Inspect the underlying cent amount.
    pub const fn cents(self) -> i64 {
        self.cents
    }
}

// The Add impl pins both sides to the same `C` — `Money<Usd> + Money<Eur>`
// fails to type-check. That IS the provable contract.
impl<C: Currency> Add for Money<C> {
    type Output = Money<C>;
    fn add(self, rhs: Self) -> Self {
        Money::from_cents(self.cents.saturating_add(rhs.cents))
    }
}

impl<C: Currency> fmt::Display for Money<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let whole = self.cents / 100;
        let frac = (self.cents % 100).abs();
        write!(f, "{}.{:02} {}", whole, frac, C::code())
    }
}

/// Demo: build a USD ledger, sum it, return the marker line. Refactored
/// out of `main.rs` so the contract proof itself is unit-testable.
pub fn run() -> String {
    let a: Money<Usd> = Money::from_cents(1_99);
    let b: Money<Usd> = Money::from_cents(3_50);
    let total = a + b;
    assert_eq!(total.cents(), 5_49, "USD ledger sum violated");
    assert_eq!(format!("{total}"), "5.49 USD");

    let yen_a: Money<Jpy> = Money::from_cents(100);
    let yen_b: Money<Jpy> = Money::from_cents(250);
    let yen_total = yen_a + yen_b;
    assert_eq!(yen_total.cents(), 350, "JPY ledger sum violated");

    // The compile-time half of the proof: this line FAILS to compile
    // if uncommented, because the Add impl pins both sides to the same C.
    //
    //     let bad = a + yen_a;
    //
    // No `if currency_mismatch { panic! }` anywhere in this crate —
    // the type system rejects the bug before the program runs.

    format!("contract: Money<C> + Money<C> is currency-safe (USD={total}, JPY={yen_total}) — OK")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usd_and_eur_have_distinct_codes() {
        assert_eq!(Usd::code(), "USD");
        assert_eq!(Eur::code(), "EUR");
        assert_eq!(Jpy::code(), "JPY");
    }

    #[test]
    fn money_constructs_from_cents() {
        let m: Money<Usd> = Money::from_cents(1234);
        assert_eq!(m.cents(), 1234);
    }

    #[test]
    fn money_negative_cents_round_trip() {
        let m: Money<Eur> = Money::from_cents(-50);
        assert_eq!(m.cents(), -50);
    }

    #[test]
    fn same_currency_addition_sums_cents() {
        let a: Money<Usd> = Money::from_cents(199);
        let b: Money<Usd> = Money::from_cents(350);
        assert_eq!((a + b).cents(), 549);
    }

    #[test]
    fn addition_saturates_on_overflow() {
        let a: Money<Usd> = Money::from_cents(i64::MAX);
        let b: Money<Usd> = Money::from_cents(1);
        assert_eq!((a + b).cents(), i64::MAX);
    }

    #[test]
    fn addition_saturates_on_underflow() {
        let a: Money<Usd> = Money::from_cents(i64::MIN);
        let b: Money<Usd> = Money::from_cents(-1);
        assert_eq!((a + b).cents(), i64::MIN);
    }

    #[test]
    fn money_display_formats_with_currency_code() {
        let m: Money<Usd> = Money::from_cents(549);
        assert_eq!(format!("{m}"), "5.49 USD");

        let e: Money<Eur> = Money::from_cents(1000);
        assert_eq!(format!("{e}"), "10.00 EUR");
    }

    #[test]
    fn money_display_handles_negative_fraction() {
        let m: Money<Usd> = Money::from_cents(-149);
        // -1 dollars and 49 cents-of-magnitude.
        assert_eq!(format!("{m}"), "-1.49 USD");
    }

    #[test]
    fn money_equality_respects_currency_tag() {
        let a: Money<Usd> = Money::from_cents(100);
        let b: Money<Usd> = Money::from_cents(100);
        assert_eq!(a, b);
        let c: Money<Usd> = Money::from_cents(101);
        assert_ne!(a, c);
        // Money<Usd> and Money<Eur> CANNOT be compared with == ; that
        // would be a compile error. The type system is the proof.
    }

    #[test]
    fn run_returns_success_marker() {
        let line = run();
        assert!(line.contains("Money<C> + Money<C>"));
        assert!(line.contains("USD=5.49 USD"));
        assert!(line.contains("JPY=3.50 JPY"));
        assert!(line.contains("OK"));
    }
}
