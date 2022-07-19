use crate::expression::Expression;
use crate::{expect_rule, AstError, AstResult, Located, Parseable};
use gor_parse::Rule;
use pest::iterators::Pair;
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

impl<'i> TryFrom<Pair<'i, Rule>> for Statement<'i> {
    type Error = AstError;
    fn try_from(pair: Pair<'i, Rule>) -> AstResult<Self> {
        expect_rule(&pair, Rule::statement)?;
        let span = pair.as_span();
        let debug_expr = span.as_str().to_string();
        let mut inner = pair.into_inner();
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
            span,
        })
    }
}

impl<'i> Located<'i> for Statement<'i> {
    fn as_span(&self) -> Span<'i> {
        self.span.clone()
    }
}
