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

use backtrace::Backtrace;
use pest::iterators::{Pair, Pairs};
use pest::Span;
use thiserror::Error;

use gor_parse::Rule;

/// Errors that may be encountered when generating ASTs
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum AstError {
    /// Our Pest grammar and our AST code don't agree
    ///
    /// Pest documentation suggests that we'd unwrap and panic, but in the interests of nice error messages we wrap and return instead.
    #[error("Invalid Rule attempting to match a {0}: {1:?}, {2:?}")]
    InvalidRuleClass(&'static str, Rule, String),
    #[error("Invalid State During Parse: {0}")]
    InvalidState(&'static str),
    #[error("Invalid State During Parse: {0}")]
    InvalidStateString(String),
    #[error("Parse Rule Mismatch: expected {expected:?}, found {found:?}, at:\n{trace:?}")]
    RuleMismatch {
        expected: Rule,
        found: Rule,
        trace: Box<Backtrace>,
    },
    #[error("{context}:\n{error}")]
    Context {
        error: Box<AstError>,
        context: String,
    },
    #[error("Failed to parse an integer")]
    IntError(#[from] ParseIntError),
}

impl AstError {
    fn with_span(self, span: &Span) -> AstError {
        AstError::Context {
            error: Box::new(self),
            context: format!("With Span `{}`", span.as_str()),
        }
    }

    fn with_rule(self, rule: Rule) -> AstError {
        AstError::Context {
            error: Box::new(self),
            context: format!("With Rule `{:?}`", rule),
        }
    }
}

trait AstErrorContext {
    fn with_span(self, span: &Span) -> Self;
    fn with_rule(self, rule: Rule) -> Self;
}

impl<T> AstErrorContext for Result<T, AstError> {
    fn with_span(self, span: &Span) -> Self {
        self.map_err(|e| e.with_span(span))
    }
    fn with_rule(self, rule: Rule) -> Self {
        self.map_err(|e| e.with_rule(rule))
    }
}

type AstResult<R> = Result<R, AstError>;

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
/// AST for Statements
pub mod statement;
/// The unitary `-` operation
pub mod unitary_op;

fn expect_rule(pair: &Pair<Rule>, expected: Rule) -> AstResult<()> {
    let found = pair.as_rule();
    if found == expected {
        Ok(())
    } else {
        Err(AstError::RuleMismatch {
            expected,
            found,
            trace: Box::new(Backtrace::new()),
        })
    }
}

pub trait Parseable<'i>
where
    Self: Sized,
{
    const RULE: Rule;

    fn build(span: &Span<'i>, pairs: Pairs<'i, Rule>) -> AstResult<Self>;

    fn descend(pair: Pair<'i, Rule>) -> AstResult<Self> {
        let span = pair.as_span();
        Self::build(&span, pair.into_inner())
    }

    fn parse(mut pairs: Pairs<'i, Rule>) -> AstResult<Self> {
        let pair = pairs.next().ok_or(AstError::InvalidState(
            "Expected to get some source, but found nothing to parse",
        ))?;
        let span = pair.as_span();
        expect_rule(&pair, Self::RULE).with_span(&span)?;
        let item = Self::build(&span, pair.into_inner());
        if let Some(pair) = pairs.next() {
            Err(AstError::InvalidStateString(format!(
                "Expected to consume all of the parse: {}",
                pair.as_str()
            )))
        } else {
            Ok(item?)
        }
    }
}

#[cfg(test)]
mod test;
