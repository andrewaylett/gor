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

/// Errors
pub mod error;

pub use gor_ast::Located;
pub use gor_eval::exec;
pub use gor_eval::Value;

/// Utilities for integration testing
#[doc(hidden)]
pub mod test {
    use gor_ast::module::Module;
    use std::fs::read_to_string;
    use std::path::PathBuf;

    use gor_parse::parse;
    use gor_parse::Rule;

    /// Called by generated integration tests
    #[doc(hidden)]
    pub async fn test_go_file<T: Into<PathBuf>>(path: T) {
        let path: PathBuf = path.into();
        let input = read_to_string(path);
        match input {
            Ok(input) => {
                let parse = parse(Rule::module, &input);
                match parse {
                    Err(e) => {
                        panic!("Parse failed: {}", e);
                    }
                    Ok(pairs) => {
                        let module = Module::try_from(pairs);
                        if let Err(e) = module {
                            panic!("AST failed: {:?}", e);
                        }
                    }
                }
            }
            Err(err) => {
                panic!("Failed to read input: {}", err);
            }
        }
    }
}
