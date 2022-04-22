use crate::{try_static_eval, EvalResult, ExecutionContext, RuntimeError, Value};
use async_trait::async_trait;
use futures::future::join_all;
use gor_ast::binop::BinOp;
use gor_ast::expression::{Expression, InnerExpression};
use gor_ast::shortcircuitop::ShortCircuitOp;
use gor_ast::uniop::UniOp;
use tokio::join;

pub(crate) trait BinOpExt {
    fn static_apply(&self, l: Value, r: Value) -> EvalResult;
    fn evaluate(&self, left: Value, right: Value) -> EvalResult;
}

impl BinOpExt for BinOp {
    fn static_apply(&self, l: Value, r: Value) -> EvalResult {
        l.bin_op(*self, r)
    }

    fn evaluate(&self, left: Value, right: Value) -> EvalResult {
        self.static_apply(left, right).map_err(Into::into)
    }
}

#[async_trait]
pub(crate) trait ShortCircuitOpExt {
    async fn evaluate<'i>(
        &self,
        left: &Expression<'i>,
        right: &Expression<'i>,
        context: &ExecutionContext,
    ) -> EvalResult;
    fn static_apply(&self, left: Value, right: impl FnOnce() -> EvalResult) -> EvalResult;
}

#[async_trait]
impl ShortCircuitOpExt for ShortCircuitOp {
    async fn evaluate<'i>(
        &self,
        left: &Expression<'i>,
        right: &Expression<'i>,
        context: &ExecutionContext,
    ) -> EvalResult {
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

    fn static_apply(&self, left: Value, right: impl FnOnce() -> EvalResult) -> EvalResult {
        let left = left.as_bool()?;
        match self {
            ShortCircuitOp::LogicalOr => Ok(Value::Boolean(left || right()?.as_bool()?)),
            ShortCircuitOp::LogicalAnd => Ok(Value::Boolean(left && right()?.as_bool()?)),
        }
    }
}

#[async_trait]
pub(crate) trait Evaluable {
    async fn evaluate(&self, context: &ExecutionContext) -> EvalResult;
}

#[async_trait]
impl<'i> Evaluable for Expression<'i> {
    async fn evaluate(&self, context: &ExecutionContext) -> EvalResult {
        if let Ok(r) = try_static_eval(self) {
            return Ok(r);
        }

        Ok(match &self.inner {
            InnerExpression::BinOp { left, op, right } => {
                let left = left.evaluate(context);
                let right = right.evaluate(context);
                let (left, right) = join!(left, right);
                op.evaluate(left?, right?)?
            }
            InnerExpression::ShortCircuitOp { left, op, right } => {
                op.evaluate(left, right, context).await?
            }
            InnerExpression::String(s) => Value::String(s.to_owned()),
            InnerExpression::Number(n) => Value::Int(*n),
            InnerExpression::Name(n) => context.lookup(n)?.clone(),
            InnerExpression::UniOp { op, exp } => op.evaluate(exp.evaluate(context).await?)?,
            InnerExpression::Call { name, parameters } => {
                let parameter_futures: Vec<_> = parameters
                    .iter()
                    .map(|expr| expr.evaluate(context))
                    .collect();
                let mut vector = vec![];
                vector.reserve_exact(parameters.len());
                let parameters = join_all(parameter_futures).await.into_iter().try_fold(
                    vector,
                    |mut r, p| {
                        r.push(p?);
                        Ok(r) as core::result::Result<Vec<Value>, RuntimeError>
                    },
                )?;
                context.lookup(name)?.call(&parameters)?
            }
        })
    }
}

pub(crate) trait UniOpExt {
    fn static_apply(&self, v: Value) -> EvalResult;
    fn evaluate(&self, value: Value) -> EvalResult;
}

impl UniOpExt for UniOp {
    fn static_apply(&self, v: Value) -> EvalResult {
        let v = v.as_int()?;
        let v = match self {
            UniOp::Negate => -v,
        };
        Ok(Value::Int(v))
    }

    fn evaluate(&self, value: Value) -> EvalResult {
        self.static_apply(value).map_err(Into::into)
    }
}
