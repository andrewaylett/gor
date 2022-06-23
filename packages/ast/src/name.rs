use crate::{expect_rule, AstError, AstResult};
use gor_core::interned_string::InternedString;
use gor_parse::Rule;
use pest::iterators::Pair;
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

impl TryFrom<Pair<'_, Rule>> for Name {
    type Error = AstError;

    fn try_from(pair: Pair<Rule>) -> AstResult<Self> {
        expect_rule(&pair, Rule::name)?;
        Ok(pair.as_str().into())
    }
}

impl AsRef<Path> for Name {
    fn as_ref(&self) -> &Path {
        self.deref().as_ref()
    }
}
