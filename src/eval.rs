use crate::ast::Name;
use crate::error::{LuaError, Result};
use crate::Expression;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Type {
    Int,
    String,
}

#[derive(Debug, PartialEq)]
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
            Err(LuaError::TypeError {
                expected: Type::Int,
                found: self.as_type(),
            })
        }
    }
}

pub(crate) fn try_static_eval(exp: &Expression) -> Result<Value> {
    Ok(match exp {
        Expression::BinOp { left, op, right } => {
            op.static_apply(try_static_eval(left)?, try_static_eval(right)?)?
        }
        Expression::String(_) => {
            todo!()
        }
        Expression::Number(n) => Value::Int(*n),
        Expression::Name(_) => {
            todo!()
        }
        Expression::UniOp { .. } => {
            todo!()
        }
    })
}

pub(crate) struct Context {}

impl Context {
    pub(crate) fn lookup(&self, name: &Name) -> Result<Value> {
        Err(LuaError::NameError(name.clone()))
    }
}
