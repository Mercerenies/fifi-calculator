
use super::number::{Number, ComplexNumber};

use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub enum Atom {
  Number(Number),
  Complex(ComplexNumber),
}

impl From<Number> for Atom {
  fn from(n: Number) -> Self {
    Self::Number(n)
  }
}

impl From<i64> for Atom {
  fn from(n: i64) -> Self {
    Self::Number(Number::from(n))
  }
}

impl From<ComplexNumber> for Atom {
  fn from(n: ComplexNumber) -> Self {
    Self::Complex(n)
  }
}

impl Display for Atom {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Atom::Number(n) => write!(f, "{n}"),
      Atom::Complex(n) => write!(f, "{n}"),
    }
  }
}
