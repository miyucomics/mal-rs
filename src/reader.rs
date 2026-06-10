#![warn(clippy::pedantic)]

use core::fmt;
use std::{
    collections::BTreeMap,
    fmt::{Display, Formatter},
};

use crate::types::Atom;

struct Reader {
    tokens: Vec<String>,
    position: usize,
}

impl Reader {
    fn new(tokens: Vec<String>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    fn next(&mut self) -> Option<&str> {
        self.position += 1;
        self.tokens.get(self.position - 1).map(String::as_str)
    }

    fn peek(&mut self) -> Option<&str> {
        self.tokens.get(self.position).map(String::as_str)
    }
}

#[derive(Debug)]
pub enum ReadError {
    UnexpectedEof,
    UnexpectedToken(String),
    InvalidEscape(String),
}

impl Display for ReadError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEof => write!(formatter, "EOF"),
            Self::UnexpectedToken(token) => write!(formatter, "unexpected token: {token}"),
            Self::InvalidEscape(ch) => write!(formatter, "invalid escape: {ch}"),
        }
    }
}

/// # Errors
/// This function may return a ``ReadError`` if the input can not be parsed.
/// The variants of ``ReadError`` explain what the issue is.
pub fn read_str(code: &str) -> Result<Atom, ReadError> {
    let tokens = tokenize(code)?;
    let mut reader = Reader::new(tokens);
    read_form(&mut reader)
}

const IMMEDIATELY_TOKENIZE: &[char] = &['[', ']', '{', '}', '(', ')', '\'', '`', '~', '^', '@'];

fn tokenize(input: &str) -> Result<Vec<String>, ReadError> {
    let chars: Vec<char> = input.chars().collect();
    let mut tokens: Vec<String> = Vec::new();
    let mut i: usize = 0;

    while i < chars.len() {
        let ch = chars[i];

        if ch.is_whitespace() || ch == ',' {
            i += 1;
            continue;
        }

        if ch == '~' && chars.get(i + 1) == Some(&'@') {
            tokens.push("~@".to_string());
            i += 2;
            continue;
        }

        if IMMEDIATELY_TOKENIZE.contains(&ch) {
            tokens.push(ch.to_string());
            i += 1;
            continue;
        }

        if ch == ';' {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }

        if ch == '"' {
            let (string, consumed) = read_string_literal(&chars[i..])?;
            tokens.push(string);
            i += consumed;
            continue;
        }

        let start = i;
        while i < chars.len()
            && !chars[i].is_whitespace()
            && !IMMEDIATELY_TOKENIZE.contains(&chars[i])
            && chars[i] != ','
            && chars[i] != '"'
            && chars[i] != ';'
        {
            i += 1;
        }
        tokens.push(chars[start..i].iter().collect());
    }

    Ok(tokens)
}

fn read_string_literal(chars: &[char]) -> Result<(String, usize), ReadError> {
    debug_assert_eq!(chars[0], '"');
    let mut buffer = String::new();
    let mut i = 1;

    loop {
        match chars.get(i) {
            None => return Err(ReadError::UnexpectedEof),
            Some('"') => {
                i += 1;
                break;
            }
            Some('\\') => {
                let escaped = match chars.get(i + 1) {
                    Some('n') => '\n',
                    Some('"') => '"',
                    Some('\\') => '\\',
                    Some(c) => return Err(ReadError::InvalidEscape(format!("\\{c}"))),
                    None => return Err(ReadError::UnexpectedEof),
                };
                buffer.push(escaped);
                i += 2;
            }
            Some(c) => {
                buffer.push(*c);
                i += 1;
            }
        }
    }

    Ok((format!("\"{buffer}\""), i))
}

fn read_form(reader: &mut Reader) -> Result<Atom, ReadError> {
    match reader.peek().ok_or(ReadError::UnexpectedEof)? {
        "^" => read_with_meta(reader),
        "'" => read_quotelike(reader, "quote"),
        "`" => read_quotelike(reader, "quasiquote"),
        "~" => read_quotelike(reader, "unquote"),
        "@" => read_quotelike(reader, "deref"),
        "~@" => read_quotelike(reader, "splice-unquote"),
        "(" => read_list(reader),
        "[" => read_vector(reader),
        "{" => read_map(reader),
        _ => read_atom(reader),
    }
}

fn read_with_meta(reader: &mut Reader) -> Result<Atom, ReadError> {
    reader.next();
    let meta = read_form(reader)?;
    let value = read_form(reader)?;
    Ok(Atom::List(vec![
        Atom::Symbol("with-meta".to_string()),
        value,
        meta,
    ]))
}

fn read_quotelike(reader: &mut Reader, name: &str) -> Result<Atom, ReadError> {
    reader.next();
    let inner = read_form(reader)?;
    Ok(Atom::List(vec![Atom::Symbol(name.to_string()), inner]))
}

fn read_list(reader: &mut Reader) -> Result<Atom, ReadError> {
    Ok(Atom::List(read_sequence(reader, ")")?))
}

fn read_vector(reader: &mut Reader) -> Result<Atom, ReadError> {
    Ok(Atom::Vector(read_sequence(reader, "]")?))
}

fn read_map(reader: &mut Reader) -> Result<Atom, ReadError> {
    reader.next();
    let mut map = BTreeMap::new();
    loop {
        match reader.peek().ok_or(ReadError::UnexpectedEof)? {
            "}" => {
                reader.next();
                return Ok(Atom::Map(map));
            }
            _ => map.insert(read_form(reader)?, read_form(reader)?),
        };
    }
}

fn read_sequence(reader: &mut Reader, closing: &str) -> Result<Vec<Atom>, ReadError> {
    reader.next();
    let mut contents = Vec::new();
    loop {
        match reader.peek().ok_or(ReadError::UnexpectedEof)? {
            token if token == closing => {
                reader.next();
                return Ok(contents);
            }
            _ => contents.push(read_form(reader)?),
        }
    }
}

fn read_atom(reader: &mut Reader) -> Result<Atom, ReadError> {
    let token = reader.next().ok_or(ReadError::UnexpectedEof)?;
    match token {
        "nil" => Ok(Atom::Nil),
        "true" => Ok(Atom::Bool(true)),
        "false" => Ok(Atom::Bool(false)),
        token if token.starts_with('"') && token.ends_with('"') => {
            Ok(Atom::Str(token[1..token.len() - 1].to_string()))
        }
        token if token.starts_with(':') => Ok(Atom::Keyword(token[1..].to_string())),
        token if let Ok(n) = token.parse::<i32>() => Ok(Atom::Int(n)),
        token => Ok(Atom::Symbol(token.to_string())),
    }
}
