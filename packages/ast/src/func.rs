use crate::name::Name;
use crate::statement::Statement;
use crate::{expect_rule, AstError, AstResult, Located, Parseable};
use gor_core::{Function, Member};
use gor_parse::Rule;
use pest::iterators::Pairs;
use pest::Span;

#[allow(unused)]
#[derive(Debug)]
pub struct SourceFunction<'i> {
    pub(crate) name: Name,
    signature: Signature<'i>,
    body: Body<'i>,
    span: Span<'i>,
}

impl<'i> Parseable<'i> for SourceFunction<'i> {
    const RULE: Rule = Rule::func;

    fn build(span: &Span<'i>, pairs: Pairs<'i, Rule>) -> AstResult<Self> {
        let mut pairs = pairs;
        let next = pairs
            .next()
            .ok_or(AstError::InvalidState("No name in func"))?;
        expect_rule(&next, Rule::name)?;
        let name = Name::descend(next)?;
        let next = pairs
            .next()
            .ok_or(AstError::InvalidState("No params in func"))?;
        expect_rule(&next, Rule::signature)?;
        let signature = Signature::descend(next)?;
        let next = pairs
            .next()
            .ok_or(AstError::InvalidState("No body in func"))?;
        expect_rule(&next, Rule::block)?;
        let body = Body::descend(next)?;
        Ok(SourceFunction {
            name,
            signature,
            body,
            span: span.clone(),
        })
    }
}

impl<'i> Member for SourceFunction<'i> {}
impl<'i> Function<'i> for SourceFunction<'i> {}

#[derive(Debug)]
pub struct Parameter<'i> {
    span: Span<'i>,
}

impl<'i> Parseable<'i> for Parameter<'i> {
    const RULE: Rule = Rule::param;

    fn build(_span: &Span<'i>, _pairs: Pairs<'i, Rule>) -> AstResult<Self> {
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

impl<'i> Parseable<'i> for Parameters<'i> {
    const RULE: Rule = Rule::params;

    fn build(span: &Span<'i>, pairs: Pairs<'i, Rule>) -> AstResult<Self> {
        let parameters = pairs
            .map(Parameter::descend)
            .collect::<AstResult<Vec<Parameter>>>()?;
        Ok(Self {
            span: span.clone(),
            parameters,
        })
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

impl<'i> Parseable<'i> for Signature<'i> {
    const RULE: Rule = Rule::signature;

    fn build(span: &Span<'i>, pairs: Pairs<'i, Rule>) -> AstResult<Self> {
        let mut pairs = pairs;
        let next = pairs
            .next()
            .ok_or(AstError::InvalidState("No parameters in signature"))?;
        let parameters = Parameters::descend(next)?;
        Ok(Self {
            span: span.clone(),
            parameters,
        })
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

impl<'i> Parseable<'i> for Body<'i> {
    const RULE: Rule = Rule::block;

    fn build(span: &Span<'i>, pairs: Pairs<'i, Rule>) -> AstResult<Self> {
        let statements = pairs
            .map(Statement::descend)
            .collect::<AstResult<Vec<Statement>>>()?;
        Ok(Body {
            statements,
            span: span.clone(),
        })
    }
}

impl<'i> Located<'i> for Body<'i> {
    fn as_span(&self) -> Span<'i> {
        self.span.clone()
    }
}
