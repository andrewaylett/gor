#![deny(
    bad_style,
    const_err,
    dead_code,
    improper_ctypes,
    missing_debug_implementations,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    unconditional_recursion,
    unreachable_pub,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
    clippy::expect_used
)]
#![doc = include_str!("../README.md")]

use std::path::PathBuf;
use std::process::exit;

use gor::exec;
use gor_eval::Value;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Opt {
    input: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let opts = Opt::from_args();

    let result = exec(&opts.input).await?;
    if let Value::Void = result {
        Ok(())
    } else if let Value::Int(rv) = result {
        exit(rv as i32);
    } else {
        Ok(())
    }
}
