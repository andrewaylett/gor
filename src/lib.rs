#![deny(
    bad_style,
    const_err,
    dead_code,
    improper_ctypes,
    missing_debug_implementations,
    missing_docs,
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
#![forbid(unsafe_code)]

//! An implementation of Go, written as an interpreter in Rust.
//!
//! We provide a binary as well as this library.

mod ast;
mod error;
mod eval;
mod parse;

pub use ast::Located;
pub use error::{GoError, GoResult};
pub use eval::exec;
pub use eval::Value;

/// Utilities for integration testing
#[doc(hidden)]
pub mod test {
    use std::fs::read_to_string;
    use std::path::PathBuf;

    use crate::parse::parse;
    use crate::parse::Rule;

    /// Called by generated integration tests
    #[doc(hidden)]
    pub async fn test_go_file<T: Into<PathBuf>>(path: T) {
        let path: PathBuf = path.into();
        let input = read_to_string(path);
        match input {
            Ok(input) => {
                let parse = parse(Rule::module, &input);
                if let Err(e) = parse {
                    panic!("Parse failed: {}", e);
                }
            }
            Err(err) => {
                panic!("Failed to read input: {}", err);
            }
        }
    }
}
