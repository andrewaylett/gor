use thiserror::Error;

use gor_eval::RuntimeError;
use gor_eval::Value;
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
}

/// The regular return type for code dealing with Go values
pub type GoResult = core::result::Result<Value, GoError>;
