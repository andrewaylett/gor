use lazy_static::lazy_static;
use std::any::Any;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;

use crate::ast::expression::Expression;
use crate::ast::name::Name;
use crate::error::LuaResult;
use crate::parse::{parse, Rule};

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
    pub const fn as_type(&self) -> Type {
        match self {
            Value::Int(_) => Type::Int,
            Value::String(_) => Type::String,
            Value::Intrinsic(_) => Type::Function,
            Value::Void => Type::Void,
        }
    }

    pub const fn as_int(&self) -> Result<i64> {
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

pub(crate) struct ExecutionContext {
    globals: HashMap<Name, Value>,
}

impl ExecutionContext {
    pub(crate) fn lookup(&self, name: &Name) -> Result<&Value> {
        self.globals
            .get(name)
            .ok_or_else(|| RuntimeError::NameError(*name))
    }
}

lazy_static! {
    pub(crate) static ref GLOBAL_CONTEXT: ExecutionContext = {
        let mut m = HashMap::new();
        m.insert("print".into(), Value::Intrinsic(Intrinsic::Print));
        ExecutionContext { globals: m }
    };
}

pub async fn exec(input: &str) -> LuaResult {
    let p = parse(Rule::expression, input)?;
    let e = Expression::try_from(p)?;
    e.evaluate(&*GLOBAL_CONTEXT).await
}
