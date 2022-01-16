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
use crate::eval::{Value, GLOBAL_CONTEXT};

mod ast;
pub mod eval;
mod parse;

pub mod error {
    use thiserror::Error;

    use crate::ast::AstError;
    use crate::eval::RuntimeError;

    #[derive(Error, Debug)]
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

    pub type Result<R> = core::result::Result<R, LuaError>;
}

pub async fn exec(input: &str) -> Result<Value> {
    let p = parse::parse(parse::Rule::expression, input)?;
    let e = Expression::try_from(p)?;
    e.evaluate(core::ops::Deref::deref(&GLOBAL_CONTEXT)).await
}
#[cfg(test)]
mod test {
    use anyhow::Result;

    use crate::eval::try_static_eval;
    use crate::parse::test::parse_expression;
    use crate::{Expression, Value};

    #[test]
    fn static_eval_int_addition() -> Result<()> {
        let parse = parse_expression("1+2")?;
        let exp: Expression = parse.try_into()?;
        let result = try_static_eval(&exp)?;
        assert_eq!(result, Value::Int(3));
        Ok(())
    }
}
