use lazy_static::lazy_static;
use std::any::Any;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;
use crate::ast::binop::BinOp;

use crate::ast::expression::Expression;
use crate::ast::Located;
use crate::ast::name::Name;
use crate::error::LuaResult;
use crate::eval::RuntimeError::{TypeMismatch, TypeOpMismatch};
use crate::parse::{parse, Rule};

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Not a function: {0:?}")]
    NotAFunction(Value),
    #[error("Name not found: {0}")]
    NameError(Name),
    #[error("Can't static eval {0:?}")]
    StaticEvaluationFailure(String),
    #[error("Type Mismatch: expected {expected:?}, not {found:?}")]
    TypeError { expected: Type, found: Type },
    #[error("Type Mismatch: expected the same type, found {left:?} {op:?} {right:?}")]
    TypeMismatch { left: Type, op: BinOp, right: Type },
    #[error("Can't {op:?} on {r#type:?}")]
    TypeOpMismatch { op: BinOp, r#type: Type },
}

type Result<R> = core::result::Result<R, RuntimeError>;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Type {
    Int,
    Boolean,
    String,
    Function,
    Void,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Int(i64),
    Boolean(bool),
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
            Value::Boolean(b) => Display::fmt(&b, f),
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
            Value::Boolean(_) => Type::Boolean,
            Value::String(_) => Type::String,
            Value::Intrinsic(_) => Type::Function,
            Value::Void => Type::Void,
        }
    }

    pub const fn as_int(&self) -> Result<i64> {
        match self {
            Value::Int(n) => Ok(*n),
            Value::Boolean(b) => Ok(if *b { 1 } else { 0 }),
            _ => Err(RuntimeError::TypeError {
                expected: Type::Int,
                found: self.as_type(),
            }),
        }
    }

    pub const fn as_bool(&self) -> Result<bool> {
        match self {
            Value::Int(n) => Ok(*n != 0),
            Value::Boolean(b) => Ok(*b),
            _ => Err(RuntimeError::TypeError {
                expected: Type::Boolean,
                found: self.as_type(),
            }),
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

    pub fn bin_op(self, op: BinOp, right: Value) -> Result<Value> {
        if self.as_type() != right.as_type() {
            return Err(TypeMismatch { left: self.as_type(), op, right: right.as_type() });
        }
        match self.as_type() {
            Type::Int => {
                let left = self.as_int()?;
                let right = right.as_int()?;
                Ok(match op {
                    BinOp::Eq => Value::Boolean(left == right),
                    BinOp::Neq => Value::Boolean(left != right),
                    BinOp::Lt => Value::Boolean(left < right),
                    BinOp::Leq => Value::Boolean(left <= right),
                    BinOp::Gt => Value::Boolean(left > right),
                    BinOp::Geq => Value::Boolean(left >= right),
                    BinOp::Add => Value::Int(left + right),
                    BinOp::Sub => Value::Int(left - right),
                    BinOp::BitOr => Value::Int(left | right),
                    BinOp::BitXor => Value::Int(left ^ right),
                    BinOp::Mul => Value::Int(left * right),
                    BinOp::Div => Value::Int(left / right),
                    BinOp::Modulo => Value::Int(left % right),
                    BinOp::Shl => Value::Int(left << right),
                    BinOp::Shr => Value::Int(left >> right),
                    BinOp::BitAnd => Value::Int(left & right),
                    BinOp::BitClear => Value::Int(left & !right),
                })
            }
            Type::Boolean => {
                let left = self.as_bool()?;
                let right = right.as_bool()?;
                Ok(match op {
                    BinOp::Eq => Value::Boolean(left == right),
                    BinOp::Neq => Value::Boolean(left != right),
                    BinOp::Lt => Value::Boolean(!left & right),
                    BinOp::Leq => Value::Boolean(left <= right),
                    BinOp::Gt => Value::Boolean(left & !right),
                    BinOp::Geq => Value::Boolean(left >= right),
                    BinOp::BitOr => Value::Boolean(left | right),
                    BinOp::BitXor => Value::Boolean(left ^ right),
                    BinOp::BitAnd => Value::Boolean(left & right),
                    BinOp::BitClear => Value::Boolean(left & !right),
                    _ => return Err(TypeOpMismatch { op, r#type: self.as_type() }),
                })
            }
            _ => Err(TypeOpMismatch { op, r#type: self.as_type() })
        }
    }
}

pub(crate) fn try_static_eval<'i>(exp: &'i Located<'i, Expression<'i>>) -> Result<Value> {
    match &exp.item {
        Expression::BinOp { left, op, right } => {
            Ok(op.static_apply(try_static_eval(left)?, try_static_eval(right)?)?)
        }
        Expression::ShortCircuitOp { left, op, right } => {
            Ok(op.static_apply(try_static_eval(left)?, || try_static_eval(right))?)
        }
        Expression::String(_) | Expression::Name(_) | Expression::Call { .. } => {
            Err(RuntimeError::StaticEvaluationFailure(exp.span.as_str().to_string()))
        }
        Expression::Number(n) => Ok(Value::Int(*n)),
        Expression::UniOp { op, exp } => Ok(op.static_apply(try_static_eval(exp)?)?),
    }
}

#[derive(Debug)]
pub struct ExecutionContext {
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
    let e = Located::try_from(p)?;
    e.evaluate(&*GLOBAL_CONTEXT).await
}
