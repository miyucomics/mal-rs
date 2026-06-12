#![warn(clippy::pedantic)]
#![allow(clippy::too_many_lines)]

use std::{fs::File, io::Read, rc::Rc};

use crate::{
    env::{Env, EnvRef},
    printer::print_str,
    reader::read_str,
    types::Atom,
};

macro_rules! two_int_op {
    ($op:literal, $f:expr) => {
        Atom::Function(Rc::new(|args| match args {
            [Atom::Int(a), Atom::Int(b)] => Ok($f(*a, *b)),
            _ => Err(format!("improper arguments for '{}'", $op)),
        }))
    };
}

macro_rules! is_type_op {
    ($($ps:pat),*) => {{
        |atoms: &[Atom]| { Ok(Atom::Bool(match atoms[0] { $($ps => true,)* _ => false})) }
    }};
}

fn func(f: fn(&[Atom]) -> Result<Atom, String>) -> Atom {
    Atom::Function(Rc::new(f))
}

fn standard_library() -> Vec<(&'static str, Atom)> {
    let mut lib = Vec::new();
    lib.push(("+", two_int_op!("+", |a, b| Atom::Int(a + b))));
    lib.push(("-", two_int_op!("-", |a, b| Atom::Int(a - b))));
    lib.push(("*", two_int_op!("*", |a, b| Atom::Int(a * b))));
    lib.push(("/", two_int_op!("/", |a, b| Atom::Int(a / b))));

    lib.push(("list", func(|atoms| Ok(Atom::List(Rc::from(atoms))))));
    lib.push(("list?", func(is_type_op!(Atom::List(_)))));
    lib.push((
        "empty?",
        func(|atoms| match atoms.first().ok_or("empty? needs a list")? {
            Atom::List(list) | Atom::Vector(list) => Ok(Atom::Bool(list.is_empty())),
            _ => Err("empty? needs a list".to_string()),
        }),
    ));
    lib.push((
        "count",
        func(
            |atoms| match atoms.first().ok_or("count needs an argument")? {
                Atom::List(list) | Atom::Vector(list) => Ok(Atom::Int(
                    i32::try_from(list.len()).expect("List overflow?"),
                )),
                _ => Ok(Atom::Int(0)), // for some reason, MAL wants us to return 0 even if it's nil
            },
        ),
    ));
    lib.push((
        "=",
        func(|atoms| {
            let first = atoms.first().ok_or("= needs two atoms")?;
            let second = atoms.get(1).ok_or("= needs two atoms")?;
            Ok(Atom::Bool(first == second))
        }),
    ));
    lib.push(("<", two_int_op!("<", |a, b| Atom::Bool(a < b))));
    lib.push(("<=", two_int_op!("<=", |a, b| Atom::Bool(a <= b))));
    lib.push((">", two_int_op!(">", |a, b| Atom::Bool(a > b))));
    lib.push((">=", two_int_op!(">=", |a, b| Atom::Bool(a >= b))));
    lib.push((
        "pr-str",
        func(|atoms| {
            Ok(Atom::Str(Rc::from(
                atoms
                    .iter()
                    .map(|x| print_str(x, true))
                    .collect::<Vec<String>>()
                    .join(" "),
            )))
        }),
    ));
    lib.push((
        "str",
        func(|atoms| {
            Ok(Atom::Str(Rc::from(
                atoms
                    .iter()
                    .map(|x| print_str(x, false))
                    .collect::<String>(),
            )))
        }),
    ));
    lib.push((
        "prn",
        func(|atoms| {
            let output = atoms
                .iter()
                .map(|x| print_str(x, true))
                .collect::<Vec<String>>()
                .join(" ");
            println!("{output}");
            Ok(Atom::Nil)
        }),
    ));
    lib.push((
        "println",
        func(|atoms| {
            let output = atoms
                .iter()
                .map(|x| print_str(x, false))
                .collect::<Vec<String>>()
                .join(" ");
            println!("{output}");
            Ok(Atom::Nil)
        }),
    ));

    lib.push((
        "read-string",
        func(|atoms| {
            if let Some(Atom::Str(str)) = atoms.first() {
                return match read_str(str) {
                    Ok(a) => Ok(a),
                    Err(error) => Ok(Atom::Str(Rc::from(error.to_string()))),
                };
            }

            Err("read-string needs a string".to_string())
        }),
    ));
    lib.push((
        "slurp",
        func(|atoms| {
            if let Some(Atom::Str(path)) = atoms.first() {
                let mut contents = String::new();
                return match File::open(path.to_string())
                    .and_then(|mut f| f.read_to_string(&mut contents))
                {
                    Ok(_) => Ok(Atom::Str(Rc::from(contents))),
                    Err(e) => Err(e.to_string()),
                };
            }

            Err("slurp needs a string".to_string())
        }),
    ));

    lib
}

#[must_use]
pub fn construct_repl_env() -> EnvRef {
    let repl_env: EnvRef = Env::new(None);
    {
        let mut env = repl_env.borrow_mut();
        for (key, value) in standard_library() {
            env.set(key, value);
        }
    }
    repl_env
}
