use pest::error::Error;
use pest::iterators::Pairs;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "module.pest"]
pub struct ModuleParser;

pub fn parse(rule: Rule, input: &str) -> Result<Pairs<Rule>, Error<Rule>> {
    ModuleParser::parse(rule, input)
}

#[cfg(test)]
pub(crate) mod test {
    use anyhow::Result;
    use pest::iterators::Pairs;

    use crate::parse::parse;
    use crate::parse::Rule;

    pub fn parse_expression(input: &str) -> Result<Pairs<Rule>> {
        Ok(parse(Rule::expression, input)?)
    }

    #[test]
    fn parse_int_add() -> Result<()> {
        parse(Rule::expression, "1+2")?;
        Ok(())
    }
}
