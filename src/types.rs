#![warn(clippy::pedantic)]

use std::{collections::BTreeMap, rc::Rc};

use crate::env::EnvRef;

pub type MalFn = fn(&[Atom]) -> Result<Atom, String>;

#[derive(Clone)]
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
    Lambda {
        params: Rc<[Rc<str>]>,
        body: Box<Atom>,
        env: EnvRef,
    },
}

// so many custom implementations below because of the lambda type, and more specifically
// because EnvRef does not implement any of this stuff so I can not just derive the traits

impl Atom {
    fn sort_key(&self) -> u8 {
        match self {
            Atom::Nil => 0,
            Atom::Bool(_) => 1,
            Atom::Int(_) => 2,
            Atom::Keyword(_) => 3,
            Atom::Str(_) => 4,
            Atom::Symbol(_) => 5,
            Atom::List(_) => 6,
            Atom::Vector(_) => 7,
            Atom::Map(_) => 8,
            Atom::Function(_) => 9,
            Atom::Lambda { .. } => 10,
        }
    }
}

impl PartialEq for Atom {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Atom::Nil, Atom::Nil) => true,
            (Atom::Bool(a), Atom::Bool(b)) => a == b,
            (Atom::Int(a), Atom::Int(b)) => a == b,
            (Atom::Keyword(a), Atom::Keyword(b))
            | (Atom::Str(a), Atom::Str(b))
            | (Atom::Symbol(a), Atom::Symbol(b)) => a == b,
            (Atom::List(a) | Atom::Vector(a), Atom::List(b) | Atom::Vector(b)) => a == b,
            (Atom::Map(a), Atom::Map(b)) => a == b,

            // functions and lambdas are never equal
            _ => false,
        }
    }
}

impl Eq for Atom {}

impl PartialOrd for Atom {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Atom {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.sort_key().cmp(&other.sort_key())
    }
}
