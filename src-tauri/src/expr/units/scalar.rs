
use crate::expr::Expr;
use crate::expr::number::Number;

use std::ops::{Mul, Div};

/// `UnitScalar` is a newtype wrapper around [`Expr`] which provides
/// `Mul`, `Div`, and other operator implementations useful for
/// interacting with unit arithmetic.
#[derive(Debug, Clone, PartialEq)]
pub struct UnitScalar(pub Expr);

impl From<Number> for UnitScalar {
  fn from(n: Number) -> Self {
    UnitScalar(n.into())
  }
}

impl Mul for UnitScalar {
  type Output = Self;

  fn mul(self, other: Self) -> Self {
    let expr = Expr::call("*", vec![self.0, other.0]);
    Self(expr)
  }
}

impl Mul<&Number> for UnitScalar {
  type Output = Self;

  fn mul(self, other: &Number) -> Self {
    self * UnitScalar::from(other.clone())
  }
}

impl Div for UnitScalar {
  type Output = Self;

  fn div(self, other: Self) -> Self {
    let expr = Expr::call("/", vec![self.0, other.0]);
    Self(expr)
  }
}

impl Div<&Number> for UnitScalar {
  type Output = Self;

  fn div(self, other: &Number) -> Self {
    self / UnitScalar::from(other.clone())
  }
}
