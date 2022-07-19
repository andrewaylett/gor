use crate::name::Name;
use crate::statement::Statement;
use crate::{expect_rule, AstError, AstResult, Located};
use gor_core::{Function, Member};
use gor_parse::Rule;
use pest::iterators::Pair;
use pest::Span;

#[allow(unused)]
#[derive(Debug)]
pub struct SourceFunction<'i> {
    pub(crate) name: Name,
    signature: Signature<'i>,
    body: Body<'i>,
    span: Span<'i>,
}

impl<'i> TryFrom<Pair<'i, Rule>> for SourceFunction<'i> {
    type Error = AstError;

    fn try_from(pair: Pair<'i, Rule>) -> AstResult<Self> {
        let span = pair.as_span();
        let mut inner = pair.into_inner();
        let next = inner
            .next()
            .ok_or(AstError::InvalidState("No name in func"))?;
        expect_rule(&next, Rule::name)?;
        let name = next.try_into()?;
        let next = inner
            .next()
            .ok_or(AstError::InvalidState("No params in func"))?;
        expect_rule(&next, Rule::signature)?;
        let signature = next.try_into()?;
        let next = inner
            .next()
            .ok_or(AstError::InvalidState("No body in func"))?;
        expect_rule(&next, Rule::block)?;
        let body = next.try_into()?;
        Ok(SourceFunction {
            name,
            signature,
            body,
            span,
        })
    }
}

impl<'i> Member for SourceFunction<'i> {}
impl<'i> Function<'i> for SourceFunction<'i> {}

#[derive(Debug)]
pub struct Parameter<'i> {
    span: Span<'i>,
}

impl<'i> TryFrom<Pair<'i, Rule>> for Parameter<'i> {
    type Error = AstError;
    fn try_from(pair: Pair<'i, Rule>) -> AstResult<Self> {
        expect_rule(&pair, Rule::param)?;
        unimplemented!("Nothing needs parameters yet")
    }
}

impl<'i> Located<'i> for Parameter<'i> {
    fn as_span(&self) -> Span<'i> {
        self.span.clone()
    }
}

#[allow(unused)]
#[derive(Debug)]
pub struct Parameters<'i> {
    parameters: Vec<Parameter<'i>>,
    span: Span<'i>,
}

impl<'i> TryFrom<Pair<'i, Rule>> for Parameters<'i> {
    type Error = AstError;

    fn try_from(pair: Pair<'i, Rule>) -> AstResult<Self> {
        expect_rule(&pair, Rule::params)?;
        let span = pair.as_span();
        let parameters = pair
            .into_inner()
            .map(Parameter::try_from)
            .collect::<AstResult<Vec<Parameter>>>()?;
        Ok(Self { span, parameters })
    }
}

impl<'i> Located<'i> for Parameters<'i> {
    fn as_span(&self) -> Span<'i> {
        self.span.clone()
    }
}

#[allow(unused)]
#[derive(Debug)]
pub struct Signature<'i> {
    parameters: Parameters<'i>,
    span: Span<'i>,
}

impl<'i> TryFrom<Pair<'i, Rule>> for Signature<'i> {
    type Error = AstError;

    fn try_from(pair: Pair<'i, Rule>) -> AstResult<Self> {
        expect_rule(&pair, Rule::signature)?;
        let span = pair.as_span();
        let mut inner = pair.into_inner();
        let next = inner
            .next()
            .ok_or(AstError::InvalidState("No parameters in signature"))?;
        let parameters = Parameters::try_from(next)?;
        Ok(Self { span, parameters })
    }
}

impl<'i> Located<'i> for Signature<'i> {
    fn as_span(&self) -> Span<'i> {
        self.span.clone()
    }
}

#[derive(Debug)]
pub struct Body<'i> {
    #[allow(dead_code)]
    statements: Vec<Statement<'i>>,
    span: Span<'i>,
}

impl<'i> TryFrom<Pair<'i, Rule>> for Body<'i> {
    type Error = AstError;
    fn try_from(pair: Pair<'i, Rule>) -> AstResult<Self> {
        expect_rule(&pair, Rule::block)?;
        let span = pair.as_span();

        let inner = pair.into_inner();
        let statements = inner
            .map(Statement::try_from)
            .collect::<AstResult<Vec<Statement>>>()?;
        Ok(Body { statements, span })
    }
}

impl<'i> Located<'i> for Body<'i> {
    fn as_span(&self) -> Span<'i> {
        self.span.clone()
    }
}
