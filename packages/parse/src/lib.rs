#![deny(
    bad_style,
    const_err,
    dead_code,
    improper_ctypes,
    missing_debug_implementations,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    unconditional_recursion,
    unreachable_pub,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
    clippy::expect_used
)]
#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

use lazy_static::lazy_static;
use pest::iterators::Pairs;
use pest::prec_climber::{Assoc, Operator, PrecClimber};
use pest::Parser;
use thiserror::Error;

use implementation::ModuleParser;

mod implementation {
    use pest_derive::Parser;
    #[derive(Parser)]
    #[grammar = "module.pest"]
    pub(crate) struct ModuleParser;
}

pub use implementation::Rule;

#[derive(Error, Debug, PartialEq)]
pub enum ParseError {
    #[error(transparent)]
    PestError(#[from] pest::error::Error<Rule>),
}

pub fn parse(rule: Rule, input: &str) -> Result<Pairs<Rule>, ParseError> {
    Ok(ModuleParser::parse(rule, input)?)
}

macro_rules! l {
    ($rule:ident) => {
        Operator::new(Rule::$rule, Assoc::Left)
    };
}

lazy_static! {
    /// Go operator precedence
    ///
    /// Per https://go.dev/ref/spec#Operator_precedence
    ///
    /// | Precedence | Operator                  |
    /// |------------|---------------------------|
    /// |    5       |    *  /  %  <<  >>  &  &^ |
    /// |    4       |    +  -  |  ^             |
    /// |    3       |    ==  !=  <  <=  >  >=   |
    /// |    2       |    &&                     |
    /// |    1       |    ||                     |
    ///
    /// Gór adds `.` as the highest precedence binary operator for AST parsing
    pub static ref PRECEDENCE: PrecClimber<Rule> = PrecClimber::new(vec![
        l!(dot),
        l!(bool_or),
        l!(bool_and),
        l!(eq) | l!(neq) | l!(lt) | l!(leq) | l!(gt) | l!(geq),
        l!(add) | l!(sub) | l!(bit_or) | l!(bit_xor),
        l!(mul) | l!(div) | l!(modulo) | l!(shl) | l!(shr) | l!(bit_and) | l!(bit_clear),
    ]);
}
