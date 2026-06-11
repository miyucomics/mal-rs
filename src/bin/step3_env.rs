#![warn(clippy::pedantic)]

use std::{collections::BTreeMap, rc::Rc};

use mal::{
    env::{Env, EnvRef},
    printer::print_str,
    reader::{ReadError, read_str},
    readline::readline,
    types::Atom,
};

fn read(input: &str) -> Result<Atom, ReadError> {
    read_str(input)
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
                    "def!" => {
                        let key = match atoms.get(1) {
                            Some(Atom::Symbol(s)) => s.clone(),
                            _ => return Err("def! requires a symbol as first argument".to_string()),
                        };
                        let value = eval(
                            atoms
                                .get(2)
                                .ok_or("def! requires a value as second argument")?
                                .clone(),
                            env,
                        )?;
                        env.borrow_mut().set(&key, value.clone());
                        return Ok(value);
                    }
                    "let*" => {
                        let bindings = match atoms.get(1) {
                            Some(Atom::List(b) | Atom::Vector(b)) => b.clone(),
                            _ => return Err("let* needs a list of bindings".to_string()),
                        };

                        let to_execute = atoms
                            .get(2)
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
                                    return Err(
                                        "let* binding list must have symbol/value pairings"
                                            .to_string(),
                                    );
                                }
                            }
                        }

                        return eval(to_execute, &let_env);
                    }
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
    let repl_env: EnvRef = Env::new(None);

    {
        let mut env = repl_env.borrow_mut();

        env.set(
            "+",
            Atom::Function(|args| match args {
                [Atom::Int(a), Atom::Int(b)] => Ok(Atom::Int(a + b)),
                _ => Err("improper arguments for '+'".to_string()),
            }),
        );

        env.set(
            "-",
            Atom::Function(|args| match args {
                [Atom::Int(a), Atom::Int(b)] => Ok(Atom::Int(a - b)),
                _ => Err("improper arguments for '-'".to_string()),
            }),
        );

        env.set(
            "*",
            Atom::Function(|args| match args {
                [Atom::Int(a), Atom::Int(b)] => Ok(Atom::Int(a * b)),
                _ => Err("improper arguments for '*'".to_string()),
            }),
        );

        env.set(
            "/",
            Atom::Function(|args| match args {
                [Atom::Int(a), Atom::Int(b)] => Ok(Atom::Int(a / b)),
                _ => Err("improper arguments for '/'".to_string()),
            }),
        );
    }

    while let Some(ref line) = readline("user> ") {
        if !line.is_empty() {
            match rep(line, &repl_env) {
                Ok(response) => println!("{response}"),
                Err(err) => println!("error: {err}"),
            }
        }
    }
}
