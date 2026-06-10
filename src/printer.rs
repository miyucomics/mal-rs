#![warn(clippy::pedantic)]

use crate::types::Atom;

pub fn print_str(atom: &Atom) -> String {
    match atom {
        Atom::Nil => "nil".to_string(),
        Atom::Bool(value) => {
            if *value {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        Atom::Integer(value) => value.to_string(),
        Atom::Keyword(name) => format!(":{name}"),
        Atom::String(string) => {
            let new = string
                .replace('\\', "\\\\")
                .replace('\n', "\\n")
                .replace('"', "\\\"");
            format!("\"{new}\"")
        }
        Atom::Symbol(value) => value.clone(),
        Atom::List(contents) => {
            let inner = contents
                .iter()
                .map(print_str)
                .collect::<Vec<String>>()
                .join(" ");
            format!("({inner})")
        }
        Atom::Vector(contents) => {
            let inner = contents
                .iter()
                .map(print_str)
                .collect::<Vec<String>>()
                .join(" ");
            format!("[{inner}]")
        }
        Atom::Map(map) => {
            let contents = map
                .iter()
                .map(|(key, value)| format!("{} {}", print_str(key), print_str(value)))
                .collect::<Vec<String>>()
                .join(" ");
            format!("{{{contents}}}").to_string()
        }
    }
}
