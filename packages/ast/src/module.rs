use crate::func::SourceFunction;
use crate::name::Name;
use crate::{expect_rule, AstError, AstResult};
use gor_parse::Rule;
use pest::iterators::{Pair, Pairs};
use std::collections::HashMap;
use std::fmt::Debug;

#[allow(dead_code)]
#[derive(Debug)]
pub struct SourceModule<'i> {
    pub package: Name,
    pub imports: Vec<Name>,
    functions: HashMap<Name, Box<SourceFunction<'i>>>,
}

impl<'s: 'i, 'i> TryFrom<Pairs<'s, Rule>> for SourceModule<'i> {
    type Error = AstError;

    fn try_from(mut pairs: Pairs<'s, Rule>) -> AstResult<Self> {
        let pair = pairs.next().ok_or(AstError::InvalidState(
            "Expected to get a module, but found nothing to parse",
        ))?;
        let item: Result<SourceModule<'i>, AstError> = SourceModule::try_from(pair);
        if pairs.next().is_some() {
            Err(AstError::InvalidState(
                "Expected to consume all of the parse",
            ))
        } else {
            item
        }
    }
}

impl<'s: 'i, 'i> TryFrom<Pair<'s, Rule>> for SourceModule<'i> {
    type Error = AstError;

    fn try_from(module: Pair<'s, Rule>) -> AstResult<Self> {
        expect_rule(&module, Rule::module)?;
        primary(module)
    }
}

impl<'i> SourceModule<'i> {
    pub fn function(&self, name: Name) -> Option<&SourceFunction<'i>> {
        self.functions.get(&name).map(|b| b.as_ref())
    }
}

fn primary<'s: 'i, 'i>(module: Pair<'s, Rule>) -> AstResult<SourceModule<'i>> {
    expect_rule(&module, Rule::module)?;

    let inner: Pairs<'s, Rule> = module.into_inner();
    let mut package = None;
    let mut imports = vec![];
    let mut functions: HashMap<Name, Box<SourceFunction<'i>>> = HashMap::new();
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
                functions.insert(func.name, Box::new(func));
            }
            Rule::EOI => {}
            r => return Err(AstError::InvalidRule("module contents", r)),
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
