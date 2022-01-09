use lazy_static::lazy_static;
use pest::error::Error;
use pest::iterators::Pairs;
use pest::prec_climber::{Assoc, Operator, PrecClimber};
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "module.pest"]
pub(crate) struct ModuleParser;

pub(crate) fn parse(rule: Rule, input: &str) -> Result<Pairs<Rule>, Error<Rule>> {
    ModuleParser::parse(rule, input)
}

lazy_static! {
    pub static ref PRECEDENCE: PrecClimber<Rule> = PrecClimber::new(vec![
        Operator::new(Rule::add, Assoc::Left) | Operator::new(Rule::sub, Assoc::Left),
        Operator::new(Rule::mul, Assoc::Left)
            | Operator::new(Rule::div, Assoc::Left)
            | Operator::new(Rule::modulo, Assoc::Left),
        Operator::new(Rule::pow, Assoc::Right)
    ]);
}

#[cfg(test)]
pub(crate) mod test {
    use anyhow::Result;
    use pest::iterators::Pairs;

    use crate::parse::parse;
    use crate::parse::Rule;
    use crate::{try_static_eval, Expression, Value};

    pub fn parse_expression(input: &str) -> Result<Pairs<Rule>> {
        Ok(parse(Rule::expression, input)?)
    }

    #[test]
    fn parse_int_add() -> Result<()> {
        parse(Rule::expression, "1+2")?;
        Ok(())
    }

    #[test]
    fn multiply_higher_precedence_than_add() -> Result<()> {
        let p = parse_expression("2+3*4")?;
        let e = Expression::try_from(p)?;
        let r = try_static_eval(&e)?;
        assert_eq!(Value::Int(14), r, "Expression was {:?}", &e);
        Ok(())
    }
}
