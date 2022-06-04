use crate::AstError;
use gor_parse::Rule;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ShortCircuitOp {
    LogicalOr,
    LogicalAnd,
}

impl TryFrom<Rule> for ShortCircuitOp {
    type Error = AstError;

    fn try_from(value: Rule) -> Result<Self, Self::Error> {
        Ok(match value {
            Rule::bool_and => ShortCircuitOp::LogicalAnd,
            Rule::bool_or => ShortCircuitOp::LogicalOr,
            r => return Err(AstError::InvalidRule("ShortCircuitOp", r)),
        })
    }
}
