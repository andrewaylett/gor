use thiserror::Error;

use crate::ast::AstError;
use crate::eval::RuntimeError;
use crate::eval::Value;

/// An error happened within Lua
#[derive(Error, Debug, PartialEq)]
pub enum LuaError {
    /// Something went wrong at runtime
    #[error(transparent)]
    RuntimeError(#[from] RuntimeError),
    /// We failed to translate the parse into an AST
    #[error(transparent)]
    AstError(#[from] AstError),
    /// Something happened within Pest
    #[error(transparent)]
    PestError(#[from] pest::error::Error<crate::parse::Rule>),
    /// Generic something went wrong
    #[error("Unknown Lua Error")]
    Unknown,
}

/// The regular return type for code dealing with Go values
pub type LuaResult = core::result::Result<Value, LuaError>;
