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
#![deny(unsafe_code)]

use crate::ast::Expression;
use crate::error::Result;
use crate::eval::Value;

mod ast;
mod parse;

pub mod error {
    use thiserror::Error;

    use crate::ast::AstError;
    use crate::eval::Type;

    #[derive(Error, Debug)]
    pub enum LuaError {
        #[error(transparent)]
        AstError(#[from] AstError),
        #[error("Type Mismatch: expected {expected:?}, not {found:?}")]
        TypeError { expected: Type, found: Type },
        #[error(transparent)]
        PestError(#[from] pest::error::Error<crate::parse::Rule>),
        #[error("Unknown Lua Error")]
        Unknown,
    }

    pub type Result<R> = core::result::Result<R, LuaError>;
}

mod eval {
    use crate::error::{LuaError, Result};

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
}

pub fn try_static_eval(exp: &Expression) -> Result<Value> {
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

pub async fn exec(input: &str) -> Result<Value> {
    let p = parse::parse(parse::Rule::expression, input)?;
    let e = Expression::try_from(p)?;
    try_static_eval(&e)
}

#[cfg(test)]
mod test {
    use anyhow::Result;

    use crate::parse::test::parse_expression;
    use crate::{try_static_eval, Expression, Value};

    #[test]
    fn static_eval_int_addition() -> Result<()> {
        let parse = parse_expression("1+2")?;
        let exp: Expression = parse.try_into()?;
        let result = try_static_eval(&exp)?;
        assert_eq!(result, Value::Int(3));
        Ok(())
    }
}
