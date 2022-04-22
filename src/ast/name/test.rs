use pretty_assertions::{assert_eq, assert_ne};

use crate::ast::name::Name;

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
