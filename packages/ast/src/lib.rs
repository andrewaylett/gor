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

use std::fmt::Debug;
use std::num::ParseIntError;

use pest::iterators::Pair;
use pest::Span;
use thiserror::Error;

use gor_parse::Rule;

#[derive(Error, Debug, PartialEq)]
pub enum AstError {
    #[error("Invalid Rule attempting to match {0}: {1:?}")]
    InvalidRule(&'static str, Rule),
    #[error("Invalid State During Parse: {0}")]
    InvalidState(&'static str),
    #[error("Invalid State During Parse: {0}")]
    InvalidStateString(String),
    #[error("Parse Rule Mismatch: expected {expected:?}, not {found:?}")]
    RuleMismatch { expected: Rule, found: Rule },
    #[error(transparent)]
    IntError(#[from] ParseIntError),
}

type AstResult<R> = core::result::Result<R, AstError>;

/// Indicates an element is derived from source.
///
/// You may obtain the relevant source for objects that implement this trait, for example in order
/// to print pretty errors.
pub trait Located<'i> {
    /// The [Pest] [Span] that represents the source for this object.
    ///
    /// [Pest]: https://docs.rs/pest/latest/pest/
    /// [Span]: https://docs.rs/pest/latest/pest/struct.Span.html
    fn as_span(&self) -> Span<'i>;
}

pub mod binary_op;
pub mod expression;
pub mod func;
pub mod module;
pub mod name;
pub mod short_circuit_op;
pub mod statement;
pub mod unitary_op;

fn expect_rule(pair: &Pair<Rule>, rule: Rule) -> AstResult<()> {
    if pair.as_rule() == rule {
        Ok(())
    } else {
        Err(AstError::RuleMismatch {
            expected: rule,
            found: pair.as_rule(),
        })
    }
}

#[cfg(test)]
mod test;
