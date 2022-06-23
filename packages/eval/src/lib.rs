#![deny(
    bad_style,
    const_err,
    dead_code,
    improper_ctypes,
    missing_debug_implementations,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    unconditional_recursion,
    unreachable_pub,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
    clippy::expect_used
)]
#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

use gor_ast::binary_op::BinOp;
use lazy_static::lazy_static;
use std::any::Any;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;

use gor_ast::expression::{Expression, InnerExpression};
use gor_ast::name::Name;
use gor_parse::{parse, ParseError, Rule};
use RuntimeError::{TypeMismatch, TypeOpMismatch};

use crate::extensions::{Evaluable, ShortCircuitOpExt, UniOpExt};
use extensions::BinOpExt;
use gor_ast::AstError;
use gor_core::parse_error::{parse_enum, InternalError};
use gor_linker::{Linker, LinkerError};
use gor_loader::ModuleDescriptor;

#[cfg(test)]
pub mod test;

#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum LanguageFeature {
    ExecutingFunctions,
}

impl TryFrom<&str> for LanguageFeature {
    type Error = InternalError;

    fn try_from(value: &str) -> Result<Self, <Self as TryFrom<&str>>::Error> {
        match value {
            "ExecutingFunctions" => Ok(LanguageFeature::ExecutingFunctions),
            _ => Err(InternalError::Error(format!(
                "Unknown language feature: {}",
                value
            ))),
        }
    }
}

