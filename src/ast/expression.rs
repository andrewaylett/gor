use crate::ast::binop::BinOp;
use crate::ast::name::Name;
use crate::ast::uniop::UniOp;
use crate::ast::Result;
use crate::ast::{expect_rule, AstError};
use crate::error::LuaError;
use crate::error::LuaResult;
use crate::eval::Value;
use crate::eval::{try_static_eval, Context};
use crate::parse::Rule;
use crate::parse::PRECEDENCE;
use async_recursion::async_recursion;
use futures::future::join_all;
use pest::iterators::{Pair, Pairs};
use tokio::join;

#[must_use = "expressions are side-effect free unless evaluated"]
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    BinOp {
        left: Box<Expression>,
        op: BinOp,
        right: Box<Expression>,
    },
    String(String),
    Number(i64),
    Name(Name),
    UniOp {
        op: UniOp,
        exp: Box<Expression>,
    },
    Call {
        name: Name,
        parameters: Vec<Expression>,
    },
}

impl TryFrom<Pairs<'_, crate::parse::Rule>> for Expression {
    type Error = AstError;

    fn try_from(mut pairs: Pairs<Rule>) -> super::Result<Self> {
        let expression = pairs.next().ok_or(AstError::InvalidState(
            "Expected to get an expression, but found nothing to parse",
        ))?;
        Expression::try_from(expression)
    }
}

impl TryFrom<Pair<'_, crate::parse::Rule>> for Expression {
    type Error = AstError;

    fn try_from(expression: Pair<crate::parse::Rule>) -> Result<Self> {
        expect_rule(&expression, Rule::expression)?;
        term_precedence(expression.into_inner())
    }
}

fn term_precedence(pairs: Pairs<Rule>) -> Result<Expression> {
    PRECEDENCE.climb(pairs, term_primary, term_infix)
}

fn term_primary(pair: Pair<Rule>) -> Result<Expression> {
    expect_rule(&pair, Rule::term)?;
    let inner = pair.into_inner();
    let next = inner
        .peek()
        .ok_or(AstError::InvalidState("found term without inner pair"))?;
    match next.as_rule() {
        Rule::string => {
            let string_inner = next
                .into_inner()
                .find(|p| p.as_rule() == Rule::string_inner)
                .ok_or(AstError::InvalidState(
                    "Found a string without a string_inner",
                ))?;
            Ok(Expression::String(string_inner.as_str().to_owned()))
        }
        Rule::number => Ok(Expression::Number(next.as_str().parse()?)),
        Rule::expression => Ok(Expression::try_from(inner)?),
        Rule::name => Ok(Expression::Name(Name(next.as_str().to_owned()))),
        Rule::uniop => {
            let expr = next.into_inner();
            Ok(Expression::UniOp {
                op: UniOp::Negate,
                exp: Box::new(Expression::try_from(expr)?),
            })
        }
        Rule::call => {
            let mut call = next.into_inner();
            let name = call
                .next()
                .ok_or(AstError::InvalidState("Found a call with no name"))?;
            expect_rule(&name, Rule::name)?;
            let name = Name(name.as_str().to_owned());
            let parameters = call.try_fold(vec![], |mut result, expression| {
                result.push(Expression::try_from(expression)?);
                Ok(result) as Result<Vec<Expression>>
            })?;
            Ok(Expression::Call { name, parameters })
        }
        r => Err(AstError::InvalidRule("term", r)),
    }
}

fn term_infix(
    left: Result<Expression>,
    op: Pair<Rule>,
    right: Result<Expression>,
) -> Result<Expression> {
    let op = op.as_rule().try_into()?;
    let left = Box::new(left?);
    let right = Box::new(right?);
    Ok(Expression::BinOp { left, op, right })
}

impl Expression {
    #[async_recursion]
    pub(crate) async fn evaluate(&self, context: &Context) -> LuaResult {
        if let Ok(r) = try_static_eval(self) {
            return Ok(r);
        }

        Ok(match self {
            Expression::BinOp { left, op, right } => {
                let left = left.evaluate(context);
                let right = right.evaluate(context);
                let (left, right) = join!(left, right);
                op.evaluate(left?, right?)?
            }
            Expression::String(s) => Value::String(s.to_owned()),
            Expression::Number(n) => Value::Int(*n),
            Expression::Name(n) => context.lookup(n)?.clone(),
            Expression::UniOp { op, exp } => op.evaluate(exp.evaluate(context).await?)?,
            Expression::Call { name, parameters } => {
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
