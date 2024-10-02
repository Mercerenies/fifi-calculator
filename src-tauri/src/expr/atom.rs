
use super::number::Number;
use super::var::Var;
use crate::util::stricteq::StrictEq;

use serde::{Serialize, Deserialize};
use thiserror::Error;

use std::fmt::{self, Write, Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Atom {
  Number(Number),
  String(String),
  Var(Var),
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("Invalid escape sequence '\\{character}'")]
pub struct InvalidEscapeError {
  pub character: char,
}

/// Writes a string, using supported escape sequences from Rust when
/// necessary.
pub fn write_escaped_str(f: &mut impl Write, s: &str) -> fmt::Result {
  f.write_char('"')?;
  for c in s.chars() {
    match c {
      '"' => f.write_str("\\\"")?,
      '\0' => f.write_str("\\0")?,
      '\\' => f.write_str("\\\\")?,
      '\n' => f.write_str("\\n")?,
      '\t' => f.write_str("\\t")?,
      '\r' => f.write_str("\\r")?,
      c => f.write_char(c)?,
    }
  }
  f.write_char('"')
}

pub fn process_escape_char(character: char) -> Result<char, InvalidEscapeError> {
  match character {
    '"' => Ok('"'),
    '\\' => Ok('\\'),
    '0' => Ok('\0'),
    'n' => Ok('\n'),
    't' => Ok('\t'),
    'r' => Ok('\r'),
    character => Err(InvalidEscapeError { character }),
  }
}

impl From<Number> for Atom {
  fn from(n: Number) -> Self {
    Self::Number(n)
  }
}

impl From<String> for Atom {
  fn from(s: String) -> Self {
    Self::String(s)
  }
}

impl From<Var> for Atom {
  fn from(v: Var) -> Self {
    Self::Var(v)
  }
}

impl From<i64> for Atom {
  fn from(n: i64) -> Self {
    Self::Number(Number::from(n))
  }
}

impl From<f64> for Atom {
  fn from(n: f64) -> Self {
    Self::Number(Number::from(n))
  }
}

impl Display for Atom {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Atom::Number(n) => write!(f, "{n}"),
      Atom::Var(v) => write!(f, "{v}"),
      Atom::String(s) => write_escaped_str(f, s),
    }
  }
}

impl StrictEq for Atom {
  fn strict_eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Atom::Number(l), Atom::Number(r)) => l.strict_eq(r),
      (Atom::Var(l), Atom::Var(r)) => l == r,
      (Atom::String(l), Atom::String(r)) => l == r,
      _ => false,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_write_escaped_str_simple() {
    let s = "hello world";
    let mut buf = String::new();
    write_escaped_str(&mut buf, s).unwrap();
    assert_eq!(buf, "\"hello world\"");
  }

  #[test]
  fn test_write_escaped_str_with_escapes() {
    let s = r#"hello
wo\rl""d"#;
    let mut buf = String::new();
    write_escaped_str(&mut buf, s).unwrap();
    assert_eq!(buf, r#""hello\nwo\\rl\"\"d""#);
  }
}
