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
use gor_loader::file_loader::FileLoader;
use gor_loader::{Loader, LoaderError, LoaderResult, ModuleDescriptor};
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use thiserror::Error;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::task::JoinError;

#[derive(Error, Debug)]
pub enum LinkerError {
    #[error("Failed to read module")]
    Loader(#[from] LoaderError),
    #[error("Join issue")]
    Tokio(#[from] JoinError),
    #[error("Module not found: {0}")]
    NotFound(Name),
}

pub type LinkerResult<T> = Result<T, LinkerError>;

#[derive(Debug)]
pub struct Linker {
    modules: HashMap<Name, Box<ModuleDescriptor>>,
}

async fn do_load(
    loader: FileLoader,
    name: Name,
    sender: UnboundedSender<LoaderResult<ModuleDescriptor>>,
) {
    let descriptor = loader.load_module(name).await;
    sender.send(descriptor).unwrap();
}

impl Linker {
    pub async fn bootstrap(loader: FileLoader) -> LinkerResult<Linker> {
        let (sender, mut receiver) = unbounded_channel::<LoaderResult<ModuleDescriptor>>();
        let mut modules: HashMap<Name, Box<ModuleDescriptor>> = HashMap::new();
        let main: Name = "main".into();
        let mut loading = HashSet::from([main]);

        tokio::spawn(do_load(loader.clone(), main, sender.clone()));

        while let Some(descriptor) = receiver.recv().await {
            let descriptor = descriptor?;
            let name = descriptor.module().package;
            loading.remove(&name);
            let requirements = descriptor.module().imports.clone();
            modules.insert(name, Box::new(descriptor));

            for name in requirements {
                if modules.contains_key(&name) || loading.contains(&name) {
                    continue;
                }

                loading.insert(name);
                tokio::spawn(do_load(loader.clone(), name, sender.clone()));
            }
            if loading.is_empty() {
                break;
            }
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
