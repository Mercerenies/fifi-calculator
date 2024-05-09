
#[derive(Debug, Clone)]
pub enum Number {
  Integer(i64),
}

impl From<i64> for Number {
  fn from(i: i64) -> Number {
    Number::Integer(i)
  }
}
