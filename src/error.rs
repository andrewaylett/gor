use thiserror::Error;

use crate::ast::AstError;
use crate::eval::RuntimeError;
use crate::eval::Value;

#[derive(Error, Debug, PartialEq)]
pub enum LuaError {
    #[error(transparent)]
    RuntimeError(#[from] RuntimeError),
    #[error(transparent)]
    AstError(#[from] AstError),
    #[error(transparent)]
    PestError(#[from] pest::error::Error<crate::parse::Rule>),
    #[error("Unknown Lua Error")]
    Unknown,
}

pub(crate) type LuaResult = core::result::Result<Value, LuaError>;
