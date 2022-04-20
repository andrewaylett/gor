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

#[derive(Debug, Clone, PartialEq)]
pub struct Located<'i, R> where R: Debug+ Clone+ PartialEq {
    pub(crate) span: Span<'i>,
    pub item: R,
}

impl <'i, R> Located<'i, R> where R: Debug+ Clone+ PartialEq {
    fn new(span: Span<'i>, item: R) -> Located<'i, R> {
        Located {
            span,
            item,
        }
    }
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
    use crate::ast::expression::Expression;
    use crate::parse::test::parse_expression;
    use anyhow::{anyhow, Result};
    use crate::ast::name::Name;
    use crate::ast::shortcircuitop::ShortCircuitOp;
    use crate::Located;

    #[test]
    fn parse_name() -> Result<()> {
        let p = parse_expression("foo")?;
        let e = Located::try_from(p)?;
        assert_eq!(Expression::Name("foo".into()), e.item);
        Ok(())
    }

    #[test]
    fn parse_string() -> Result<()> {
        let p = parse_expression("\"foo\"")?;
        let e = Located::try_from(p)?;
        assert_eq!(Expression::String("foo".to_owned()), e.item);
        Ok(())
    }

    #[test]
    fn parse_with_escaped_quote() -> Result<()> {
        let p = parse_expression(r#""f\"oo""#)?;
        let e = Located::try_from(p)?;
        assert_eq!(Expression::String(r#"f\"oo"#.to_owned()), e.item);
        Ok(())
    }

    #[test]
    fn parse_with_escaped_slash() -> Result<()> {
        let p = parse_expression(r##""foo\\""##)?;
        let e = Located::try_from(p)?;
        assert_eq!(Expression::String("foo\\\\".to_owned()), e.item);
        Ok(())
    }

    #[test]
    fn parse_call() -> Result<()> {
        let p = parse_expression("foo()")?;
        let e = Located::try_from(p)?;
        assert_eq!(
            Expression::Call {
                name: "foo".into(),
                parameters: vec![]
            },
            e.item
        );
        Ok(())
    }

    #[test]
    fn parse_call_with_params() -> Result<()> {
        let p = parse_expression("foo(1,2)")?;
        let e = Located::try_from(p)?;
        if let Expression::Call {name, parameters} = e.item {
            assert_eq!(Name::from("foo"), name);
            assert_eq!(2, parameters.len());
            assert_eq!(Expression::Number(1), parameters[0].item);
            assert_eq!(Expression::Number(2), parameters[1].item);
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
                let e = Located::try_from(p)?;
                if let Expression::ShortCircuitOp {left, op, right} = e.item {
                    assert_eq!(Expression::Number($left), left.item);
                    assert_eq!(ShortCircuitOp::$op, op);
                    assert_eq!(Expression::Number($right), right.item);
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
