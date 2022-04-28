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

use std::fmt::Debug;
use std::num::ParseIntError;

use pest::iterators::Pair;
use pest::Span;
use thiserror::Error;

use gor_parse::Rule;

/// Errors that may be encountered when generating ASTs
#[derive(Error, Debug, PartialEq)]
pub enum AstError {
    /// Our Pest grammar and our AST code don't agree
    ///
    /// Pest documentation suggests that we'd unwrap and panic, but in the interests of nice error messages we wrap and return instead.
    #[error("Invalid Rule attempting to match {0}: {1:?}")]
    InvalidRule(&'static str, Rule),
    #[error("Invalid State During Parse: {0}")]
    InvalidState(&'static str),
    #[error("Invalid State During Parse: {0}")]
    InvalidStateString(String),
    #[error("Parse Rule Mismatch: expected {expected:?}, not {found:?}")]
    RuleMismatch { expected: Rule, found: Rule },
    #[error("Failed to parse an integer")]
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

/// Operations with exactly two inputs
///
/// Currently all are infix operators.
pub mod binary_op;
/// AST for Expressions
pub mod expression;
/// AST for Functions
pub mod func;
/// AST for Modules
pub mod module;
/// An interned String usable as a name
pub mod name;
/// Short-circuiting binary operations
pub mod short_circuit_op;
/// AST for Statements
pub mod statement;
/// The unitary `-` operation
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
