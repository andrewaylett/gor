use crate::{expect_rule, AstError, AstResult};
use gor_core::interned_string::InternedString;
use gor_parse::Rule;
use pest::iterators::Pair;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::ops::Deref;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Name(InternedString);

impl Display for Name {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl Deref for Name {
    type Target = InternedString;
    fn deref(&self) -> &InternedString {
        &self.0
    }
}

impl From<&str> for Name {
    fn from(name: &str) -> Self {
        Name(InternedString::from(name))
    }
}

impl TryFrom<Pair<'_, Rule>> for Name {
    type Error = AstError;

    fn try_from(pair: Pair<Rule>) -> AstResult<Self> {
        expect_rule(&pair, Rule::name)?;
        Ok(pair.as_str().into())
    }
}
