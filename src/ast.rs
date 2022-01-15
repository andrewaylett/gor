use std::fmt::{Debug, Display, Formatter};
use std::num::ParseIntError;

use async_recursion::async_recursion;
use pest::iterators::{Pair, Pairs};
use thiserror::Error;
use tokio::join;

use crate::eval::{try_static_eval, Context, RuntimeError};
use crate::parse::{Rule, PRECEDENCE};
use crate::Result as LuaResult;
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
    pub(crate) fn static_apply(
        &self,
        l: Value,
        r: Value,
    ) -> core::result::Result<Value, RuntimeError> {
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

    pub(crate) fn evaluate(&self, left: Value, right: Value) -> LuaResult<Value> {
        self.static_apply(left, right).map_err(Into::into)
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

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum UniOp {
    Negate,
}

impl UniOp {
    pub(crate) fn static_apply(&self, v: Value) -> core::result::Result<Value, RuntimeError> {
        let v = v.as_int()?;
        let v = match self {
            UniOp::Negate => -v,
        };
        Ok(Value::Int(v))
    }

    pub(crate) fn evaluate(&self, value: Value) -> LuaResult<Value> {
        self.static_apply(value).map_err(Into::into)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    BinOp {
        left: Box<Expression>,
        op: BinOp,
        right: Box<Expression>,
    },
    String(String),
    Number(i64),
    Name(Name),
    UniOp {
        op: UniOp,
        exp: Box<Expression>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Name(String);

impl Display for Name {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
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
    let inner = pair.into_inner();
    let next = inner
        .peek()
        .ok_or(AstError::InvalidState("found term without inner pair"))?;
    match next.as_rule() {
        Rule::string => {
            let string_inner = next
                .into_inner()
                .find(|p| p.as_rule() == Rule::string_inner)
                .ok_or(AstError::InvalidState(
                    "Found a string without a string_inner",
                ))?;
            Ok(Expression::String(string_inner.as_str().to_owned()))
        }
        Rule::number => Ok(Expression::Number(next.as_str().parse()?)),
        Rule::expression => Ok(Expression::try_from(inner)?),
        Rule::name => Ok(Expression::Name(Name(next.as_str().to_owned()))),
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

impl Expression {
    #[async_recursion]
    pub(crate) async fn evaluate(&self, context: &Context) -> LuaResult<Value> {
        if let Ok(r) = try_static_eval(self) {
            return Ok(r);
        }

        Ok(match self {
            Expression::BinOp { left, op, right } => {
                let left = left.evaluate(context);
                let right = right.evaluate(context);
                let (left, right) = join!(left, right);
                op.evaluate(left?, right?)?
            }
            Expression::String(s) => Value::String(s.to_owned()),
            Expression::Number(n) => Value::Int(*n),
            Expression::Name(n) => context.lookup(n)?,
            Expression::UniOp { op, exp } => op.evaluate(exp.evaluate(context).await?)?,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::ast::Name;
    use crate::parse::test::parse_expression;
    use crate::Expression;
    use anyhow::Result;

    #[test]
    fn parse_name() -> Result<()> {
        let p = parse_expression("foo")?;
        let e = Expression::try_from(p)?;
        assert_eq!(Expression::Name(Name("foo".to_owned())), e);
        Ok(())
    }

    #[test]
    fn parse_string() -> Result<()> {
        let p = parse_expression("\"foo\"")?;
        let e = Expression::try_from(p)?;
        assert_eq!(Expression::String("foo".to_owned()), e);
        Ok(())
    }

    #[test]
    fn parse_with_escaped_quote() -> Result<()> {
        let p = parse_expression(r#""f\"oo""#)?;
        let e = Expression::try_from(p)?;
        assert_eq!(Expression::String(r#"f\"oo"#.to_owned()), e);
        Ok(())
    }

    #[test]
    fn parse_with_escaped_slash() -> Result<()> {
        let p = parse_expression(r##""foo\\""##)?;
        let e = Expression::try_from(p)?;
        assert_eq!(Expression::String("foo\\\\".to_owned()), e);
        Ok(())
    }
}
