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
#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

use async_trait::async_trait;
use gor_ast::module::SourceModule;
use gor_ast::name::Name;
use gor_ast::AstError;
use gor_core::parse_error::{parse_enum, InternalError};
use gor_parse::ParseError;
use self_cell::self_cell;
use std::io;
use thiserror::Error;

pub mod file_loader;

#[derive(Error, Debug)]
pub enum LoaderError {
    #[error("Failed to read module")]
    IOError(#[from] io::Error),
    #[error("Failed to parse module")]
    AstError(#[from] AstError),
    #[error("Failed to tokenise module")]
    ParseError(#[from] ParseError),
    #[error("Module not found: {0}")]
    ModuleNotFound(Name),
}

pub type LoaderResult<T> = Result<T, LoaderError>;

impl TryFrom<&str> for LoaderError {
    type Error = InternalError;

    fn try_from(value: &str) -> Result<Self, <Self as TryFrom<&str>>::Error> {
        let (name, param) = parse_enum(value)?;
        match name {
            "ModuleNotFound" => Ok(LoaderError::ModuleNotFound(
                Name::from_quoted(param).ok_or_else(|| {
                    InternalError::Error(format!("Expected string to be quoted: {}", param))
                })?,
            )),
            _ => Err(InternalError::Error(format!(
                "Unknown (or unimplemented) LinkerError variant: {}",
                name
            ))),
        }
    }
}

#[async_trait]
pub trait Loader {
    async fn load_module(&self, module: Name) -> LoaderResult<ModuleDescriptor>;
}

self_cell!(
    struct InnerModuleDescriptor {
        owner: String,
        #[covariant]
        dependent: SourceModule,
    }

    impl {Debug, PartialEq, Eq, Hash}
);

/// An owned reference to a module and its source
#[derive(Debug)]
pub struct ModuleDescriptor(InnerModuleDescriptor);

impl ModuleDescriptor {
    pub fn module(&self) -> &SourceModule {
        self.0.borrow_dependent()
    }
}
