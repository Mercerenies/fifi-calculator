
mod visitor;

use visitor::NumberPair;

use num::{BigInt, BigRational, Zero, ToPrimitive, One};

use std::fmt::{self, Display, Formatter};
use std::ops::{Add, Sub, Mul, Div};

/// General-purpose number type, capable of automatically switching
/// between representations when mathematical functions demand it.
#[derive(Debug, Clone)]
pub struct Number {
  inner: NumberImpl,
}

#[derive(Debug, Clone)]
enum NumberImpl {
  Integer(BigInt),
  Ratio(BigRational),
  Float(f64),
}

/// Produce a `BigRational`, or fall back to floats if `b == 0`.
fn rational_div(a: BigRational, b: BigRational) -> Number {
  if b == BigRational::zero() {
    let a = a.to_f64().unwrap_or(f64::NAN);
    Number::from(a / 0.0)
  } else {
    Number::from(a / b)
  }
}

impl From<i64> for Number {
  fn from(i: i64) -> Number {
    Number { inner: NumberImpl::Integer(i.into()) }
  }
}

impl From<BigInt> for Number {
  fn from(i: BigInt) -> Number {
    Number { inner: NumberImpl::Integer(i) }
  }
}

impl From<BigRational> for Number {
  fn from(r: BigRational) -> Number {
    Number { inner: NumberImpl::Ratio(r) }
  }
}

impl From<f64> for Number {
  fn from(f: f64) -> Number {
    Number { inner: NumberImpl::Float(f) }
  }
}

impl Display for Number {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match &self.inner {
      NumberImpl::Integer(i) => i.fmt(f),
      NumberImpl::Ratio(r) => r.fmt(f),
      NumberImpl::Float(d) => d.fmt(f),
    }
  }
}

impl Add for Number {
  type Output = Number;

  fn add(self, other: Number) -> Number {
    match NumberPair::promote(self, other) {
      NumberPair::Integers(left, right) => Number::from(left + right),
      NumberPair::Ratios(left, right) => Number::from(left + right),
      NumberPair::Floats(left, right) => Number::from(left + right),
    }
  }
}

impl Add for &Number {
  type Output = Number;

  fn add(self, other: &Number) -> Number {
    (*self).clone() + (*other).clone()
  }
}

impl Sub for Number {
  type Output = Number;

  fn sub(self, other: Number) -> Number {
    match NumberPair::promote(self, other) {
      NumberPair::Integers(left, right) => Number::from(left - right),
      NumberPair::Ratios(left, right) => Number::from(left - right),
      NumberPair::Floats(left, right) => Number::from(left - right),
    }
  }
}

impl Sub for &Number {
  type Output = Number;

  fn sub(self, other: &Number) -> Number {
    (*self).clone() - (*other).clone()
  }
}

impl Mul for Number {
  type Output = Number;

  fn mul(self, other: Number) -> Number {
    match NumberPair::promote(self, other) {
      NumberPair::Integers(left, right) => Number::from(left * right),
      NumberPair::Ratios(left, right) => Number::from(left * right),
      NumberPair::Floats(left, right) => Number::from(left * right),
    }
  }
}

impl Mul for &Number {
  type Output = Number;

  fn mul(self, other: &Number) -> Number {
    (*self).clone() * (*other).clone()
  }
}

impl Div for Number {
  type Output = Number;

  fn div(self, other: Number) -> Number {
    match NumberPair::promote(self, other) {
      NumberPair::Integers(left, right) => rational_div(left.into(), right.into()),
      NumberPair::Ratios(left, right) => rational_div(left, right),
      NumberPair::Floats(left, right) => Number::from(left / right),
    }
  }
}

impl Div for &Number {
  type Output = Number;

  fn div(self, other: &Number) -> Number {
    (*self).clone() / (*other).clone()
  }
}

impl Zero for Number {
  fn zero() -> Number {
    Number::from(0i64)
  }
  fn is_zero(&self) -> bool {
    match &self.inner {
      NumberImpl::Integer(i) => i.is_zero(),
      NumberImpl::Ratio(r) => r.is_zero(),
      NumberImpl::Float(f) => f.is_zero(),
    }
  }
}

impl One for Number {
  fn one() -> Number {
    Number::from(1i64)
  }
  fn is_one(&self) -> bool {
    match &self.inner {
      NumberImpl::Integer(i) => i.is_one(),
      NumberImpl::Ratio(r) => r.is_one(),
      NumberImpl::Float(f) => f.is_one(),
    }
  }
}
