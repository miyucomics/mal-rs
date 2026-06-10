#![warn(clippy::pedantic)]

use mal::{
    printer::print_str,
    reader::{ReadError, read_str},
    readline::readline,
    types::Atom,
};

fn read(input: &str) -> Result<Atom, ReadError> {
    read_str(input)
}

fn eval(input: Atom) -> Atom {
    input
}

fn print(input: &Atom) -> String {
    print_str(input)
}

fn rep(input: &str) -> Result<String, ReadError> {
    Ok(print(&eval(read(input)?)))
}

fn main() {
    while let Some(ref line) = readline("user> ") {
        if !line.is_empty() {
            let response = rep(line).unwrap_or_else(|err| err.to_string());
            println!("{response}");
        }
    }
}
