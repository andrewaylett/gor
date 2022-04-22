use crate::{expect_rule, AstError, AstResult};
use gor_parse::Rule;
use lazy_static::lazy_static;
use pest::iterators::Pair;
use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::Mutex;

#[derive(Copy, Clone, Debug)]
pub struct Name(&'static String);

lazy_static! {
    static ref STRINGS: Mutex<BTreeSet<&'static String>> = Mutex::new(BTreeSet::new());
}

impl Display for Name {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl Deref for Name {
    type Target = str;
    fn deref(&self) -> &str {
        self.0
    }
}

impl PartialEq for Name {
    fn eq(&self, other: &Self) -> bool {
        let ptr_self: *const String = self.0;
        let ptr_other: *const String = other.0;
        ptr_self == ptr_other
    }
}

impl Eq for Name {}

impl Hash for Name {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl From<&'_ str> for Name {
    fn from(name: &'_ str) -> Self {
        let mut lock = STRINGS.lock().unwrap();
        let key = name.to_string();
        let intern = lock.get(&key);
        let value = if let Some(&s) = intern {
            s
        } else {
            let boxed = Box::new(key);
            let leaked: &'static String = Box::leak(boxed);
            lock.insert(leaked);
            leaked
        };
        Name(value)
    }
}

impl TryFrom<Pair<'_, Rule>> for Name {
    type Error = AstError;

    fn try_from(pair: Pair<Rule>) -> AstResult<Self> {
        expect_rule(&pair, Rule::name)?;
        Ok(pair.as_str().into())
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::{assert_eq, assert_ne};

    use super::Name;

    #[test]
    fn interns_strings() {
        let one: Name = "foo".into();
        let two: Name = "foo".into();
        let ptr_one: *const String = one.0;
        let ptr_two: *const String = two.0;
        assert_eq!(ptr_one, ptr_two);
        assert_eq!(one, two);
    }

    #[test]
    fn mismatched_strings() {
        let one: Name = "foo".into();
        let two: Name = "bar".into();
        let ptr_one: *const String = one.0;
        let ptr_two: *const String = two.0;
        assert_ne!(ptr_one, ptr_two);
        assert_ne!(one, two);
    }
}
