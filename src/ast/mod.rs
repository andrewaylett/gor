use std::fmt::Debug;
use std::num::ParseIntError;

use pest::iterators::Pair;
use thiserror::Error;

use crate::parse::Rule;

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

mod binop;
pub(crate) mod expression;
pub(crate) mod name;
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
    use anyhow::Result;

    #[test]
    fn parse_name() -> Result<()> {
        let p = parse_expression("foo")?;
        let e = Expression::try_from(p)?;
        assert_eq!(Expression::Name("foo".into()), e);
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

    #[test]
    fn parse_call() -> Result<()> {
        let p = parse_expression("foo()")?;
        let e = Expression::try_from(p)?;
        assert_eq!(
            Expression::Call {
                name: "foo".into(),
                parameters: vec![]
            },
            e
        );
        Ok(())
    }

    #[test]
    fn parse_call_with_params() -> Result<()> {
        let p = parse_expression("foo(1,2)")?;
        let e = Expression::try_from(p)?;
        assert_eq!(
            Expression::Call {
                name: "foo".into(),
                parameters: vec![Expression::Number(1), Expression::Number(2)]
            },
            e
        );
        Ok(())
    }
}
