#![deny(
    bad_style,
    const_err,
    dead_code,
    improper_ctypes,
    missing_debug_implementations,
    missing_docs,
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

//! An implementation of Go, written as an interpreter in Rust.
//!
//! We provide a binary as well as this library.

mod ast;
mod error;
mod eval;
mod parse;

pub use ast::Located;
pub use error::{LuaError, LuaResult};
pub use eval::exec;
pub use eval::Value;

#[cfg(test)]
mod test {
    use anyhow::Result;
    use pretty_assertions::assert_eq;

    use crate::ast::expression::Expression;
    use crate::eval::test::parse_expression;
    use crate::eval::try_static_eval;
    use crate::eval::Value;

    #[test]
    fn static_eval_int_addition() -> Result<()> {
        let parse = parse_expression("1+2")?;
        let exp: Expression = parse.try_into()?;
        let result = try_static_eval(&exp)?;
        assert_eq!(result, Value::Int(3));
        Ok(())
    }
}
