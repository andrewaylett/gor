use anyhow::{anyhow, Context, Result};
use pest::iterators::{Pair, Pairs};

use crate::parse::{Rule, PRECEDENCE};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Modulo,
}

impl TryFrom<Rule> for BinOp {
    type Error = anyhow::Error;

    fn try_from(value: Rule) -> std::result::Result<Self, Self::Error> {
        Ok(match value {
            Rule::add => BinOp::Add,
            Rule::sub => BinOp::Sub,
            Rule::mul => BinOp::Mul,
            Rule::div => BinOp::Div,
            Rule::pow => BinOp::Pow,
            Rule::modulo => BinOp::Modulo,
            r => return Err(anyhow!("Not a BinOp rule: {:?}", r)),
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
        Err(anyhow!(
            "Expected a {:?} but got a {:?}",
            rule,
            pair.as_rule()
        ))
    }
}

impl TryFrom<Pairs<'_, crate::parse::Rule>> for Expression {
    type Error = anyhow::Error;

    fn try_from(mut pairs: Pairs<Rule>) -> Result<Self> {
        let expression = pairs
            .next()
            .context("Expected an expression, got no pairs")?;
        expect_rule(&expression, Rule::expression)?;
        term_precedence(expression.into_inner())
    }
}

fn term_precedence(pairs: Pairs<Rule>) -> Result<Expression> {
    PRECEDENCE.climb(pairs, term_primary, term_infix)
}

fn term_primary(pair: Pair<Rule>) -> Result<Expression> {
    expect_rule(&pair, Rule::term)?;
    let inner = pair.into_inner().next().context("term contains a pair")?;
    match inner.as_rule() {
        Rule::string => Ok(Expression::String(inner.as_str().to_owned())),
        Rule::number => Ok(Expression::Number(inner.as_str().parse()?)),
        r => Err(anyhow!(
            "Not a rule that we expect to appear in a term: {:?}",
            r
        )),
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
