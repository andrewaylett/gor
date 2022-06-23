#![deny(
    bad_style,
    const_err,
    dead_code,
    improper_ctypes,
    missing_debug_implementations,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    unconditional_recursion,
    unreachable_pub,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
    clippy::expect_used
)]
#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

use crate::interned_string::InternedString;
use std::error::Error;
use std::fmt::Debug;

pub mod interned_string;
pub mod parse_error;

pub type CoreError = Box<dyn Error + 'static>;
pub type CoreResult<T> = Result<T, CoreError>;

pub trait Visited<'i, T: ?Sized, R> {
    fn visit(&'i self, by: &'i T) -> R;
}

pub trait Visitor<'i, T: Visited<'i, Self, R> + ?Sized, R> {
    fn visit(&'i self, to: &'i T) -> R {
        to.visit(self)
    }
}

pub trait Member: Debug + Sync {}

pub trait Function<'i>: Member {}

pub trait Module<'i> {
    fn function(&self, name: &str) -> Option<&dyn Function<'i>>;
}

pub trait ModuleLoader<'i, T: Module<'i>> {
    fn load(&self, name: InternedString) -> &'i T;
}
