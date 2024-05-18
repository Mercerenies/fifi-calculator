
use super::number::Number;
use super::number::complex::ComplexNumber;

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
