use lazy_static::lazy_static;
use std::any::Any;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
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
    Function,
    Void,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Int(i64),
    String(String),
    Intrinsic(Intrinsic),
    Void,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Intrinsic {
    Print,
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(n) => Display::fmt(&n, f),
            Value::String(s) => Display::fmt(&s, f),
            Value::Intrinsic(n) => Debug::fmt(&n.type_id(), f),
            Value::Void => Display::fmt("<void>", f),
        }
    }
}

impl Value {
    pub fn as_type(&self) -> Type {
        match self {
            Value::Int(_) => Type::Int,
            Value::String(_) => Type::String,
            Value::Intrinsic(_) => Type::Function,
            Value::Void => Type::Void,
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

    pub fn call(&self, parameters: &[Value]) -> Result<Value> {
        if let Value::Intrinsic(function) = self {
            match function {
                Intrinsic::Print => {
                    parameters.iter().for_each(|v| print!("{}", v));
                    println!();
                    Ok(Value::Void)
                }
            }
        } else {
            Err(RuntimeError::NotAFunction(self.clone()))
        }
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

pub(crate) struct Context<'a> {
    values: HashMap<&'a str, Value>,
}

impl Context<'_> {
    pub(crate) fn lookup(&self, name: &Name) -> Result<&Value> {
        self.values
            .get(name.to_str())
            .ok_or_else(|| RuntimeError::NameError(name.clone()))
    }
}

lazy_static! {
    pub(crate) static ref GLOBAL_CONTEXT: Context<'static> = {
        let mut m = HashMap::new();
        m.insert("print", Value::Intrinsic(Intrinsic::Print));
        Context { values: m }
    };
}
