use crate::name::Name;
use crate::{expect_rule, AstError, AstResult};
use gor_parse::Rule;
use pest::iterators::{Pair, Pairs};
use std::collections::HashMap;
use std::fmt::Debug;

trait Member: Debug {}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Module {
    package: Name,
    imports: Vec<Name>,
    members: HashMap<Name, Box<dyn Member>>,
}

impl<'i> TryFrom<Pairs<'i, Rule>> for Module {
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

impl TryFrom<Pair<'_, Rule>> for Module {
    type Error = AstError;

    fn try_from(module: Pair<Rule>) -> AstResult<Self> {
        expect_rule(&module, Rule::module)?;
        primary(module)
    }
}

fn primary(module: Pair<Rule>) -> AstResult<Module> {
    expect_rule(&module, Rule::module)?;

    let inner = module.into_inner();
    let mut package = None;
    let mut imports = vec![];
    let members = HashMap::new();
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
            Rule::statement => {}
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
