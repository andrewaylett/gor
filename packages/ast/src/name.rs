use crate::{AstResult, Parseable};
use gor_core::interned_string::InternedString;
use gor_parse::Rule;
use pest::iterators::{Pair, Pairs};
use pest::Span;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::ops::Deref;
use std::path::Path;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Name(InternedString);

macro_rules! and_then {
    ($start:tt $(, $proc:expr)*) => {
        Some($start)$(.and_then(|$start| $proc))*
    };
}

impl Name {
    pub fn from_quoted(name: &str) -> Option<Name> {
        and_then!(
            name,
            name.strip_suffix('"'),
            name.strip_prefix('"'),
            Some(Name::from(name))
        )
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Debug for Name {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
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

impl Parseable<'_> for Name {
    const RULE: Rule = Rule::name;

    fn build(span: &Span<'_>, _pairs: Pairs<'_, Rule>) -> AstResult<Self> {
        Ok(span.as_str().into())
    }

    fn descend(pair: Pair<'_, Rule>) -> AstResult<Self> {
        Ok(pair.as_str().into())
    }
}

impl AsRef<Path> for Name {
    fn as_ref(&self) -> &Path {
        self.deref().as_ref()
    }
}
