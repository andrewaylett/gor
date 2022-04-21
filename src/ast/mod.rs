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

pub trait Located {
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
