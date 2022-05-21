use lazy_static::lazy_static;
use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::Mutex;

#[derive(Copy, Clone, Debug)]
pub struct InternedString(&'static String);

lazy_static! {
    static ref STRINGS: Mutex<BTreeSet<&'static String>> = Mutex::new(BTreeSet::new());
}

impl Display for InternedString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Deref for InternedString {
    type Target = str;
    fn deref(&self) -> &str {
        self.0
    }
}

impl PartialEq for InternedString {
    fn eq(&self, other: &Self) -> bool {
        let ptr_self: *const String = self.0;
        let ptr_other: *const String = other.0;
        ptr_self == ptr_other
    }
}

impl Eq for InternedString {}

impl Hash for InternedString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl From<&str> for InternedString {
    fn from(name: &str) -> Self {
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
        InternedString(value)
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::{assert_eq, assert_ne};

    use super::InternedString;

    #[test]
    fn interns_strings() {
        let one: InternedString = "foo".into();
        let two: InternedString = "foo".into();
        let ptr_one: *const String = one.0;
        let ptr_two: *const String = two.0;
        assert_eq!(ptr_one, ptr_two);
        assert_eq!(one, two);
    }

    #[test]
    fn mismatched_strings() {
        let one: InternedString = "foo".into();
        let two: InternedString = "bar".into();
        let ptr_one: *const String = one.0;
        let ptr_two: *const String = two.0;
        assert_ne!(ptr_one, ptr_two);
        assert_ne!(one, two);
    }
}
