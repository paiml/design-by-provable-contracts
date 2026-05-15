//! Provable contract: `Connection<Authenticated>` is the only state on
//! which `query` compiles. `cargo build` is the compile-time half of
//! the proof; this binary's zero-exit is the runtime half.

fn main() {
    println!("{}", m3_typestate::run());
}
