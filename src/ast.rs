use std::num::ParseIntError;

use pest::iterators::{Pair, Pairs};
use thiserror::Error;

use crate::error::LuaError;
use crate::parse::{Rule, PRECEDENCE};
use crate::Value;

#[derive(Error, Debug)]
pub enum AstError {
    #[error("Invalid Rule attempting to match {0}: {1:?}")]
    InvalidRule(&'static str, Rule),
    #[error("Invalid State During Parse: {0}")]
    InvalidState(&'static str),
    #[error("Parse Rule Mismatch: expected {expected:?}, not {found:?}")]
    RuleMismatch { expected: Rule, found: Rule },
    #[error(transparent)]
    IntError(#[from] ParseIntError),
}

type Result<R> = core::result::Result<R, AstError>;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Modulo,
}

impl BinOp {
    pub(crate) fn static_apply(&self, l: Value, r: Value) -> core::result::Result<Value, LuaError> {
        let l = l.as_int()?;
        let r = r.as_int()?;
        let v = match self {
            BinOp::Add => l + r,
            BinOp::Sub => l - r,
            BinOp::Mul => l * r,
            BinOp::Div => l / r,
            BinOp::Pow => l ^ r,
            BinOp::Modulo => l % r,
        };
        Ok(Value::Int(v))
    }
}

impl TryFrom<Rule> for BinOp {
    type Error = AstError;

    fn try_from(value: Rule) -> std::result::Result<Self, Self::Error> {
        Ok(match value {
            Rule::add => BinOp::Add,
            Rule::sub => BinOp::Sub,
            Rule::mul => BinOp::Mul,
            Rule::div => BinOp::Div,
            Rule::pow => BinOp::Pow,
            Rule::modulo => BinOp::Modulo,
            r => return Err(AstError::InvalidRule("BinOp", r)),
        })
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum UniOp {
    Negate,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    BinOp {
        left: Box<Expression>,
        op: BinOp,
        right: Box<Expression>,
    },
    String(String),
    Number(i64),
    Name(String),
    UniOp {
        op: UniOp,
        exp: Box<Expression>,
    },
}

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

impl TryFrom<Pairs<'_, crate::parse::Rule>> for Expression {
    type Error = AstError;

    fn try_from(mut pairs: Pairs<Rule>) -> Result<Self> {
        let expression = pairs.next().ok_or(AstError::InvalidState(
            "Expected to get an expression, but found nothing to parse",
        ))?;
        expect_rule(&expression, Rule::expression)?;
        term_precedence(expression.into_inner())
    }
}

fn term_precedence(pairs: Pairs<Rule>) -> Result<Expression> {
    PRECEDENCE.climb(pairs, term_primary, term_infix)
}

fn term_primary(pair: Pair<Rule>) -> Result<Expression> {
    expect_rule(&pair, Rule::term)?;
    let inner = pair
        .into_inner()
        .next()
        .ok_or(AstError::InvalidState("found term without inner pair"))?;
    match inner.as_rule() {
        Rule::string => Ok(Expression::String(inner.as_str().to_owned())),
        Rule::number => Ok(Expression::Number(inner.as_str().parse()?)),
        r => Err(AstError::InvalidRule("term", r)),
    }
}

fn term_infix(
    left: Result<Expression>,
    op: Pair<Rule>,
    right: Result<Expression>,
) -> Result<Expression> {
    let op = op.as_rule().try_into()?;
    let left = Box::new(left?);
    let right = Box::new(right?);
    Ok(Expression::BinOp { left, op, right })
}
