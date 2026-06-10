#![warn(clippy::pedantic)]

use std::collections::BTreeMap;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Atom {
    Nil,
    Bool(bool),
    Integer(i64),
    Keyword(String),
    String(String),
    Symbol(String),
    List(Vec<Atom>),
    Vector(Vec<Atom>),
    Map(BTreeMap<Atom, Atom>),
}
