use crate::expression::Expression;
use crate::{AstError, AstResult, Located, Parseable};
use gor_parse::Rule;
use pest::iterators::Pairs;
use pest::Span;

#[derive(Debug)]
pub enum InnerStatement<'i> {
    Expression(Expression<'i>),
    Assignment,
    Func,
}

#[derive(Debug)]
pub struct Statement<'i> {
    #[allow(dead_code)]
    inner: InnerStatement<'i>,
    span: Span<'i>,
}

impl<'i> Parseable<'i> for Statement<'i> {
    const RULE: Rule = Rule::statement;

    fn build(span: &Span<'i>, pairs: Pairs<'i, Rule>) -> AstResult<Self> {
        let debug_expr = span.as_str().to_string();
        let mut inner = pairs;
        let next = inner
            .next()
            .ok_or(AstError::InvalidState("No parameters in signature"))?;
        let inner_statement = match next.as_rule() {
            Rule::expression => InnerStatement::Expression(Expression::descend(next)?),
            Rule::assignment => InnerStatement::Assignment,
            Rule::func => InnerStatement::Func,
            r => {
                return Err(AstError::InvalidRuleClass(
                    "expression, assignment, func",
                    r,
                    debug_expr,
                ))
            }
        };
        Ok(Statement {
            inner: inner_statement,
            span: span.clone(),
        })
    }
}

impl<'i> Located<'i> for Statement<'i> {
    fn as_span(&self) -> Span<'i> {
        self.span.clone()
    }
}
