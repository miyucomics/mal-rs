#![warn(clippy::pedantic)]

use crate::{reader::unwrap_map_key, types::Atom};

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
            if print_readably {
                let string = string
                    .replace('\\', "\\\\")
                    .replace('\n', "\\n")
                    .replace('"', "\\\"");
                format!("\"{string}\"")
            } else {
                string.to_string()
            }
        }
        Atom::Symbol(value) => value.to_string(),
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
                        print_str(&unwrap_map_key(key), print_readably),
                        print_str(value, print_readably)
                    )
                })
                .collect::<Vec<String>>()
                .join(" ");
            format!("{{{contents}}}").to_string()
        }
        Atom::Function(_) | Atom::Lambda { .. } => "#<function>".to_string(),
    }
}
