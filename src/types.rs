#![warn(clippy::pedantic)]
#![allow(unpredictable_function_pointer_comparisons)]

use std::{collections::BTreeMap, rc::Rc};

pub type MalFn = fn(&[Atom]) -> Result<Atom, String>;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Atom {
    Nil,
    Bool(bool),
    Int(i32),
    Keyword(Rc<str>),
    Str(Rc<str>),
    Symbol(Rc<str>),
    List(Rc<[Atom]>),
    Vector(Rc<[Atom]>),
    Map(Rc<BTreeMap<String, Atom>>),
    Function(MalFn),
}
