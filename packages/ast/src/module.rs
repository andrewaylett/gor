use crate::func::SourceFunction;
use crate::name::Name;
use crate::{expect_rule, AstError, AstResult};
use gor_parse::Rule;
use pest::iterators::{Pair, Pairs};
use std::collections::HashMap;
use std::fmt::Debug;

pub(crate) trait Member: Debug {}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Module<'i> {
    package: Name,
    imports: Vec<Name>,
    members: HashMap<Name, Box<dyn Member + 'i>>,
}

impl<'i> TryFrom<Pairs<'i, Rule>> for Module<'i> {
    type Error = AstError;

    fn try_from(mut pairs: Pairs<'i, Rule>) -> super::AstResult<Self> {
        let pair = pairs.next().ok_or(AstError::InvalidState(
            "Expected to get a module, but found nothing to parse",
        ))?;
        let item = Module::try_from(pair);
        if pairs.next().is_some() {
            Err(AstError::InvalidState(
                "Expected to consume all of the parse",
            ))
        } else {
            item
        }
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for Module<'i> {
    type Error = AstError;

    fn try_from(module: Pair<'i, Rule>) -> AstResult<Self> {
        expect_rule(&module, Rule::module)?;
        primary(module)
    }
}

fn primary<'i>(module: Pair<'i, Rule>) -> AstResult<Module<'i>> {
    expect_rule(&module, Rule::module)?;

    let inner: Pairs<'i, Rule> = module.into_inner();
    let mut package = None;
    let mut imports = vec![];
    let mut members: HashMap<Name, Box<dyn Member + 'i>> = HashMap::new();
    for pair in inner {
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
                let func = SourceFunction::try_from(pair)?;
                members.insert(func.name, Box::new(func));
            }
            Rule::EOI => {}
            r => return Err(AstError::InvalidRule("module contents", r)),
        }
    }
    match package {
        None => Err(AstError::InvalidState("Module must have package set")),
        Some(package) => Ok(Module {
            package,
            imports,
            members,
        }),
    }
}
