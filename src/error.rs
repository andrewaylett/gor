use thiserror::Error;

use gor_eval::RuntimeError;
use gor_eval::Value;
use gor_linker::LinkerError;
use gor_loader::LoaderError;

/// An error happened within Go
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum GoError {
    /// Something went wrong at runtime
    #[error("Runtime Error")]
    RuntimeError(#[from] RuntimeError),
    /// Something happened trying to load the module
    #[error("Error Loading Module")]
    LoaderError(#[from] LoaderError),
    /// Something happened trying to link the module
    #[error("Error Linking Module")]
    LinkerError(#[from] LinkerError),
    /// Just giving up
    #[error("Error: {0}")]
    Error(String),
}

/// The regular return type for code dealing with Go values
pub type GoResult = Result<Value, GoError>;
