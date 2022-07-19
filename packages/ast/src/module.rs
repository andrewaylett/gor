use crate::func::SourceFunction;
use crate::name::Name;
use crate::{AstError, AstResult, Parseable};
use gor_parse::Rule;
use pest::iterators::Pairs;
use pest::Span;
use std::collections::HashMap;
use std::fmt::Debug;

#[allow(dead_code)]
#[derive(Debug)]
pub struct SourceModule<'i> {
    pub package: Name,
    pub imports: Vec<Name>,
    functions: HashMap<Name, Box<SourceFunction<'i>>>,
}

impl<'s: 'i, 'i> Parseable<'s> for SourceModule<'i> {
    const RULE: Rule = Rule::module;

    fn build(_span: &Span<'s>, pairs: Pairs<'s, Rule>) -> AstResult<Self> {
        primary(pairs)
    }
}

impl<'i> SourceModule<'i> {
    pub fn function(&self, name: Name) -> Option<&SourceFunction<'i>> {
        self.functions.get(&name).map(|b| b.as_ref())
    }
}

fn primary<'s: 'i, 'i>(module: Pairs<'s, Rule>) -> AstResult<SourceModule<'i>> {
    let mut package = None;
    let mut imports = vec![];
    let mut functions: HashMap<Name, Box<SourceFunction<'i>>> = HashMap::new();
    for pair in module {
        match pair.as_rule() {
            Rule::package => {
                let name = pair
                    .into_inner()
                    .find(|p| p.as_rule() == Rule::name)
                    .ok_or(AstError::InvalidState(
                        "Found a package declaration without a name",
                    ))?;
                package = Some(Name::from(name.as_str()));
            }
            Rule::import => {
                let string = pair
                    .into_inner()
                    .find(|p| p.as_rule() == Rule::string)
                    .ok_or(AstError::InvalidState("Found an import without a package"))?;
                let name = string
                    .into_inner()
                    .find(|p| p.as_rule() == Rule::string_inner)
                    .ok_or(AstError::InvalidState(
                        "Found an import string without an inner",
                    ))?;
                imports.push(Name::from(name.as_str()));
            }
            Rule::func => {
                let func = SourceFunction::descend(pair)?;
                functions.insert(func.name, Box::new(func));
            }
            Rule::EOI => {}
            r => {
                return Err(AstError::InvalidRuleClass(
                    "module contents",
                    r,
                    pair.as_str().to_string(),
                ))
            }
        }
    }
    match package {
        None => Err(AstError::InvalidState("Module must have package set")),
        Some(package) => Ok(SourceModule {
            package,
            imports,
            functions,
        }),
    }
}
