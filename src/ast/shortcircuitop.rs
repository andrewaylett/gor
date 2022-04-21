use crate::ast::expression::Expression;
use crate::ast::AstError;
use crate::error::GoResult;
use crate::eval::{ExecutionContext, RuntimeError};
use crate::parse::Rule;
use crate::Value;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum ShortCircuitOp {
    LogicalOr,
    LogicalAnd,
}

impl ShortCircuitOp {
    pub(crate) async fn evaluate<'i>(
        &self,
        left: &Expression<'i>,
        right: &Expression<'i>,
        context: &ExecutionContext,
    ) -> GoResult {
        let left = left.evaluate(context).await?;
        let left = left.as_bool()?;
        match self {
            ShortCircuitOp::LogicalOr => Ok(Value::Boolean(
                left || right.evaluate(context).await?.as_bool()?,
            )),
            ShortCircuitOp::LogicalAnd => Ok(Value::Boolean(
                left && right.evaluate(context).await?.as_bool()?,
            )),
        }
    }

    pub(crate) fn static_apply(
        &self,
        left: Value,
        right: impl FnOnce() -> Result<Value, RuntimeError>,
    ) -> Result<Value, RuntimeError> {
        let left = left.as_bool()?;
        match self {
            ShortCircuitOp::LogicalOr => Ok(Value::Boolean(left || right()?.as_bool()?)),
            ShortCircuitOp::LogicalAnd => Ok(Value::Boolean(left && right()?.as_bool()?)),
        }
    }
}

impl TryFrom<Rule> for ShortCircuitOp {
    type Error = AstError;

    fn try_from(value: Rule) -> std::result::Result<Self, Self::Error> {
        Ok(match value {
            Rule::bool_and => ShortCircuitOp::LogicalAnd,
            Rule::bool_or => ShortCircuitOp::LogicalOr,
            r => return Err(AstError::InvalidRule("ShortCircuitOp", r)),
        })
    }
}
