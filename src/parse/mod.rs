use lazy_static::lazy_static;
use pest::error::Error;
use pest::iterators::Pairs;
use pest::prec_climber::{Assoc, Operator, PrecClimber};
use pest::Parser;

use crate::parse::implementation::ModuleParser;

mod implementation {
    use pest_derive::Parser;
    #[derive(Parser)]
    #[grammar = "module.pest"]
    pub(crate) struct ModuleParser;
}

pub(crate) use implementation::Rule;

pub(crate) fn parse(rule: Rule, input: &str) -> Result<Pairs<Rule>, Error<Rule>> {
    ModuleParser::parse(rule, input)
}

macro_rules! l {
    ($rule:ident) => {Operator::new(Rule::$rule, Assoc::Left)}
}

lazy_static! {
    /// Per https://go.dev/ref/spec#Operator_precedence
    ///
    /// +------------+---------------------------+
    /// | Precedence | Operator                  |
    /// +------------+---------------------------+
    //  |    5       |    *  /  %  <<  >>  &  &^ |
    //  |    4       |    +  -  |  ^             |
    //  |    3       |    ==  !=  <  <=  >  >=   |
    //  |    2       |    &&                     |
    //  |    1       |    ||                     |
    /// +------------+---------------------------+
    pub static ref PRECEDENCE: PrecClimber<Rule> = PrecClimber::new(vec![
        l!(bool_or),
        l!(bool_and),
        l!(eq) | l!(neq) | l!(lt) | l!(leq) | l!(gt) | l!(geq),
        l!(add) | l!(sub) | l!(bit_or) | l!(bit_xor),
        l!(mul) | l!(div) | l!(modulo) | l!(shl) | l!(shr) | l!(bit_and) | l!(bit_clear),
    ]);
}

#[cfg(test)]
pub(crate) mod test {
    use crate::ast::expression::Expression;
    use anyhow::{Context, Result};
    use pest::iterators::Pairs;
    use crate::ast::Located;

    use crate::eval::{try_static_eval, Value};
    use crate::parse::parse;
    use crate::parse::Rule;

    #[track_caller]
    pub(crate) fn assert_expression(expected: Value, expression: &Located<Expression>) {
        let r = try_static_eval(expression).unwrap();
        assert_eq!(expected, r, "Expression was {:?} => {:?}", expression.span, expression.item);
    }

    #[track_caller]
    pub(crate) fn parse_expression(input: &str) -> Result<Pairs<Rule>> {
        let p = parse(Rule::expression, input)?;
        let first = p.peek().context("Expected a parse")?;
        assert_eq!(first.as_span().start(), 0);
        assert_eq!(first.as_span().end(), input.len());
        Ok(p)
    }

    macro_rules! test_eval {
        ($func_name:ident, $input:expr, $result:expr) => {
            #[test]
            fn $func_name() -> Result<()> {
                let p = parse_expression($input)?;
                let e = Located::try_from(p)?;
                assert_expression($result, &e);
                Ok(())
            }
        }
    }

    macro_rules! test_eval_int {
        ($func_name:ident, $input:expr) => {
            test_eval!($func_name, stringify!($input), Value::Int($input));
        }
    }

    test_eval_int!(int_add, 1 + 2);
    test_eval_int!(multiply_higher_precedence_than_add, 2+3*4);
    test_eval_int!(parens, (1+2)*3);
    test_eval_int!(negative, -1);
    test_eval_int!(bit_and, 6 & 3);
    test_eval_int!(bit_or, 1 | 2);
    test_eval_int!(shl, 13 << 20);
    test_eval_int!(shr, 100000 >> 10);
    test_eval_int!(xor, 6 ^ 10);

    test_eval!(bit_nand, "6 &^ 10", Value::Int(4));

    test_eval!(bit_xor, "6 ^ 10", Value::Int(12));
}
