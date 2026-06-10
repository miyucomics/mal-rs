#![warn(clippy::pedantic)]

use crate::types::Atom;

#[must_use]
pub fn print_str(atom: &Atom, print_readably: bool) -> String {
    match atom {
        Atom::Nil => "nil".to_string(),
        Atom::Bool(value) => {
            if *value {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        Atom::Int(value) => value.to_string(),
        Atom::Keyword(name) => format!(":{name}"),
        Atom::Str(string) => {
            let string = if print_readably {
                string
                    .replace('\\', "\\\\")
                    .replace('\n', "\\n")
                    .replace('"', "\\\"")
            } else {
                string.clone()
            };
            format!("\"{string}\"")
        }
        Atom::Symbol(value) => value.clone(),
        Atom::List(contents) => {
            let inner = contents
                .iter()
                .map(|x| print_str(x, print_readably))
                .collect::<Vec<String>>()
                .join(" ");
            format!("({inner})")
        }
        Atom::Vector(contents) => {
            let inner = contents
                .iter()
                .map(|x| print_str(x, print_readably))
                .collect::<Vec<String>>()
                .join(" ");
            format!("[{inner}]")
        }
        Atom::Map(map) => {
            let contents = map
                .iter()
                .map(|(key, value)| {
                    format!(
                        "{} {}",
                        print_str(key, print_readably),
                        print_str(value, print_readably)
                    )
                })
                .collect::<Vec<String>>()
                .join(" ");
            format!("{{{contents}}}").to_string()
        }
        Atom::Function(_) => "#<function>".to_string(),
    }
}
