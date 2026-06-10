#![warn(clippy::pedantic)]

use mal::readline::readline;

fn read(input: &str) -> &str {
    input
}

fn eval(input: &str) -> &str {
    input
}

fn print(input: &str) -> &str {
    input
}

fn rep(input: &str) -> &str {
    print(eval(read(input)))
}

fn main() {
    while let Some(ref line) = readline("user> ") {
        if !line.is_empty() {
            println!("{}", rep(line));
        }
    }
}
