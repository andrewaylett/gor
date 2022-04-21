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
    ($rule:ident) => {
        Operator::new(Rule::$rule, Assoc::Left)
    };
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
        l!(dot),
        l!(bool_or),
        l!(bool_and),
        l!(eq) | l!(neq) | l!(lt) | l!(leq) | l!(gt) | l!(geq),
        l!(add) | l!(sub) | l!(bit_or) | l!(bit_xor),
        l!(mul) | l!(div) | l!(modulo) | l!(shl) | l!(shr) | l!(bit_and) | l!(bit_clear),
    ]);
}
