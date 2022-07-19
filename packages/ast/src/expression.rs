use crate::binary_op::BinOp;
use crate::name::Name;
use crate::unitary_op::UniOp;
use crate::{expect_rule, AstError};
use crate::{AstErrorContext, AstResult, Parseable};
use backtrace::Backtrace;
use gor_parse::Rule;
use gor_parse::PRECEDENCE;
use pest::iterators::{Pair, Pairs};
use pest::Span;

#[must_use = "expressions are side-effect free unless evaluated"]
#[derive(Debug, Clone, PartialEq)]
pub struct Expression<'i> {
    pub inner: InnerExpression<'i>,
    pub span: Span<'i>,
}

impl Expression<'_> {
    const fn new<'i>(span: Span<'i>, inner: InnerExpression<'i>) -> Expression<'i> {
        Expression { span, inner }
    }
}

#[must_use = "expressions are side-effect free unless evaluated"]
#[derive(Debug, Clone, PartialEq)]
pub enum InnerExpression<'i> {
    BinOp {
        left: Box<Expression<'i>>,
        op: BinOp,
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

impl<'i> Parseable<'i> for Expression<'i> {
    const RULE: Rule = Rule::expression;
    fn build(span: &Span<'i>, pairs: Pairs<'i, Rule>) -> AstResult<Expression<'i>> {
        let exp = term_precedence(pairs)?;
        if &exp.span == span {
            Ok(exp)
        } else {
            Err(AstError::InvalidStateString(format!(
                "Expected the parsed expression {:?} to cover the input {:?}",
                exp.span, span
            )))
        }
    }
}

fn term_precedence(pairs: Pairs<Rule>) -> AstResult<Expression> {
    PRECEDENCE.climb(pairs, term_primary, term_infix)
}

fn term_primary(pair: Pair<Rule>) -> AstResult<Expression> {
    expect_rule(&pair, Rule::term)?;
    let span = pair.as_span();
    let inner = pair.into_inner();
    let next = inner
        .peek()
        .ok_or(AstError::InvalidState("found term without inner pair"))?;
    let rule = next.as_rule();
    let expr: InnerExpression = next_to_inner(next).with_span(&span).with_rule(rule)?;
    Ok(Expression::new(span, expr))
}

fn next_to_inner(next: Pair<Rule>) -> AstResult<InnerExpression> {
    Ok(match next.as_rule() {
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
        Rule::expression => Expression::descend(next)?.inner,
        Rule::name => InnerExpression::Name(next.try_into()?),
        Rule::unitary_op => InnerExpression::UniOp {
            op: UniOp::Negate,
            exp: Box::new(Expression::parse(next.into_inner())?),
        },
        Rule::call => {
            let mut call = next.into_inner();
            let name = call
                .next()
                .ok_or(AstError::InvalidState("Found a call with no name"))?;
            expect_rule(&name, Rule::name)?;
            let name: Name = name.as_str().into();
            let parameters: Vec<Expression> = call
                .map(|expression| Ok(Expression::descend(expression)?) as AstResult<Expression>)
                .collect::<AstResult<_>>()?;
            InnerExpression::Call { name, parameters }
        }
        r => {
            return Err(AstError::RuleMismatch {
                expected: Rule::term,
                found: r,
                trace: Box::new(Backtrace::new()),
            })
        }
    })
}

fn term_infix<'i>(
    left: AstResult<Expression<'i>>,
    op: Pair<'i, Rule>,
    right: AstResult<Expression<'i>>,
) -> AstResult<Expression<'i>> {
    let left = left.with_rule(op.as_rule())?;
    let right = right.with_rule(op.as_rule())?;
    let start = left.span.start_pos();
    let end = right.span.end_pos();
    let span = start.span(&end);
    let expr = op.as_str().to_string();
    let op = op.as_rule();
    let left = Box::new(left);
    let right = Box::new(right);
    if let Ok(op) = op.try_into() {
        Ok(Expression::new(
            span,
            InnerExpression::BinOp { left, op, right },
        ))
    } else {
        Err(AstError::InvalidRuleClass("BinOp", op, expr))
    }
}
