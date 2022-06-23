use gor_core::parse_error::{parse_enum, InternalError};
use thiserror::Error;

use gor_eval::RuntimeError;
use gor_eval::Value;
use gor_loader::LoaderError;

/// An error happened within Go
#[derive(Error, Debug, PartialEq)]
#[non_exhaustive]
pub enum GoError {
    /// Something went wrong at runtime
    #[error("Runtime Error")]
    RuntimeError(#[from] RuntimeError),
    /// Something happened trying to load the module
    #[error("Error Loading Module")]
    LoaderError(#[from] LoaderError),
    /// Just giving up
    #[error("Error: {0}")]
    Error(String),
}

/// The regular return type for code dealing with Go values
pub type GoResult = Result<Value, GoError>;

impl TryFrom<&str> for GoError {
    type Error = InternalError;

    fn try_from(value: &str) -> Result<Self, <Self as TryFrom<&str>>::Error> {
        let (name, param) = parse_enum(value)?;
        match name {
            "RuntimeError" => Ok(GoError::RuntimeError(RuntimeError::try_from(param)?)),
            "Error" => Ok(GoError::Error(param.to_string())),
            _ => Err(InternalError::Error(format!(
                "Unknown (or unimplemented) error type: {}",
                name
            ))),
        }
    }
}
