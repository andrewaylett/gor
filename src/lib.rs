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
#![doc = include_str!("../README.md")]

use crate::error::{GoError, GoResult};
use gor_eval::{FunctionExecutionExt, RuntimeError};
use gor_loader::file_loader::FileLoader;
use gor_loader::Loader;
use std::path::PathBuf;

/// Errors
pub mod error;

/// Executes a main module found in the referenced file
///
/// ```
/// use gor::exec;
/// let result = exec("tests/compile/hello.go");
/// ```
pub async fn exec<T: Into<PathBuf>>(main: T) -> GoResult {
    let loader = FileLoader::new(main);
    let module_descriptor = loader.load_module("main").await?;
    let r = module_descriptor
        .module(|module| {
            let function = module.function("main");
            match function {
                Some(function) => Ok(function.execute_in_default_context()),
                None => Err(GoError::RuntimeError(RuntimeError::NameError(
                    "main".into(),
                ))),
            }
        })?
        .await?;
    Ok(r)
}

/// Utilities for integration testing
#[doc(hidden)]
pub mod test {
    use crate::{exec, GoError};
    use std::path::PathBuf;

    /// Called by generated integration tests
    #[doc(hidden)]
    pub async fn test_go_file<T: Into<PathBuf>>(path: T, error_str: Option<&str>) {
        let expected_error = error_str.map(|e| GoError::try_from(e).unwrap());
        let result = exec(path.into()).await;
        match (result, expected_error) {
            (Ok(_), None) => {
                // This is fine
            }
            (Err(e), None) => {
                // Classic failure
                panic!("Expected the test to evaluate successfully: {:?}", e);
            }
            (Ok(r), Some(e)) => {
                // It works‽
                panic!(
                    "Expected the test to evaluate to an error ({:?}), got {:?}",
                    e, r
                );
            }
            (Err(actual), Some(expected)) => {
                // Expected failure
                assert_eq!(expected, actual);
            }
        }
    }
}
