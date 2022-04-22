use crate::expression::Expression;
use crate::{expect_rule, AstError, AstResult, Located};
use gor_parse::Rule;
use pest::iterators::Pair;
use pest::Span;

#[allow(unused)]
#[derive(Debug)]
pub enum InnerStatement<'i> {
    Expression(Expression<'i>),
    Assignment,
    Func,
}

#[allow(unused)]
#[derive(Debug)]
struct Statement<'i> {
    inner: InnerStatement<'i>,
    span: Span<'i>,
}

impl<'i> TryFrom<Pair<'i, Rule>> for Statement<'i> {
    type Error = AstError;
    fn try_from(pair: Pair<'i, Rule>) -> AstResult<Self> {
        expect_rule(&pair, Rule::statement)?;
        let span = pair.as_span();
        let rule = pair.as_rule();
        let inner = pair.into_inner();
        let inner_statement = match rule {
            Rule::expression => InnerStatement::Expression(Expression::try_from(inner)?),
            Rule::assignment => InnerStatement::Assignment,
            Rule::func => InnerStatement::Func,
            r => return Err(AstError::InvalidRule("term", r)),
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
