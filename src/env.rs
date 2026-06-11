use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::types::Atom;

pub type EnvRef = Rc<RefCell<Env>>;

pub struct Env {
    outer: Option<EnvRef>,
    data: HashMap<String, Atom>,
}

impl Env {
    pub fn new(outer: Option<EnvRef>) -> EnvRef {
        Rc::new(RefCell::new(Env {
            outer,
            data: HashMap::new(),
        }))
    }

    pub fn new_with_binds(outer: Option<EnvRef>, binds: &[Rc<str>], exprs: &[Atom]) -> EnvRef {
        let env = Rc::new(RefCell::new(Env {
            outer,
            data: HashMap::new(),
        }));

        for (i, bind) in binds.iter().enumerate() {
            if bind.as_ref() == "&" {
                let rest_name = &binds[i + 1];
                let rest = Atom::List(Rc::from(&exprs[i..]));
                env.borrow_mut().set(rest_name, rest);
                break;
            }
            env.borrow_mut().set(bind, exprs[i].clone());
        }

        env
    }

    pub fn get(env: &EnvRef, key: &str) -> Option<Atom> {
        let mut current = Rc::clone(env);

        loop {
            let borrowed = current.borrow();
            if let Some(atom) = borrowed.data.get(key) {
                return Some(atom.clone());
            }

            match &borrowed.outer {
                Some(outer) => {
                    let next = Rc::clone(outer);
                    drop(borrowed);
                    current = next;
                }
                None => return None,
            }
        }
    }

    pub fn set(&mut self, symbol: &str, atom: Atom) {
        self.data.insert(symbol.to_owned(), atom);
    }
}
