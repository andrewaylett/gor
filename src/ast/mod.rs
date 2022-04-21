use std::fmt::Debug;
use std::num::ParseIntError;

use pest::iterators::Pair;
use pest::Span;
use thiserror::Error;

use crate::parse::Rule;

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

type Result<R> = core::result::Result<R, AstError>;

/// Indicates an element is derived from source.
///
/// You may obtain the relevant source for objects that implement this trait, for example in order
/// to print pretty errors.
pub trait Located {
    /// The [Pest] [Span] that represents the source for this object.
    ///
    /// [Pest]: https://docs.rs/pest/latest/pest/
    /// [Span]: https://docs.rs/pest/latest/pest/struct.Span.html
    fn as_span(&self) -> Span;
}

pub(crate) mod binop;
pub(crate) mod expression;
pub(crate) mod name;
mod shortcircuitop;
mod uniop;

fn expect_rule(pair: &Pair<Rule>, rule: Rule) -> Result<()> {
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
