# design-by-provable-contracts

<p align="center">
  <img src="assets/hero.svg" alt="Type-system invariants → typestate → property tests → YAML contracts" width="100%"/>
</p>

[![CI](https://github.com/paiml/design-by-provable-contracts/actions/workflows/ci.yml/badge.svg)](https://github.com/paiml/design-by-provable-contracts/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Coverage](https://img.shields.io/badge/coverage-100%25-brightgreen.svg)](#install)

Companion repository for the **Design by Provable Contracts** Coursera
course — course 7 of the [Rust for Data Engineering](https://www.coursera.org/specializations/rust-for-data-engineering)
specialization.

Three small Rust demos that wire together the four pillars of
contract-driven Rust:

- **Type-system invariants** — illegal states cannot be constructed
- **Typestate programming** — protocol violations are compile errors
- **Property-based testing** — `proptest` shrinks counterexamples to
  minimal failing inputs
- **YAML contracts + `pv`** — runtime obligations declared once and
  enforced at every build

## Demos

| Crate | Lesson coverage | Provable contract |
|---|---|---|
| [`m2-newtype`](m2-newtype/) | M2.1 newtype / PhantomData / Deref + M2.2 currency-unit demo | `Money<C> + Money<C>` is currency-safe; mixed currencies fail to compile |
| [`m3-typestate`](m3-typestate/) | M3.1 typestate basics / consume-return / typed builder + M3.2 connection demo | `query` only compiles on `Connection<Authenticated>` — protocol violations caught at type-check time |
| [`m4-proptest`](m4-proptest/) | M4.1 proptest / shrinking / round-trip + M4.2 parser-edge demo + M5 contract-driven API | `safe_div` is total over i32×i32; `proptest` exercises millions of pairs and shrinks any failure |

## Prerequisites

- Rust 1.75+ (`rustup default stable`)
- Optional: `aprender-contracts-cli` (provides `pv`) for YAML contract
  validation
- Optional: `pmat` for the advisory compliance report

## Quick start

```bash
git clone https://github.com/paiml/design-by-provable-contracts
cd design-by-provable-contracts

# Compile-time half of every contract proof.
cargo build --workspace --locked

# Runtime half. proptest runs millions of cases with shrinking.
cargo test  --workspace --locked

# Run any demo binary.
cargo run -p m2-newtype  --bin money-demo
cargo run -p m3-typestate --bin typestate-demo
cargo run -p m4-proptest --bin safediv-demo -- 10 2
```

## How the M4 YAML contract works

`contracts/safe-div-v1.yaml` declares the invariants `safe_div` must
satisfy: totality on `b == 0`, totality on `(i32::MIN, -1)`, agreement
with native `/` on every other input, no panics. `pv validate` is the
static check that the YAML itself is well-formed; the proptest harness
in `m4-proptest/src/lib.rs` is the runtime check that the Rust
implementation satisfies every obligation. CI runs both on every push.

The YAML is the single source of truth — the Rust unit tests, proptest
generators, and end-of-`main` `assert!`s all restate invariants named
in the YAML so a contract change forces every layer to update in
lockstep.

## Install

Build with stable Rust 1.75+:

```bash
git clone https://github.com/paiml/design-by-provable-contracts
cd design-by-provable-contracts
cargo build --workspace --locked
cargo test  --workspace --locked
```

Coverage (matches the CI gate at 100%):

```bash
cargo install cargo-llvm-cov  # one-time
cargo llvm-cov --workspace \
  --ignore-filename-regex 'main\.rs|src/bin/' \
  --fail-under-lines 100
```

Optional — the provable-contract validator and the PAIML compliance
checker the CI workflow runs:

```bash
cargo install aprender-contracts-cli  # provides `pv`
cargo install pmat                    # PAIML compliance toolkit
```

## Go deeper

For real data-engineering work with this toolkit, continue with the
rest of the
[Rust for Data Engineering](https://www.coursera.org/specializations/rust-for-data-engineering)
specialization. The production contract engine lives in
[`paiml/aprender`](https://github.com/paiml/aprender) under
`crates/aprender-contracts`; this teaching repo is the small,
end-to-end-readable version.

## License

Dual-licensed under MIT or Apache-2.0 — pick the one that fits your
downstream use. SPDX: `MIT OR Apache-2.0`.
