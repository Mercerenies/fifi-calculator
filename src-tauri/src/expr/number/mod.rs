
mod visitor;

use visitor::NumberPair;

use num::{BigInt, BigRational, Zero, ToPrimitive, One};
use thiserror::Error;
use once_cell::sync::Lazy;
use regex::Regex;

use std::fmt::{self, Display, Formatter};
use std::ops::{Add, Sub, Mul, Div, Neg};
use std::str::FromStr;

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

#[derive(Error, Debug)]
#[non_exhaustive]
pub struct ParseNumberError {}

/// The different ways a number can be represented. These are ordered
/// in terms of priority, so if `a <= b`, that implies that the
/// arithmetic system here will try to use representation `a` before
/// resorting to representation `b`. For instance, `Integer <= Float`
/// implies that we will try to use integer arithmetic and only resort
/// to floating-point values when necessary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NumberRepr {
  Integer, Ratio, Float,
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

impl Number {
  pub fn repr(&self) -> NumberRepr {
    match &self.inner {
      NumberImpl::Integer(_) => NumberRepr::Integer,
      NumberImpl::Ratio(_) => NumberRepr::Ratio,
      NumberImpl::Float(_) => NumberRepr::Float,
    }
  }

  pub fn strict_eq(&self, other: &Number) -> bool {
    self.repr() == other.repr() && self == other
  }

  /// Produces a rational number with repr [`NumberRepr::Ratio`].
  /// Panics if `denom == 0`.
  pub fn ratio(numer: BigInt, denom: BigInt) -> Number {
    Number::from(BigRational::new(numer, denom))
  }

  /// Simplify representation. If the number is stored as a rational
  /// but is in fact an integer, convert to an integer representation.
  /// This function will never simplify a floating-point
  /// representation to an exact representation, even if the
  /// represented float is current integral in value.
  fn simplify(self) -> Number {
    if let NumberImpl::Ratio(r) = &self.inner {
      if r.denom().is_one() {
        return Number::from(r.numer().clone());
      }
    }
    self
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
    Number { inner: NumberImpl::Ratio(r) }.simplify()
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
      NumberImpl::Integer(i) => {
        i.fmt(f)
      }
      NumberImpl::Ratio(r) => {
        write!(f, "{}:{}", r.numer(), r.denom())
      }
      NumberImpl::Float(d) => {
        // If the float is actually a (small) integer, force one decimal
        // point. Otherwise, use default printer.
        if d.fract().is_zero() && d.abs() < u64::MAX as f64 {
          write!(f, "{:.1}", d)
        } else {
          write!(f, "{}", d)
        }
      }
    }
  }
}

impl Display for ParseNumberError {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "Failed to parse number")
  }
}

