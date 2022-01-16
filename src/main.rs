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

use std::path::PathBuf;

use structopt::StructOpt;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[derive(StructOpt)]
struct Opt {
    input: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let opts = Opt::from_args();

    let mut s = String::new();
    let mut file = File::open(&*opts.input).await?;
    file.read_to_string(&mut s).await?;

    let result = lua::exec(&s).await?;
    if let lua::eval::Value::Void = result {
        // Ignore
    } else {
        println!("{:?}", result);
    }
    Ok(())
}
