#![warn(clippy::pedantic)]

use std::{
    collections::{BTreeMap, HashMap},
    rc::Rc,
};

use mal::{
    printer::print_str,
    reader::{ReadError, read_str},
    readline::readline,
    types::Atom,
};

fn read(input: &str) -> Result<Atom, ReadError> {
    read_str(input)
}

fn eval(input: Atom, env: &HashMap<Rc<str>, Atom>) -> Result<Atom, String> {
    if let Some(debug_val) = env.get("DEBUG-EVAL") {
        match debug_val {
            Atom::Nil | Atom::Bool(false) => {}
            _ => println!("EVAL: {}", print(&input)),
        }
    }

    match input {
        Atom::Symbol(symbol) => env
            .get(&symbol)
            .cloned()
            .ok_or_else(|| format!("'{symbol}' not found in environment")),
        Atom::List(atoms) => {
            if atoms.is_empty() {
                return Ok(Atom::List(atoms));
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

fn rep(input: &str, env: &HashMap<Rc<str>, Atom>) -> Result<String, String> {
    let parsed = read(input).map_err(|e| format!("{e:?}"))?;
    let evaluated = eval(parsed, env)?;
    Ok(print(&evaluated))
}

fn main() {
    let mut repl_env: HashMap<Rc<str>, Atom> = HashMap::new();

    repl_env.insert(
        Rc::from("+"),
        Atom::Function(|args| match args {
            [Atom::Int(a), Atom::Int(b)] => Ok(Atom::Int(a + b)),
            _ => Err("improper arguments for '+'".to_string()),
        }),
    );

    repl_env.insert(
        Rc::from("-"),
        Atom::Function(|args| match args {
            [Atom::Int(a), Atom::Int(b)] => Ok(Atom::Int(a - b)),
            _ => Err("improper arguments for '-'".to_string()),
        }),
    );

    repl_env.insert(
        Rc::from("*"),
        Atom::Function(|args| match args {
            [Atom::Int(a), Atom::Int(b)] => Ok(Atom::Int(a * b)),
            _ => Err("improper arguments for '*'".to_string()),
        }),
    );

    repl_env.insert(
        Rc::from("/"),
        Atom::Function(|args| match args {
            [Atom::Int(a), Atom::Int(b)] => Ok(Atom::Int(a / b)),
            _ => Err("improper arguments for '/'".to_string()),
        }),
    );

    while let Some(ref line) = readline("user> ") {
        if !line.is_empty() {
            match rep(line, &repl_env) {
                Ok(response) => println!("{response}"),
                Err(err) => println!("error: {err}"),
            }
        }
    }
}
