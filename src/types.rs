#![warn(clippy::pedantic)]
#![allow(unpredictable_function_pointer_comparisons)]

use std::collections::BTreeMap;

pub type MalFn = fn(&[Atom]) -> Result<Atom, String>;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Atom {
    Nil,
    Bool(bool),
    Int(i32),
    Keyword(String),
    Str(String),
    Symbol(String),
    List(Vec<Atom>),
    Vector(Vec<Atom>),
    Map(BTreeMap<Atom, Atom>),
    Function(MalFn),
}
