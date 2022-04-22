use crate::expression::{Expression, InnerExpression};
use crate::name::Name;
use crate::shortcircuitop::ShortCircuitOp;
use anyhow::{anyhow, Context, Result};
use gor_parse::{parse, Rule};
use pest::iterators::Pairs;
use pretty_assertions::assert_eq;

#[track_caller]
fn parse_expression(input: &str) -> Result<Pairs<Rule>> {
    let p = parse(Rule::expression, input)?;
    let first = p.peek().context("Expected a parse")?;
    assert_eq!(first.as_span().start(), 0);
    assert_eq!(first.as_span().end(), input.len());
    Ok(p)
}

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
    if let InnerExpression::Call { name, parameters } = e.inner {
        assert_eq!(Name::from("foo"), name);
        assert_eq!(2, parameters.len());
        assert_eq!(InnerExpression::Number(1), parameters[0].inner);
        assert_eq!(InnerExpression::Number(2), parameters[1].inner);
        Ok(())
    } else {
        Err(anyhow!("Expected a Call: {:?}", e))
    }
}

macro_rules! parse_short_circuit_binop {
    ($name:ident, $input:literal, $left:literal, $op:ident, $right:literal) => {
        #[test]
        fn $name() -> Result<()> {
            let p = parse_expression($input)?;
            let e = Expression::try_from(p)?;
            if let InnerExpression::ShortCircuitOp { left, op, right } = e.inner {
                assert_eq!(InnerExpression::Number($left), left.inner);
                assert_eq!(ShortCircuitOp::$op, op);
                assert_eq!(InnerExpression::Number($right), right.inner);
                Ok(())
            } else {
                Err(anyhow!("Expected {}: {:?}", $input, e))
            }
        }
    };
}

parse_short_circuit_binop!(parse_bool_or, "1 || 2", 1, LogicalOr, 2);
parse_short_circuit_binop!(parse_bool_and, "1 && 2", 1, LogicalAnd, 2);

#[allow(non_snake_case)]
mod binop {
    use super::parse_expression;
    use crate::binop::BinOp;
    use crate::expression::{Expression, InnerExpression};
    use anyhow::{anyhow, Result};

    macro_rules! parse_binop {
        ($input:literal, $op:ident) => {
            #[test]
            fn $op() -> Result<()> {
                let p = parse_expression($input)?;
                let e = Expression::try_from(p)?;
                if let InnerExpression::BinOp { left, op, right } = e.inner {
                    assert_eq!(InnerExpression::Number(1), left.inner);
                    assert_eq!(BinOp::$op, op);
                    assert_eq!(InnerExpression::Number(2), right.inner);
                    Ok(())
                } else {
                    Err(anyhow!("Expected {}: {:?}", $input, e))
                }
            }
        };
    }

    parse_binop!("1 << 2", Shl);
    parse_binop!("1 >> 2", Shr);
    parse_binop!("1 <= 2", Leq);
    parse_binop!("1 >= 2", Geq);
    parse_binop!("1 == 2", Eq);
    parse_binop!("1 != 2", Neq);
    parse_binop!("1 < 2", Lt);
    parse_binop!("1 > 2", Gt);
    parse_binop!("1 + 2", Add);
    parse_binop!("1 - 2", Sub);
    parse_binop!("1 | 2", BitOr);
    parse_binop!("1 ^ 2", BitXor);
    parse_binop!("1 * 2", Mul);
    parse_binop!("1 / 2", Div);
    parse_binop!("1 % 2", Modulo);
    parse_binop!("1 &^ 2", BitClear);
    parse_binop!("1 & 2", BitAnd);
}
