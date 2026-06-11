#![warn(clippy::pedantic)]

use std::{collections::BTreeMap, rc::Rc};

use mal::{
    core::construct_repl_env,
    env::{Env, EnvRef},
    printer::print_str,
    reader::{ReadError, read_str},
    readline::readline,
    types::Atom,
};

fn read(input: &str) -> Result<Atom, ReadError> {
    read_str(input)
}

fn special_def(atoms: &[Atom], env: &EnvRef) -> Result<Atom, String> {
    let key = match atoms.first() {
        Some(Atom::Symbol(s)) => s.clone(),
        _ => return Err("def! requires a symbol as first argument".to_string()),
    };

    let value = eval(
        atoms
            .get(1)
            .ok_or("def! requires a value as second argument")?
            .clone(),
        env,
    )?;

    env.borrow_mut().set(&key, value.clone());
    Ok(value)
}

fn special_let(atoms: &[Atom], env: &EnvRef) -> Result<Atom, String> {
    let bindings = match atoms.first() {
        Some(Atom::List(b) | Atom::Vector(b)) => b.clone(),
        _ => return Err("let* needs a list of bindings".to_string()),
    };

    let to_execute = atoms
        .get(1)
        .ok_or("let* needs a function to execute")?
        .clone();

    let let_env = Env::new(Some(env.clone()));
    let mut iter = bindings.iter();
    loop {
        match (iter.next(), iter.next()) {
            (Some(Atom::Symbol(key)), Some(other)) => {
                let value = eval(other.clone(), &let_env)?;
                let_env.borrow_mut().set(key, value);
            }
            (None, _) => break,
            _ => {
                return Err("let* binding list must have symbol/value pairings".to_string());
            }
        }
    }

    eval(to_execute, &let_env)
}

fn special_do(atoms: &[Atom], env: &EnvRef) -> Result<Atom, String> {
    if atoms.is_empty() {
        return Err("do needs at least one element".to_string());
    }
    let last = atoms.len() - 1;
    for atom in &atoms[..last] {
        eval(atom.clone(), env)?;
    }
    eval(atoms[last].clone(), env)
}

fn special_if(atoms: &[Atom], env: &EnvRef) -> Result<Atom, String> {
    let condition = eval(
        atoms
            .first()
            .cloned()
            .ok_or("if needs a condition as first argument")?,
        env,
    )?;

    match condition {
        Atom::Nil | Atom::Bool(false) => {
            if let Some(false_branch) = atoms.get(2) {
                eval(false_branch.clone(), env)
            } else {
                Ok(Atom::Nil)
            }
        }
        _ => eval(atoms.get(1).ok_or("if needs a true branch")?.clone(), env),
    }
}

fn special_fn(atoms: &[Atom], env: &EnvRef) -> Result<Atom, String> {
    if let Some(Atom::List(raw_params) | Atom::Vector(raw_params)) = atoms.first() {
        let params: Rc<[Rc<str>]> = raw_params
            .iter()
            .map(|a| match a {
                Atom::Symbol(s) => Ok(Rc::clone(s)),
                _ => Err("fn* parameters must be symbols".to_string()),
            })
            .collect::<Result<Vec<_>, _>>()?
            .into();

        let body = atoms.get(1).ok_or("fn* requires a body")?.clone();

        Ok(Atom::Lambda {
            params,
            body: Box::new(body),
            env: Rc::clone(env),
        })
    } else {
        Err("fn* requires a parameter list".to_string())
    }
}

fn eval(input: Atom, env: &EnvRef) -> Result<Atom, String> {
    if let Some(debug_val) = Env::get(env, "DEBUG-EVAL") {
        match debug_val {
            Atom::Nil | Atom::Bool(false) => {}
            _ => println!("EVAL: {}", print(&input)),
        }
    }

    match input {
        Atom::Symbol(symbol) => {
            Env::get(env, &symbol).ok_or_else(|| format!("'{symbol}' not found in environment"))
        }
        Atom::List(atoms) => {
            if atoms.is_empty() {
                return Ok(Atom::List(atoms));
            }

            if let Some(Atom::Symbol(symbol)) = atoms.first() {
                match symbol.as_ref() {
                    "def!" => return special_def(&atoms[1..], env),
                    "let*" => return special_let(&atoms[1..], env),
                    "do" => return special_do(&atoms[1..], env),
                    "if" => return special_if(&atoms[1..], env),
                    "fn*" => return special_fn(&atoms[1..], env),
                    _ => {}
                }
            }

            let mut evaluated_atoms = Vec::new();
            for atom in atoms.iter() {
                evaluated_atoms.push(eval(atom.clone(), env)?);
            }

            let mut iter = evaluated_atoms.into_iter();
            let first = iter.next().unwrap();
            let remaining: Vec<Atom> = iter.collect();

            match first {
                Atom::Function(func) => func(&remaining),
                Atom::Lambda {
                    params,
                    body,
                    env: closed_env,
                } => {
                    let fn_env =
                        Env::new_with_binds(Some(Rc::clone(&closed_env)), &params, &remaining);
                    eval(*body.clone(), &fn_env)
                }
                _ => Err("first element in a list must be a function".to_string()),
            }
        }
        Atom::Vector(atoms) => {
            let mut evaluated_atoms = Vec::new();
            for atom in atoms.iter() {
                evaluated_atoms.push(eval(atom.clone(), env)?);
            }
            Ok(Atom::Vector(Rc::from(evaluated_atoms)))
        }
        Atom::Map(atoms) => {
            let mut evaluated_atoms = BTreeMap::new();
            for (key, value) in atoms.iter() {
                evaluated_atoms.insert(key.clone(), eval(value.clone(), env)?);
            }
            Ok(Atom::Map(Rc::from(evaluated_atoms)))
        }
        _ => Ok(input),
    }
}

fn print(input: &Atom) -> String {
    print_str(input, true)
}

fn rep(input: &str, env: &EnvRef) -> Result<String, String> {
    let parsed = read(input).map_err(|e| format!("{e:?}"))?;
    let evaluated = eval(parsed, env)?;
    Ok(print(&evaluated))
}

fn main() {
    let repl_env = construct_repl_env();
    let _ = rep("(def! not (fn* (a) (if a false true)))", &repl_env);

    while let Some(ref line) = readline("user> ") {
        if !line.is_empty() {
            match rep(line, &repl_env) {
                Ok(response) => println!("{response}"),
                Err(err) => println!("error: {err}"),
            }
        }
    }
}
