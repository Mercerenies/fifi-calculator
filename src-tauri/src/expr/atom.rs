
use super::number::Number;
use super::var::Var;

use serde::{Serialize, Deserialize};

use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Atom {
  Number(Number),
  Var(Var),
}

impl From<Number> for Atom {
  fn from(n: Number) -> Self {
    Self::Number(n)
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
    }
  }
}
