#![warn(clippy::pedantic)]

use std::{collections::BTreeMap, rc::Rc};

use mal::{
    core::construct_repl_env,
    env::{Env, EnvRef},
    printer::print_str,
    reader::read_str,
    readline::readline,
    types::Atom,
};

enum Step {
    Done(Atom),
    Thunk(Atom, EnvRef),
}

fn trampoline(mut step: Result<Step, String>) -> Result<Atom, String> {
    loop {
        match step? {
            Step::Done(value) => return Ok(value),
            Step::Thunk(input, env) => step = eval_step(input, &env),
        }
    }
}

fn eval_step(input: Atom, env: &EnvRef) -> Result<Step, String> {
    if let Some(debug_val) = Env::get(env, "DEBUG-EVAL") {
        match debug_val {
            Atom::Nil | Atom::Bool(false) => {}
            _ => println!("EVAL: {}", print(&input)),
        }
    }

    match input {
        Atom::Symbol(ref sym) => Env::get(env, sym)
            .map(Step::Done)
            .ok_or(format!("'{sym}' not found in environment")),
        Atom::List(ref atoms) if atoms.is_empty() => Ok(Step::Done(input)),
        Atom::List(ref atoms) => {
            if let Atom::Symbol(sym) = &atoms[0] {
                match sym.as_ref() {
                    "def!" => return special_def(&atoms[1..], env),
                    "let*" => return special_let(&atoms[1..], env),
                    "do" => return special_do(&atoms[1..], env),
                    "if" => return special_if(&atoms[1..], env),
                    "fn*" => return special_fn(&atoms[1..], env),
                    _ => {}
                }
            }

            let head = trampoline(eval_step(atoms[0].clone(), env))?;
            let args = atoms[1..]
                .iter()
                .map(|x| trampoline(eval_step(x.clone(), env)))
                .collect::<Result<Vec<_>, _>>()?;

            match head {
                Atom::Function(func) => Ok(Step::Done(func(&args)?)),
                Atom::Lambda {
                    params,
                    body,
                    env: closed_env,
                } => Ok(Step::Thunk(
                    *body,
                    Env::new_with_binds(Some(Rc::clone(&closed_env)), &params, &args),
                )),
                _ => Err("first element in a list must be a function".to_string()),
            }
        }
        Atom::Vector(ref atoms) => {
            let evaled = atoms
                .iter()
                .map(|x| trampoline(eval_step(x.clone(), env)))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Step::Done(Atom::Vector(Rc::from(evaled))))
        }
        Atom::Map(ref pairs) => {
            let evaled = pairs
                .iter()
                .map(|(k, v)| trampoline(eval_step(v.clone(), env)).map(|val| (k.clone(), val)))
                .collect::<Result<BTreeMap<_, _>, _>>()?;
            Ok(Step::Done(Atom::Map(Rc::from(evaled))))
        }
        _ => Ok(Step::Done(input)),
    }
}

fn special_def(atoms: &[Atom], env: &EnvRef) -> Result<Step, String> {
    let key = match atoms.first() {
        Some(Atom::Symbol(s)) => s.clone(),
        _ => return Err("def! requires a symbol as first argument".to_string()),
    };

    let value = trampoline(eval_step(
        atoms
            .get(1)
            .ok_or("def! requires a value as second argument")?
            .clone(),
        env,
    ))?;

    env.borrow_mut().set(&key, value.clone());
    Ok(Step::Done(value))
}

fn special_let(atoms: &[Atom], env: &EnvRef) -> Result<Step, String> {
    let bindings = match atoms.first() {
        Some(Atom::List(b) | Atom::Vector(b)) => b.clone(),
        _ => return Err("let* needs a list of bindings".to_string()),
    };

    let body = atoms
        .get(1)
        .ok_or("let* needs a function to execute")?
        .clone();

    let let_env = Env::new(Some(env.clone()));
    let mut iter = bindings.iter();

    loop {
        match (iter.next(), iter.next()) {
            (Some(Atom::Symbol(key)), Some(other)) => {
                let value = trampoline(eval_step(other.clone(), &let_env))?;
                let_env.borrow_mut().set(key, value);
            }
            (None, _) => break,
            _ => {
                return Err("let* binding list must have symbol/value pairings".to_string());
            }
        }
    }

    Ok(Step::Thunk(body, let_env))
}

fn special_do(atoms: &[Atom], env: &EnvRef) -> Result<Step, String> {
    let (last, rest) = atoms
        .split_last()
        .ok_or("do needs at least one expression")?;

    for atom in rest {
        trampoline(eval_step(atom.clone(), env))?;
    }

    Ok(Step::Thunk(last.clone(), env.clone()))
}

fn special_if(atoms: &[Atom], env: &EnvRef) -> Result<Step, String> {
    let condition = trampoline(eval_step(
        atoms
            .first()
            .cloned()
            .ok_or("if needs a condition as first argument")?,
        env,
    ))?;

    let branch = match condition {
        Atom::Nil | Atom::Bool(false) => atoms.get(2).cloned().unwrap_or(Atom::Nil),
        _ => atoms.get(1).ok_or("if needs a true branch")?.clone(),
    };

    Ok(Step::Thunk(branch, env.clone()))
}

fn special_fn(atoms: &[Atom], env: &EnvRef) -> Result<Step, String> {
    let Some(Atom::List(raw_params) | Atom::Vector(raw_params)) = atoms.first() else {
        return Err("fn* requires a parameter list".to_string());
    };

    let params: Rc<[Rc<str>]> = raw_params
        .iter()
        .map(|x| match x {
            Atom::Symbol(s) => Ok(Rc::clone(s)),
            _ => Err("fn* parameters must be symbols".to_string()),
        })
        .collect::<Result<Vec<_>, _>>()?
        .into();

    let body = atoms.get(1).ok_or("fn* requires a body")?.clone();

    Ok(Step::Done(Atom::Lambda {
        params,
        body: Box::new(body),
        env: Rc::clone(env),
    }))
}

fn print(input: &Atom) -> String {
    print_str(input, true)
}

fn rep(input: &str, env: &EnvRef) -> Result<String, String> {
    let parsed = read_str(input).map_err(|e| format!("{e:?}"))?;
    let evaluated = trampoline(eval_step(parsed, env))?;
    Ok(print(&evaluated))
}

fn main() {
    let repl_env = construct_repl_env();
    let _ = rep("(def! not (fn* (a) (if a false true)))", &repl_env);
    let _ = rep(
        r#"(def! load-file (fn* (f) (eval (read-string (str "(do " (slurp f) "\nnil)")))))"#,
        &repl_env,
    );

    let env_reference = repl_env.clone();
    repl_env.borrow_mut().set(
        "eval",
        Atom::Function(Rc::new(move |atoms| {
            let code = atoms.first().ok_or("eval needs something to eval")?.clone();
            trampoline(eval_step(code, &env_reference))
        })),
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