/// `PartialEq` impl for `Number` compares the numerical value and
/// ignores the representation. To include the representation, use
/// [`Number::strict_eq`].
impl PartialEq for Number {
  fn eq(&self, other: &Number) -> bool {
    match NumberPair::promote(self.clone(), other.clone()) {
      NumberPair::Integers(left, right) => left == right,
      NumberPair::Ratios(left, right) => left == right,
      NumberPair::Floats(left, right) => left == right,
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

impl Neg for Number {
  type Output = Number;

  fn neg(self) -> Number {
    match self.inner {
      NumberImpl::Integer(i) => Number::from(-i),
      NumberImpl::Ratio(r) => Number::from(-r),
      NumberImpl::Float(f) => Number::from(-f),
    }
  }
}

impl Neg for &Number {
  type Output = Number;

  fn neg(self) -> Number {
    (*self).clone().neg()
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

impl FromStr for Number {
  type Err = ParseNumberError;

  fn from_str(s: &str) -> Result<Number, ParseNumberError> {
    parse_integer(s).or_else(|| {
      parse_ratio(s)
    }).or_else(|| {
      parse_float(s)
    }).ok_or(ParseNumberError {})
  }
}

fn parse_integer(s: &str) -> Option<Number> {
  BigInt::from_str(s).map(Number::from).ok()
}

fn parse_ratio(s: &str) -> Option<Number> {
  static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"([+-]?\d+):([+-]?\d+)").unwrap());
  let caps = RE.captures(s)?;
  // Note: We panic if BigInt::from_str fails here, since the regex
  // *should* guarantee it succeeds. If that assumption fails, it's a
  // bug and I want to know.
  let numerator = BigInt::from_str(caps.get(1).unwrap().as_str()).unwrap();
  let denominator = BigInt::from_str(caps.get(2).unwrap().as_str()).unwrap();
  if denominator.is_zero() {
    // Fail the parse.
    return None;
  }
  let ratio = BigRational::new(numerator, denominator);
  Some(Number::from(ratio))
}

fn parse_float(s: &str) -> Option<Number> {
  f64::from_str(s).map(Number::from).ok()
}

#[cfg(test)]
mod tests {
  use super::*;
  use num::bigint::Sign;

  // TODO Incomplete test suite in this file :)

  fn roundtrip_display(number: Number) -> Number {
    Number::from_str(&number.to_string()).unwrap()
  }

  fn assert_roundtrip_display(number: Number) {
    assert!(number.strict_eq(&roundtrip_display(number.clone())));
  }

  #[test]
  fn test_display_roundtrip() {
    // Small integers
    assert_roundtrip_display(Number::from(0i64));
    assert_roundtrip_display(Number::from(10i64));
    assert_roundtrip_display(Number::from(999i64));
    assert_roundtrip_display(Number::from(-99i64));
    // Big integers
    assert_roundtrip_display(Number::from(BigInt::from_slice(Sign::Plus, &[9, 10, 100, 488, 22, 3])));
    assert_roundtrip_display(Number::from(BigInt::from_slice(Sign::Minus, &[9, 100, 488, 10, 22, 3])));
    // Rational numbers
    assert_roundtrip_display(Number::ratio(BigInt::from(9), BigInt::from(100)));
    assert_roundtrip_display(Number::ratio(BigInt::from(-100), BigInt::from(3)));
    assert_roundtrip_display(Number::ratio(BigInt::from(38324), BigInt::from(288)));
    // Floats
    assert_roundtrip_display(Number::from(9.1));
    assert_roundtrip_display(Number::from(3.1415));
    assert_roundtrip_display(Number::from(93763782432.22));
    assert_roundtrip_display(Number::from(-9.000001));
    assert_roundtrip_display(Number::from(-8.0));
  }

  #[test]
  fn test_parse_integer() {
    assert!(Number::from_str("7").unwrap().strict_eq(&Number::from(7i64)));
    assert!(Number::from_str("-99").unwrap().strict_eq(&Number::from(-99i64)));
    assert!(Number::from_str("888888888888888888888888888888888").unwrap().strict_eq(&Number::from(BigInt::from_str("888888888888888888888888888888888").unwrap())));
  }

  #[test]
  fn test_parse_ratio() {
    assert!(Number::from_str("1:2").unwrap().strict_eq(&Number::ratio(BigInt::from(1), BigInt::from(2))));
    assert!(Number::from_str("-7:9").unwrap().strict_eq(&Number::ratio(BigInt::from(-7), BigInt::from(9))));
    assert!(Number::from_str("7:-9").unwrap().strict_eq(&Number::ratio(BigInt::from(-7), BigInt::from(9))));
    assert!(Number::from_str("1:0").is_err());
  }

  #[test]
  fn test_parse_float() {
    assert!(Number::from_str("1.9").unwrap().strict_eq(&Number::from(1.9f64)));
    assert!(Number::from_str("-88.0").unwrap().strict_eq(&Number::from(-88f64)));
    assert!(Number::from_str("3e-6").unwrap().strict_eq(&Number::from(3e-6f64)));
    assert!(Number::from_str("3e6").unwrap().strict_eq(&Number::from(3e6f64)));
  }
}
