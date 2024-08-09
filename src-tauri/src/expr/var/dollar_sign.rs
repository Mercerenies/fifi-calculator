
//! A variable name satisfying the regex [`DOLLAR_SIGN_NAME_RE`] is a
//! dollar-sign reference variable. These variables are recognized by
//! many commands as references to other positions on the stack.

use super::Var;

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

use std::str::FromStr;
use std::fmt::{self, Display, Formatter};

pub static DOLLAR_SIGN_NAME_RE: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"^\$+([0-9]*)$").unwrap()
});

/// A variable which has been parsed as a dollar-sign reference
/// variable.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DollarSignVar {
  value: usize,
}

impl DollarSignVar {
  /// Create a new `DollarSignVar`.
  pub fn new(value: usize) -> Self {
    Self { value }
  }

  pub fn is_dollar_sign_var(var: &Var) -> bool {
    DOLLAR_SIGN_NAME_RE.is_match(var.as_str())
  }

  pub fn value(&self) -> usize {
    self.value
  }
}

impl Display for DollarSignVar {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "${}", self.value)
  }
}

impl From<DollarSignVar> for Var {
  fn from(var: DollarSignVar) -> Var {
    // unwrap: This will always satisfy the valid variable name regex.
    Var::new(format!("${}", var.value)).unwrap()
  }
}

impl TryFrom<Var> for DollarSignVar {
  type Error = Var;

  fn try_from(var: Var) -> Result<Self, Self::Error> {
    match DollarSignVar::try_from(&var) {
      Ok(dollar_sign_var) => Ok(dollar_sign_var),
      Err(_) => Err(var),
    }
  }
}

impl<'a> TryFrom<&'a Var> for DollarSignVar {
  type Error = &'a Var;

  fn try_from(var: &'a Var) -> Result<Self, Self::Error> {
    let Some(captures) = DOLLAR_SIGN_NAME_RE.captures(var.as_str()) else {
      return Err(var);
    };
    let value = if &captures[1] == "" {
      1
    } else {
      // unwrap: Regex capture is a valid usize.
      usize::from_str(&captures[1]).unwrap()
    };
    Ok(Self { value })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_value_of_dollar_sign_var() {
    assert_eq!(DollarSignVar::new(3).value(), 3);
  }

  #[test]
  fn test_dollar_sign_var_into_var() {
    assert_eq!(
      Var::from(DollarSignVar::new(97)),
      Var::new("$97").unwrap(),
    );
  }

  #[test]
  fn test_try_from_var_into_dollar_sign_var() {
    assert_eq!(DollarSignVar::try_from(Var::new("$97").unwrap()), Ok(DollarSignVar::new(97)));
    assert_eq!(DollarSignVar::try_from(Var::new("$").unwrap()), Ok(DollarSignVar::new(1)));
    assert_eq!(DollarSignVar::try_from(Var::new("xyz").unwrap()), Err(Var::new("xyz").unwrap()));
  }
}
