use crate::ast::binop::BinOp;
use crate::ast::name::Name;
use crate::ast::shortcircuitop::ShortCircuitOp;
use crate::ast::uniop::UniOp;
use crate::ast::{Result};
use crate::ast::{expect_rule, AstError};
use crate::error::LuaError;
use crate::error::LuaResult;
use crate::eval::Value;
use crate::eval::{try_static_eval, ExecutionContext};
use crate::parse::Rule;
use crate::parse::PRECEDENCE;
use async_recursion::async_recursion;
use futures::future::join_all;
use pest::iterators::{Pair, Pairs};
use pest::Span;
use tokio::join;

#[must_use = "expressions are side-effect free unless evaluated"]
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Expression<'i> {
    pub(crate) inner: InnerExpression<'i>,
    pub(crate) span: Span<'i>,
}

impl Expression<'_> {
    fn new<'i>(span: Span<'i>, inner: InnerExpression<'i>) -> Expression<'i> {
        Expression { span, inner }
    }
}

#[must_use = "expressions are side-effect free unless evaluated"]
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum InnerExpression<'i> {
    BinOp {
        left: Box<Expression<'i>>,
        op: BinOp,
        right: Box<Expression<'i>>,
    },
    ShortCircuitOp {
        left: Box<Expression<'i>>,
        op: ShortCircuitOp,
        right: Box<Expression<'i>>,
    },
    String(String),
    Number(i64),
    Name(Name),
    UniOp {
        op: UniOp,
        exp: Box<Expression<'i>>,
    },
    Call {
        name: Name,
        parameters: Vec<Expression<'i>>,
    },
}

impl <'i> TryFrom<Pairs<'i, Rule>> for Expression<'i> {
    type Error = AstError;

    fn try_from(mut pairs: Pairs<'i, Rule>) -> super::Result<Self> {
        let expression = pairs.next().ok_or(AstError::InvalidState(
            "Expected to get an expression, but found nothing to parse",
        ))?;
        let span = expression.as_span();
        let item = InnerExpression::try_from(expression);
        if pairs.next().is_some() {
            Err(AstError::InvalidState("Expected to consume all of the parse"))
        } else {
            Ok(Expression::new(span, item?))
        }
    }
}

impl <'i> TryFrom<Pair<'i, Rule>> for InnerExpression<'i> {
    type Error = AstError;

    fn try_from(expression: Pair<'i, Rule>) -> Result<Self> {
        expect_rule(&expression, Rule::expression)?;
        let span = expression.as_span();
        let item = term_precedence(expression.into_inner())?;
        if span == item.span {
            Ok(item.inner)
        } else {
            Err(AstError::InvalidStateString(format!("Expected the parsed expression {:?} to cover the input {:?}", item.span, span)))
        }
    }
}

fn term_precedence(pairs: Pairs<Rule>) -> Result<Expression> {
    PRECEDENCE.climb(pairs, term_primary, term_infix)
}

fn term_primary(pair: Pair<Rule>) -> Result<Expression> {
    expect_rule(&pair, Rule::term)?;
    let span = pair.as_span();
    let inner = pair.into_inner();
    let next = inner
        .peek()
        .ok_or(AstError::InvalidState("found term without inner pair"))?;
    let expr: InnerExpression = match next.as_rule() {
        Rule::string => {
            let string_inner = next
                .into_inner()
                .find(|p| p.as_rule() == Rule::string_inner)
                .ok_or(AstError::InvalidState(
                    "Found a string without a string_inner",
                ))?;
            InnerExpression::String(string_inner.as_str().to_owned())
        }
        Rule::number => InnerExpression::Number(next.as_str().parse()?),
        Rule::expression => {
            InnerExpression::try_from(next)?
        },
        Rule::name => InnerExpression::Name(next.as_str().into()),
        Rule::unitary_op => {
            let expr = next.into_inner();
            InnerExpression::UniOp {
                op: UniOp::Negate,
                exp: Box::new(Expression::try_from(expr)?),
            }
        }
        Rule::call => {
            let mut call = next.into_inner();
            let name = call
                .next()
                .ok_or(AstError::InvalidState("Found a call with no name"))?;
            expect_rule(&name, Rule::name)?;
            let name: Name = name.as_str().into();
            let parameters = call.try_fold(vec![], |mut result, expression| {
                let span = expression.as_span();
                result.push(Expression::new(span, InnerExpression::try_from(expression)?));
                Ok(result) as Result<Vec<Expression>>
            })?;
            InnerExpression::Call { name, parameters }
        }
        r => return Err(AstError::InvalidRule("term", r)),
    };
    Ok(Expression::new(span, expr))
}

fn term_infix<'i>(
    left: Result<Expression<'i>>,
    op: Pair<'i, Rule>,
    right: Result<Expression<'i>>,
) -> Result<Expression<'i>> {
    let left = left?;
    let right = right?;
    let start = left.span.start_pos();
    let end = right.span.end_pos();
    let span = start.span(&end);
    let op = op.as_rule();
    let left = Box::new(left);
    let right = Box::new(right);
    match op {
        Rule::bool_and | Rule::bool_or => {
            let op = op.try_into()?;
            Ok(Expression::new(span, InnerExpression::ShortCircuitOp { left, op, right }))
        }
        _ => {
            let op = op.try_into()?;
            Ok(Expression::new(span, InnerExpression::BinOp { left, op, right }))
        }
    }
}

impl <'i> Expression<'i> {
    #[async_recursion]
    pub(crate) async fn evaluate(&self, context: &ExecutionContext) -> LuaResult {
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
                        Ok(r) as core::result::Result<Vec<Value>, LuaError>
                    },
                )?;
                context.lookup(name)?.call(&parameters)?
            }
        })
    }
}
