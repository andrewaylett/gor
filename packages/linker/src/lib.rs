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

use gor_ast::name::Name;
use gor_core::parse_error::{parse_enum, InternalError};
use gor_loader::file_loader::FileLoader;
use gor_loader::{Loader, LoaderError, ModuleDescriptor};
use std::collections::HashMap;
use std::ops::Deref;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum LinkerError {
    #[error("Failed to read module")]
    Loader(#[from] LoaderError),
    #[error("Module not found: {0}")]
    NotFound(Name),
}

pub type LinkerResult<T> = Result<T, LinkerError>;

impl TryFrom<&str> for LinkerError {
    type Error = InternalError;

    fn try_from(value: &str) -> Result<Self, <Self as TryFrom<&str>>::Error> {
        let (name, param) = parse_enum(value)?;
        match name {
            "Loader" => Ok(LinkerError::Loader(LoaderError::try_from(param)?)),
            _ => Err(InternalError::Error(format!(
                "Unknown (or unimplemented) LinkerError variant: {}",
                name
            ))),
        }
    }
}

#[derive(Debug)]
pub struct Linker {
    modules: HashMap<Name, Box<ModuleDescriptor>>,
}

impl Linker {
    pub async fn bootstrap(loader: FileLoader) -> LinkerResult<Linker> {
        let mut still_to_load = vec![Name::from("main")];
        let mut modules = HashMap::new();
        while let Some(module_name) = still_to_load.pop() {
            if modules.contains_key(&module_name) {
                continue;
            }

            let descriptor = loader.load_module(module_name).await?;
            still_to_load.append(&mut descriptor.module().imports.clone());
            modules.insert(module_name, Box::new(descriptor));
        }
        Ok(Linker { modules })
    }

    pub fn lookup(&self, name: Name) -> LinkerResult<&ModuleDescriptor> {
        self.modules
            .get(&name)
            .ok_or(LinkerError::NotFound(name))
            .map(|boxed| boxed.deref())
    }
}
