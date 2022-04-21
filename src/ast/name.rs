use lazy_static::lazy_static;
use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::sync::Mutex;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
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

impl From<&'_ str> for Name {
    fn from(name: &'_ str) -> Self {
        let mut lock = STRINGS.lock().unwrap();
        let key = name.to_string();
        let intern = lock.get(&key);
        let value = if let Some(&s) = intern {
            s
        } else {
            let boxed = Box::new(key);
            Box::leak(boxed)
        };
        lock.insert(value);
        Name(value)
    }
}

#[cfg(test)]
mod test;
