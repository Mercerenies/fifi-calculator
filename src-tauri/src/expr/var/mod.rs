
pub mod constants;
pub mod table;

use crate::util::prism::Prism;

use regex::Regex;
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};

use std::error::{Error as StdError};
use std::fmt::{self, Display, Formatter};

/// A variable in an equation, left intentionally un-evaluated.
///
/// Variables are identified by strings. A variable's name must begin
/// with a letter, followed by zero or more letters, digits, or
/// apostrophes. This structure enforces these constraints.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Var(String);

/// A prism which parses a string as a variable.
#[derive(Debug, Clone, Copy, Default)]
pub struct StringToVar;

#[derive(Clone, Debug)]
pub struct TryFromStringError {
  original_string: String,
}

pub static VALID_NAME_RE: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"^[a-zA-Z$][a-zA-Z$0-9']*$").unwrap()
});

impl Var {
  pub fn new(name: impl Into<String>) -> Option<Self> {
    Self::try_from(name.into()).ok()
  }

  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl StringToVar {
  pub fn new() -> Self {
    Self
  }
}

impl TryFrom<String> for Var {
  type Error = TryFromStringError;

  fn try_from(name: String) -> Result<Self, Self::Error> {
    if VALID_NAME_RE.is_match(&name) {
      Ok(Self(name))
    } else {
      Err(TryFromStringError { original_string: name })
    }
  }
}

impl From<Var> for String {
  fn from(v: Var) -> Self {
    v.0
  }
}

impl Prism<String, Var> for StringToVar {
  fn narrow_type(&self, s: String) -> Result<Var, String> {
    Var::try_from(s.to_owned()).map_err(|e| e.original_string)
  }

  fn widen_type(&self, v: Var) -> String {
    v.0
  }
}

impl Display for Var {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", &self.0)
  }
}

impl Display for TryFromStringError {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "Invalid variable name")
  }
}

impl StdError for TryFromStringError {}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_valid_variable_names() {
    Var::new("abc").unwrap();
    Var::new("q0").unwrap();
    Var::new("q999").unwrap();
    Var::new("e0e0e0").unwrap();
    Var::new("x1234567890").unwrap();
    Var::new("AaAaAa").unwrap();
    Var::new("aBCd").unwrap();
    Var::new("abc'").unwrap();
    Var::new("abc''''").unwrap();
    Var::new("a''''A").unwrap();
    Var::new("A''''A").unwrap();
    Var::new("A''''a").unwrap();
    Var::new("r0'0").unwrap();
    Var::new("W''9''W").unwrap();
    Var::new("$").unwrap();
    Var::new("$'").unwrap();
    Var::new("$123").unwrap();
    Var::new("$a$").unwrap();
    Var::new("$A$").unwrap();
    Var::new("A$").unwrap();
  }

  #[test]
  fn test_invalid_variable_names() {
    assert_eq!(Var::new(""), None);
    assert_eq!(Var::new("0"), None);
    assert_eq!(Var::new("0a"), None);
    assert_eq!(Var::new("'"), None);
    assert_eq!(Var::new("'1"), None);
    assert_eq!(Var::new("2'"), None);
    assert_eq!(Var::new("a b"), None);
    assert_eq!(Var::new(" "), None);
    assert_eq!(Var::new("\t"), None);
    assert_eq!(Var::new("c-d"), None);
    assert_eq!(Var::new("@"), None);
    assert_eq!(Var::new("abc "), None);
    assert_eq!(Var::new(" abc"), None);
    assert_eq!(Var::new("3''2"), None);
    assert_eq!(Var::new("'''''''"), None);
    assert_eq!(Var::new("$^"), None);
    assert_eq!(Var::new("$["), None);
  }

  #[test]
  fn test_widen_var_prism() {
    let prism = StringToVar;
    let var = Var::new("abc").unwrap();
    assert_eq!(prism.widen_type(var), "abc");
  }

  #[test]
  fn test_narrow_var_prism_success() {
    let prism = StringToVar;
    let s = "abc";
    assert_eq!(prism.narrow_type(s.to_owned()).unwrap(), Var::new(s).unwrap());
  }

  #[test]
  fn test_narrow_var_prism_failure() {
    let prism = StringToVar;
    let s = "00";
    assert_eq!(prism.narrow_type(s.to_owned()).unwrap_err(), "00");
  }
}
