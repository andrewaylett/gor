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

/// Errors
pub mod error;

pub use gor_ast::Located;
pub use gor_eval::exec;
pub use gor_eval::Value;

/// Utilities for integration testing
#[doc(hidden)]
pub mod test {
    use gor_loader::file_loader::FileLoader;
    use gor_loader::Loader;
    use std::path::PathBuf;

    /// Called by generated integration tests
    #[doc(hidden)]
    pub async fn test_go_file<T: Into<PathBuf>>(path: T) {
        let path: PathBuf = path.into();
        let loader = FileLoader::new(path);
        let module = loader.load_module("main").await;
        match module {
            Ok(_) => {
                //OK
            }
            Err(err) => {
                panic!("Failed to read input: {:?}", err);
            }
        }
    }
}
