
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone)]
pub enum Number {
  Integer(i64),
}

impl From<i64> for Number {
  fn from(i: i64) -> Number {
    Number::Integer(i)
  }
}

impl Display for Number {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Number::Integer(i) => i.fmt(f),
    }
  }
}