#[derive(Error, Debug, PartialEq)]
#[non_exhaustive]
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
    #[error(transparent)]
    AstError(#[from] AstError),
    #[error(transparent)]
    ParseError(#[from] ParseError),
    #[error(transparent)]
    LinkerError(#[from] LinkerError),
    /// Something happened trying to load the module
    #[error("Unsupported Language Feature: {0:?}")]
    UnsupportedFeature(LanguageFeature),
}

impl TryFrom<&str> for RuntimeError {
    type Error = InternalError;

    fn try_from(value: &str) -> Result<Self, <Self as TryFrom<&str>>::Error> {
        let (name, param) = parse_enum(value)?;
        match name {
            "UnsupportedFeature" => Ok(RuntimeError::UnsupportedFeature(
                LanguageFeature::try_from(param)?,
            )),
            _ => Err(InternalError::Error(format!(
                "Unknown (or unimplemented) RuntimeError variant: {}",
                name
            ))),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[non_exhaustive]
pub enum Type {
    Int,
    Boolean,
    String,
    Function,
    Void,
}

/// A primative value that may be the result of a Go [expression].
///
/// [expression]: ../ast/expression.html
#[derive(Debug, PartialEq, Clone)]
#[non_exhaustive]
pub enum Value {
    /// A 64-bit signed int
    Int(i64),
    /// A boolean
    Boolean(bool),
    /// A string literal
    String(String),
    /// An intrinsic -- globally scoped, known to Rust code.
    Intrinsic(Intrinsic),
    /// The "bottom" type, no value.
    Void,
}

pub type EvalResult = Result<Value, RuntimeError>;
type RuntimeResult<R> = Result<R, RuntimeError>;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[non_exhaustive]
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
    /// Acquire the type of the value
    pub const fn as_type(&self) -> Type {
        match self {
            Value::Int(_) => Type::Int,
            Value::Boolean(_) => Type::Boolean,
            Value::String(_) => Type::String,
            Value::Intrinsic(_) => Type::Function,
            Value::Void => Type::Void,
        }
    }

    /// If this value is able to be represented as a signed integer, return it.
    pub const fn as_int(&self) -> RuntimeResult<i64> {
        match self {
            Value::Int(n) => Ok(*n),
            Value::Boolean(b) => Ok(if *b { 1 } else { 0 }),
            _ => Err(RuntimeError::TypeError {
                expected: Type::Int,
                found: self.as_type(),
            }),
        }
    }

    /// If this value is able to be represented as a boolean, return it.
    pub const fn as_bool(&self) -> RuntimeResult<bool> {
        match self {
            Value::Int(n) => Ok(*n != 0),
            Value::Boolean(b) => Ok(*b),
            _ => Err(RuntimeError::TypeError {
                expected: Type::Boolean,
                found: self.as_type(),
            }),
        }
    }

    /// If this value has function type, apply the parameters to the function
    pub fn call(&self, parameters: &[Value]) -> EvalResult {
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

    /// Attempt to apply `right` to this value using `op`.
    pub fn bin_op(self, op: BinOp, right: Value) -> EvalResult {
        if self.as_type() != right.as_type() {
            return Err(TypeMismatch {
                left: self.as_type(),
                op,
                right: right.as_type(),
            });
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
                    _ => {
                        return Err(TypeOpMismatch {
                            op,
                            r#type: self.as_type(),
                        })
                    }
                })
            }
            _ => Err(TypeOpMismatch {
                op,
                r#type: self.as_type(),
            }),
        }
    }
}

/// Attempt to synchronously evaluate an expression
///
/// If an expression doesn't contain any external references then we can evaluate it without context
/// or any async calls.
pub fn try_static_eval<'i>(exp: &'i Expression<'i>) -> EvalResult {
    match &exp.inner {
        InnerExpression::BinOp { left, op, right } => {
            Ok(op.static_apply(try_static_eval(left)?, try_static_eval(right)?)?)
        }
        InnerExpression::ShortCircuitOp { left, op, right } => {
            Ok(op.static_apply(try_static_eval(left)?, || try_static_eval(right))?)
        }
        InnerExpression::String(_) | InnerExpression::Name(_) | InnerExpression::Call { .. } => {
            Err(RuntimeError::StaticEvaluationFailure(
                exp.span.as_str().to_string(),
            ))
        }
        InnerExpression::Number(n) => Ok(Value::Int(*n)),
        InnerExpression::UniOp { op, exp } => Ok(op.static_apply(try_static_eval(exp)?)?),
    }
}

pub trait ExecutionContext: Sync + Debug {
    fn value(&self, name: Name) -> RuntimeResult<&Value> {
        Err(RuntimeError::NameError(name))
    }

    fn module(&self, name: Name) -> RuntimeResult<&ModuleDescriptor> {
        Err(RuntimeError::NameError(name))
    }
}

#[derive(Debug)]
pub struct GlobalExecutionContext {
    globals: HashMap<Name, Value>,
}

impl ExecutionContext for GlobalExecutionContext {
    fn value(&self, name: Name) -> RuntimeResult<&Value> {
        self.globals.get(&name).ok_or(RuntimeError::NameError(name))
    }
}

impl ExecutionContext for Linker {
    fn module(&self, name: Name) -> RuntimeResult<&ModuleDescriptor> {
        Ok(self.lookup(name)?)
    }
}

lazy_static! {
    pub(crate) static ref GLOBAL_CONTEXT: GlobalExecutionContext = {
        let mut m = HashMap::new();
        m.insert("print".into(), Value::Intrinsic(Intrinsic::Print));
        GlobalExecutionContext { globals: m }
    };
}

/// Parse and execute the given Go ~~module~~ expression
///
/// ```
/// # use gor_eval::EvalResult;
/// # async fn try_main() -> EvalResult {
/// use gor_eval::{Value, exec};
/// let res = exec("2 * 24").await?;
/// assert_eq!(Value::Int(48), res);
/// # Ok(res) // returning from try_main
/// # }
/// # #[tokio::main]
/// # async fn main() {
/// #    try_main().await.unwrap();
/// # }
/// ```
pub async fn exec(input: &str) -> EvalResult {
    let p = parse(Rule::expression, input)?;
    let e = Expression::try_from(p)?;
    e.evaluate(&*GLOBAL_CONTEXT).await
}

#[derive(Debug, Default)]
struct ContextLadder {
    contexts: Vec<Box<dyn ExecutionContext>>,
}

impl ContextLadder {
    fn add(&mut self, context: Box<dyn ExecutionContext>) {
        self.contexts.insert(0, context)
    }

    fn lookup<'i, T, F>(&'i self, name: Name, mut f: F) -> RuntimeResult<&'i T>
    where
        F: FnMut(&'i dyn ExecutionContext, Name) -> RuntimeResult<&'i T>,
    {
        for context in &self.contexts {
            if let Ok(item) = f(context.as_ref(), name) {
                return Ok(item);
            }
        }
        Err(RuntimeError::NameError(name))
    }
}

impl ExecutionContext for ContextLadder {
    fn value(&self, name: Name) -> RuntimeResult<&Value> {
        self.lookup(name, ExecutionContext::value)
    }

    fn module(&self, name: Name) -> RuntimeResult<&ModuleDescriptor> {
        self.lookup(name, ExecutionContext::module)
    }
}

pub async fn execute_in_default_context<T: Into<Name>>(
    linker: Linker,
    module: T,
    fun: T,
) -> EvalResult {
    execute_in_context(linker, ContextLadder::default(), module.into(), fun.into()).await
}

async fn execute_in_context(
    linker: Linker,
    mut context: ContextLadder,
    module: Name,
    fun: Name,
) -> EvalResult {
    context.add(Box::new(linker));
    context
        .module(module)?
        .module()
        .function(fun)
        .ok_or(RuntimeError::NameError(fun))?
        .evaluate(&context)
        .await
}

mod extensions;
