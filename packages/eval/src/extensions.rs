use crate::LanguageFeature::ExecutingFunctions;
use crate::{try_static_eval, EvalResult, ExecutionContext, RuntimeError, Value};
use async_trait::async_trait;
use futures::future::join_all;
use gor_ast::binary_op::BinOp;
use gor_ast::expression::{Expression, InnerExpression};
use gor_ast::func::SourceFunction;
use gor_ast::unitary_op::UniOp;
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
        context: &dyn ExecutionContext,
    ) -> EvalResult;
    fn static_apply(&self, left: Value, right: impl FnOnce() -> EvalResult) -> EvalResult;
}

#[async_trait]
pub(crate) trait Evaluable {
    async fn evaluate(&self, context: &dyn ExecutionContext) -> EvalResult;
}

#[async_trait]
impl<'i> Evaluable for Expression<'i> {
    async fn evaluate(&self, context: &dyn ExecutionContext) -> EvalResult {
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
            InnerExpression::String(s) => Value::String(s.to_owned()),
            InnerExpression::Number(n) => Value::Int(*n),
            InnerExpression::Name(n) => context.value(*n)?.clone(),
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
                        Ok(r) as Result<Vec<Value>, RuntimeError>
                    },
                )?;
                context.value(*name)?.call(&parameters)?
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

#[async_trait]
impl Evaluable for SourceFunction<'_> {
    async fn evaluate(&self, _: &dyn ExecutionContext) -> EvalResult {
        Err(RuntimeError::UnsupportedFeature(ExecutingFunctions))
    }
}
