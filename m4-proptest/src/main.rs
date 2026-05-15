//! Provable contract: `contracts/safe-div-v1.yaml` validated with `pv
//! validate`. The binary translates `DivOutcome` from the lib into
//! stdout/stderr + exit codes. `cargo run -p m4-proptest --bin
//! safediv-demo -- 10 2` exits zero with the contract marker on stderr.

use std::env;
use std::process;

use m4_proptest::{run, DivOutcome};

fn main() {
    let args: Vec<String> = env::args().collect();
    match run(&args) {
        DivOutcome::Ok(q) => {
            println!("{q}");
            eprintln!("contract: safe-div-v1 holds — totality + no panic — OK");
        }
        DivOutcome::Missing => {
            eprintln!("usage: safediv-demo <a> <b>");
            process::exit(2);
        }
        DivOutcome::BadA(msg) | DivOutcome::BadB(msg) => {
            eprintln!("error: {msg}");
            process::exit(2);
        }
        DivOutcome::DivByZero => {
            eprintln!("error: divide by zero — safe_div returned None per contract");
            process::exit(3);
        }
        DivOutcome::Overflow => {
            eprintln!("error: i32::MIN / -1 overflow — safe_div returned None per contract");
            process::exit(3);
        }
    }
}
