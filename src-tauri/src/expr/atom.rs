
use super::number::Number;

#[derive(Debug, Clone)]
pub enum Atom {
  Number(Number),
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
