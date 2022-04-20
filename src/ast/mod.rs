use std::fmt::Debug;
use std::num::ParseIntError;

use pest::iterators::Pair;
use pest::Span;
use thiserror::Error;

use crate::parse::Rule;

#[derive(Error, Debug)]
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
mod test {
    use crate::ast::expression::{Expression, InnerExpression};
    use crate::parse::test::parse_expression;
    use anyhow::{anyhow, Result};
    use crate::ast::name::Name;
    use crate::ast::shortcircuitop::ShortCircuitOp;

    #[test]
    fn parse_name() -> Result<()> {
        let p = parse_expression("foo")?;
        let e = Expression::try_from(p)?;
        assert_eq!(InnerExpression::Name("foo".into()), e.inner);
        Ok(())
    }

    #[test]
    fn parse_string() -> Result<()> {
        let p = parse_expression("\"foo\"")?;
        let e = Expression::try_from(p)?;
        assert_eq!(InnerExpression::String("foo".to_owned()), e.inner);
        Ok(())
    }

    #[test]
    fn parse_with_escaped_quote() -> Result<()> {
        let p = parse_expression(r#""f\"oo""#)?;
        let e = Expression::try_from(p)?;
        assert_eq!(InnerExpression::String(r#"f\"oo"#.to_owned()), e.inner);
        Ok(())
    }

    #[test]
    fn parse_with_escaped_slash() -> Result<()> {
        let p = parse_expression(r##""foo\\""##)?;
        let e = Expression::try_from(p)?;
        assert_eq!(InnerExpression::String("foo\\\\".to_owned()), e.inner);
        Ok(())
    }

    #[test]
    fn parse_call() -> Result<()> {
        let p = parse_expression("foo()")?;
        let e = Expression::try_from(p)?;
        assert_eq!(
            InnerExpression::Call {
                name: "foo".into(),
                parameters: vec![]
            },
            e.inner
        );
        Ok(())
    }

    #[test]
    fn parse_call_with_params() -> Result<()> {
        let p = parse_expression("foo(1,2)")?;
        let e = Expression::try_from(p)?;
        if let InnerExpression::Call {name, parameters} = e.inner {
            assert_eq!(Name::from("foo"), name);
            assert_eq!(2, parameters.len());
            assert_eq!(InnerExpression::Number(1), parameters[0].inner);
            assert_eq!(InnerExpression::Number(2), parameters[1].inner);
            Ok(())
        } else {
            Err(anyhow!("Expected a Call: {:?}", e))
        }
    }

    macro_rules! parse_binop {
        ($name:ident, $input:literal, $left:literal, $op:ident, $right:literal) => {
            #[test]
            fn $name() -> Result<()> {
                let p = parse_expression($input)?;
                let e = Expression::try_from(p)?;
                if let InnerExpression::ShortCircuitOp {left, op, right} = e.inner {
                    assert_eq!(InnerExpression::Number($left), left.inner);
                    assert_eq!(ShortCircuitOp::$op, op);
                    assert_eq!(InnerExpression::Number($right), right.inner);
                    Ok(())
                } else {
                    Err(anyhow!("Expected {}: {:?}", $input, e))
                }
            }
        }
    }

    parse_binop!(parse_bool_or, "1 || 2", 1, LogicalOr, 2);
    parse_binop!(parse_bool_and, "1 && 2", 1, LogicalAnd, 2);
}
