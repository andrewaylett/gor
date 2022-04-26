use async_trait::async_trait;
use gor_ast::module::Module;
use gor_ast::name::Name;
use gor_ast::AstError;
use gor_parse::ParseError;
use self_cell::self_cell;
use std::io;
use thiserror::Error;

pub mod file_loader;

#[derive(Error, Debug)]
pub enum LoaderError {
    #[error(transparent)]
    IOError(#[from] io::Error),
    #[error(transparent)]
    AstError(#[from] AstError),
    #[error(transparent)]
    ParseError(#[from] ParseError),
    #[error("Module not found: {}", .0)]
    ModuleNotFound(Name),
}

pub type LoaderResult<T> = Result<T, LoaderError>;

#[async_trait]
pub trait Loader {
    async fn load_module(&self, module: &str) -> LoaderResult<ModuleDescriptor>;
}

self_cell!(
    struct InnerModuleDescriptor {
        owner: String,
        #[covariant]
        dependent: Module,
    }

    impl {Debug, PartialEq, Eq, Hash}
);

pub struct ModuleDescriptor(InnerModuleDescriptor);
