use thiserror::Error;

use crate::ast::Name;
use crate::Expression;

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Not a function: {0:?}")]
    NotAFunction(Value),
    #[error("Name not found: {0}")]
    NameError(Name),
    #[error("Can't static eval {0:?}")]
    StaticEvaluationFailure(Expression),
    #[error("Type Mismatch: expected {expected:?}, not {found:?}")]
    TypeError { expected: Type, found: Type },
}

type Result<R> = core::result::Result<R, RuntimeError>;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Type {
    Int,
    String,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Int(i64),
    String(String),
}

impl Value {
    pub fn as_type(&self) -> Type {
        match self {
            Value::Int(_) => Type::Int,
            Value::String(_) => Type::String,
        }
    }

    pub fn as_int(&self) -> Result<i64> {
        if let Value::Int(n) = self {
            Ok(*n)
        } else {
            Err(RuntimeError::TypeError {
                expected: Type::Int,
                found: self.as_type(),
            })
        }
    }

    pub fn call(&self, _parameters: &[Value]) -> Result<Value> {
        Err(RuntimeError::NotAFunction(self.clone()))
    }
}

pub(crate) fn try_static_eval(exp: &Expression) -> Result<Value> {
    match exp {
        Expression::BinOp { left, op, right } => {
            Ok(op.static_apply(try_static_eval(left)?, try_static_eval(right)?)?)
        }
        Expression::String(_) | Expression::Name(_) | Expression::Call { .. } => {
            Err(RuntimeError::StaticEvaluationFailure(exp.clone()))
        }
        Expression::Number(n) => Ok(Value::Int(*n)),
        Expression::UniOp { op, exp } => Ok(op.static_apply(try_static_eval(exp)?)?),
    }
}

pub(crate) struct Context {}

impl Context {
    pub(crate) fn lookup(&self, name: &Name) -> Result<Value> {
        Err(RuntimeError::NameError(name.clone()))
    }
}
