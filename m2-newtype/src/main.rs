//! Provable contract: `Money<C> + Money<C>` is currency-safe. Mixed
//! currencies fail to compile, so `cargo build` is half the proof and
//! `cargo run` exiting zero is the other half.
//!
//! All logic lives in `m2_newtype::run`; `main` is the one-line shell
//! that prints whatever the run returns.

fn main() {
    println!("{}", m2_newtype::run());
}
