
use super::visitor::NumberPair;
use super::repr::NumberRepr;
use super::powi_by_repeated_square;
use crate::util::stricteq::StrictEq;

use num::{BigInt, BigRational, Zero, One, FromPrimitive};
use num::integer::div_floor;
use num::traits::ToPrimitive;
use thiserror::Error;
use once_cell::sync::Lazy;
use regex::Regex;
use approx::{AbsDiffEq, RelativeEq, UlpsEq};

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;
use std::ops;
use std::cmp::Ordering;
use std::convert::TryFrom;

/// General-purpose real number type, capable of automatically
/// switching between representations when mathematical functions
/// demand it.
///
/// A real number can be represented as an exact (arbitrary-precision)
/// integer, a rational number, or an IEEE 754 floating point value.
/// Use [`Number::repr`] to get the number's current representation.
///
/// Note that, if a `Number` is being represented as a floating point
/// value, it is safe to assume that the contained `Number` is finite
/// and real. That is, the `Number` struct in particular will never be
/// used to store NaN or infinity constants.
#[derive(Debug, Clone)]
pub struct Number {
  pub(super) inner: NumberImpl,
}

#[derive(Debug, Clone)]
pub(super) enum NumberImpl {
  Integer(Box<BigInt>),
  Ratio(Box<BigRational>),
  Float(f64),
}

#[derive(Debug, Error, Clone)]
#[error("Expected integer, got {number}")]
pub struct TryFromNumberToBigIntError {
  pub number: Number,
  _priv: (),
}

#[derive(Error, Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct ParseNumberError {}

impl Number {
  /// Gets the current representation of the number.
  pub fn repr(&self) -> NumberRepr {
    match &self.inner {
      NumberImpl::Integer(_) => NumberRepr::Integer,
      NumberImpl::Ratio(_) => NumberRepr::Ratio,
      NumberImpl::Float(_) => NumberRepr::Float,
    }
  }

  /// Produces a rational number. If the denominator divides evenly
  /// into the numerator, then the resulting value will have
  /// reprentation `NumberRepr::Integer`. Otherwise, the resulting
  /// value will have representation `NumberRepr::Ratio`.
  ///
  /// Panics if `denom == 0`.
  pub fn ratio(numer: impl Into<BigInt>, denom: impl Into<BigInt>) -> Number {
    Number::from(BigRational::new(numer.into(), denom.into()))
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

  /// Divide, but truncate toward negative infinity.
  pub fn div_floor(&self, other: &Number) -> Number {
    match NumberPair::promote(self.clone(), other.clone()) {
      NumberPair::Integers(left, right) => {
        Number::from(div_floor(left, right))
      }
      NumberPair::Ratios(left, right) => {
        let quotient = (left / right).floor();
        Number::from(quotient.to_integer())
      }
      NumberPair::Floats(left, right) => {
        let quotient = (left / right).floor();
        Number::from(BigInt::from_f64(quotient).expect("floor should produce integer value"))
      }
    }
  }

  pub fn recip(&self) -> Number {
    &Number::one() / self
  }

  /// Raises a `Number` to an integer power.
  ///
  /// The indeterminate form `0^0` is treated as 1.
  pub fn powi(&self, exp: BigInt) -> Number {
    match exp.cmp(&BigInt::zero()) {
      Ordering::Equal => {
        // Exponent is zero, so return 1.
        Number::one()
      }
      Ordering::Less => {
        // Exponent is negative, so make it positive and apply to
        // reciprocal.
        self.recip().powi(- exp)
      }
      Ordering::Greater => {
        match &self.inner {
          NumberImpl::Integer(n) => Number::from(powi_by_repeated_square((**n).clone(), exp)),
          NumberImpl::Ratio(r) => Number::from(powi_by_repeated_square((**r).clone(), exp)),
          NumberImpl::Float(f) =>
            // In the floating case, we're already going to end up
            // with an inexact result, so just rely on the hardware
            // powf implementation instead of repeated squaring.
            Number::from(f.powf(exp.to_f64().unwrap_or(f64::NAN))),
        }
      }
    }
  }

  /// Converts `self` to an `f64` on a best-effort basis.
  pub fn to_f64(&self) -> Option<f64> {
    match &self.inner {
      NumberImpl::Integer(i) => i.to_f64(),
      NumberImpl::Ratio(r) => r.to_f64(),
      NumberImpl::Float(d) => Some(*d),
    }
  }

  /// Raises a `Number` to a floating point power. The result is
  /// always a floating point, which may be NaN if the value does not,
  /// mathematically, exist as a real number.
  pub fn powf(&self, exp: f64) -> f64 {
    let self_as_f64 = self.to_f64().unwrap_or(f64::NAN);
    self_as_f64.powf(exp)
  }

  /// The absolute value of `self`.
  pub fn abs(&self) -> Number {
    if self >= &Number::zero() {
      self.clone()
    } else {
      - self
    }
  }

  /// Returns the sign of the number, as an exact integer.
  ///
  /// If the number is a non-orderable floating-point constant (such
  /// as NaN), then NaN is returned.
  pub fn signum(&self) -> Number {
    match self.partial_cmp(&Number::zero()) {
      Some(Ordering::Greater) => Number::from(1),
      Some(Ordering::Less) => Number::from(-1),
      Some(Ordering::Equal) => Number::from(0),
      None => Number::from(f64::NAN),
    }
  }

  pub fn is_positive(&self) -> bool {
    self > &Number::zero()
  }

  pub fn is_negative(&self) -> bool {
    self < &Number::zero()
  }

  /// Returns the natural logarithm of `self`. This always returns an
  /// inexact floating result. Panics if `self <= 0`.
  pub fn ln(&self) -> Number {
    let x = self.to_f64().expect("Could not convert number to f64");
    assert!(x > 0.0, "Argument to Number::ln should be positive, got {}", x);
    Number::from(x.ln())
  }

  /// Returns the logarithm of `self` with respect to the given base.
  /// `self` and `base` must be positive. If both `self` and `base`
  /// are exact and the result is a small integer, the result may be
  /// represented exactly. This function falls back to an inexact
  /// representation in all other cases.
  pub fn log(&self, base: &Number) -> Number {
    assert!(self > &Number::zero(), "Argument to Number::log should be positive, got {}", self);
    assert!(base > &Number::zero(), "Argument to Number::log should be positive, got {}", base);

    fn float_log(arg: f64, base: f64) -> Number {
      Number::from(arg.log(base))
    }
    fn rational_log(arg: BigRational, base: BigRational) -> Number {
      // Try small integer powers by brute force. If that fails,
      // fall-back to floating-point.
      let mut x = BigRational::one();
      for i in 0..10 {
        if x == arg {
          return Number::from(i);
        }
        x *= &base;
      }
      let arg = arg.to_f64().expect("Could not convert number to f64");
      let base = base.to_f64().expect("Could not convert number to f64");
      float_log(arg, base)
    }

    match NumberPair::promote(self.clone(), base.clone()) {
      NumberPair::Integers(arg, base) => rational_log(arg.into(), base.into()),
      NumberPair::Ratios(arg, base) => rational_log(arg, base),
      NumberPair::Floats(arg, base) => float_log(arg, base),
    }
  }
}

impl TryFrom<Number> for BigInt {
  type Error = TryFromNumberToBigIntError;

  fn try_from(n: Number) -> Result<BigInt, Self::Error> {
    match n.inner {
      NumberImpl::Integer(i) => Ok(*i),
      _ => Err(TryFromNumberToBigIntError {
        number: n,
        _priv: (),
      }),
    }
  }
}

impl From<i32> for Number {
  fn from(i: i32) -> Number {
    Number { inner: NumberImpl::Integer(Box::new(i.into())) }
  }
}

/// Constructs an integer number from an `i64`.
impl From<i64> for Number {
  fn from(i: i64) -> Number {
    Number { inner: NumberImpl::Integer(Box::new(i.into())) }
  }
}

impl From<usize> for Number {
  fn from(i: usize) -> Number {
    Number { inner: NumberImpl::Integer(Box::new(i.into())) }
  }
}

/// Constructs an integer number from an arbitrary-sized `BigInt`
/// integer.
impl From<BigInt> for Number {
  fn from(i: BigInt) -> Number {
    Number { inner: NumberImpl::Integer(Box::new(i)) }
  }
}

/// Constructs a rational number from a `BigRational` value.
impl From<BigRational> for Number {
  fn from(r: BigRational) -> Number {
    Number { inner: NumberImpl::Ratio(Box::new(r)) }.simplify()
  }
}

/// Constructs a floating-point number from an `f64` value.
impl From<f64> for Number {
  fn from(f: f64) -> Number {
    Number { inner: NumberImpl::Float(f) }
  }
}

impl Default for Number {
  fn default() -> Number {
    Number::from(0)
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
/// [`StrictEq::strict_eq`].
///
/// # Examples
///
/// ```
/// # use fifi::expr::number::Number;
/// assert_eq!(Number::from(0), Number::from(0));
/// assert_eq!(Number::from(0), Number::from(0.0));
/// assert_eq!(Number::ratio(1, 2), Number::from(0.5));
/// assert_ne!(Number::ratio(1, 3), Number::from(0.5));
/// ```
impl PartialEq for Number {
  fn eq(&self, other: &Number) -> bool {
    match NumberPair::promote(self.clone(), other.clone()) {
      NumberPair::Integers(left, right) => left == right,
      NumberPair::Ratios(left, right) => left == right,
      NumberPair::Floats(left, right) => left == right,
    }
  }
}

impl StrictEq for Number {
  /// Compares both the representation and the value of the type. This
  /// is stricter than the standard [`PartialEq`] implementation.
  ///
  /// # Examples
  ///
  /// ```
  /// # use fifi::expr::number::Number;
  /// # use fifi::util::stricteq::StrictEq;
  /// assert!(Number::from(0).strict_eq(&Number::from(0)));
  /// assert!(!Number::from(0).strict_eq(&Number::from(0.0)));
  /// ```
  fn strict_eq(&self, other: &Number) -> bool {
    self.repr() == other.repr() && self == other
  }
}

impl AbsDiffEq for Number {
  type Epsilon = f64;

  fn default_epsilon() -> f64 {
    <f64 as AbsDiffEq>::default_epsilon()
  }

  fn abs_diff_eq(&self, other: &Number, epsilon: f64) -> bool {
    let left = self.to_f64().unwrap_or(f64::NAN);
    let right = other.to_f64().unwrap_or(f64::NAN);
    left.abs_diff_eq(&right, epsilon)
  }
}

impl RelativeEq for Number {
  fn default_max_relative() -> f64 {
    <f64 as RelativeEq>::default_max_relative()
  }

  fn relative_eq(&self, other: &Number, epsilon: f64, max_relative: f64) -> bool {
    let left = self.to_f64().unwrap_or(f64::NAN);
    let right = other.to_f64().unwrap_or(f64::NAN);
    left.relative_eq(&right, epsilon, max_relative)
  }
}

impl UlpsEq for Number {
  fn default_max_ulps() -> u32 {
    <f64 as UlpsEq>::default_max_ulps()
  }

  fn ulps_eq(&self, other: &Number, epsilon: f64, max_ulps: u32) -> bool {
    let left = self.to_f64().unwrap_or(f64::NAN);
    let right = other.to_f64().unwrap_or(f64::NAN);
    left.ulps_eq(&right, epsilon, max_ulps)
  }
}

impl PartialOrd for Number {
  fn partial_cmp(&self, other: &Number) -> Option<Ordering> {
    match NumberPair::promote(self.clone(), other.clone()) {
      NumberPair::Integers(left, right) => left.partial_cmp(&right),
      NumberPair::Ratios(left, right) => left.partial_cmp(&right),
      NumberPair::Floats(left, right) => left.partial_cmp(&right),
    }
  }
}

impl ops::Add for Number {
  type Output = Number;

  fn add(self, other: Number) -> Number {
    match NumberPair::promote(self, other) {
      NumberPair::Integers(left, right) => Number::from(left + right),
      NumberPair::Ratios(left, right) => Number::from(left + right),
      NumberPair::Floats(left, right) => Number::from(left + right),
    }
  }
}

impl ops::Add for &Number {
  type Output = Number;

  fn add(self, other: &Number) -> Number {
    (*self).clone() + (*other).clone()
  }
}

impl ops::Sub for Number {
  type Output = Number;

  fn sub(self, other: Number) -> Number {
    match NumberPair::promote(self, other) {
      NumberPair::Integers(left, right) => Number::from(left - right),
      NumberPair::Ratios(left, right) => Number::from(left - right),
      NumberPair::Floats(left, right) => Number::from(left - right),
    }
  }
}

impl ops::Sub for &Number {
  type Output = Number;

  fn sub(self, other: &Number) -> Number {
    (*self).clone() - (*other).clone()
  }
}

impl ops::Mul for Number {
  type Output = Number;

  fn mul(self, other: Number) -> Number {
    match NumberPair::promote(self, other) {
      NumberPair::Integers(left, right) => Number::from(left * right),
      NumberPair::Ratios(left, right) => Number::from(left * right),
      NumberPair::Floats(left, right) => Number::from(left * right),
    }
  }
}

impl ops::Mul for &Number {
  type Output = Number;

  fn mul(self, other: &Number) -> Number {
    (*self).clone() * (*other).clone()
  }
}

/// This division operation will not truncate, even if given two
/// values of representation `NumberRepr::Integer`. However, it will
/// preserve exactness, so given two exact inputs, the output will be
/// exact as well.
impl ops::Div for Number {
  type Output = Number;

  fn div(self, other: Number) -> Number {
    match NumberPair::promote(self, other) {
      NumberPair::Integers(left, right) =>
        Number::from(BigRational::from(left) / BigRational::from(right)),
      NumberPair::Ratios(left, right) => Number::from(left / right),
      NumberPair::Floats(left, right) => Number::from(left / right),
    }
  }
}

impl ops::Div for &Number {
  type Output = Number;

  fn div(self, other: &Number) -> Number {
    (*self).clone() / (*other).clone()
  }
}

/// We implement the Euclidean remainder here, for simplicitly in
/// interacting with the calculator functions (all of which use the
/// Euclidean remainder).
impl ops::Rem for Number {
  type Output = Number;

  fn rem(self, other: Number) -> Number {
    let result = match NumberPair::promote(self, other.clone()) {
      NumberPair::Integers(left, right) => Number::from(left % right),
      NumberPair::Ratios(left, right) => Number::from(left % right),
      NumberPair::Floats(left, right) => Number::from(left % right),
    };
    // Adjust sign to match divisor
    if result.signum() * other.signum() == Number::from(-1) {
      result + other
    } else {
      result
    }
  }
}

impl ops::Rem for &Number {
  type Output = Number;

  fn rem(self, other: &Number) -> Number {
    (*self).clone() % (*other).clone()
  }
}

impl ops::Neg for Number {
  type Output = Number;

  fn neg(self) -> Number {
    match self.inner {
      NumberImpl::Integer(i) => Number::from(- (*i)),
      NumberImpl::Ratio(r) => Number::from(- (*r)),
      NumberImpl::Float(f) => Number::from(- f),
    }
  }
}

impl ops::Neg for &Number {
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
  use crate::{assert_strict_eq, assert_strict_ne};

  use approx::assert_abs_diff_eq;

  use num::bigint::Sign;

  // TODO Missing tests: PartialOrd, to_f64, Signed

  fn roundtrip_display(number: Number) -> Number {
    Number::from_str(&number.to_string()).unwrap()
  }

  fn assert_roundtrip_display(number: Number) {
    assert_strict_eq!(number.clone(), roundtrip_display(number));
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
    assert_strict_eq!(Number::from_str("7").unwrap(), Number::from(7i64));
    assert_strict_eq!(Number::from_str("-99").unwrap(), Number::from(-99i64));
    assert_strict_eq!(
      Number::from_str("888888888888888888888888888888888").unwrap(),
      Number::from(BigInt::from_str("888888888888888888888888888888888").unwrap()),
    );
  }

  #[test]
  fn test_parse_ratio() {
    assert_strict_eq!(
      Number::from_str("1:2").unwrap(),
      Number::ratio(BigInt::from(1), BigInt::from(2)),
    );
    assert_strict_eq!(
      Number::from_str("-7:9").unwrap(),
      Number::ratio(BigInt::from(-7), BigInt::from(9)),
    );
    assert_strict_eq!(
      Number::from_str("7:-9").unwrap(),
      Number::ratio(BigInt::from(-7), BigInt::from(9)),
    );
    assert_eq!(Number::from_str("1:0"), Err(ParseNumberError {}));
  }

  #[test]
  fn test_parse_float() {
    assert_strict_eq!(Number::from_str("1.9").unwrap(), Number::from(1.9f64));
    assert_strict_eq!(Number::from_str("-88.0").unwrap(), Number::from(-88f64));
    assert_strict_eq!(Number::from_str("3e-6").unwrap(), Number::from(3e-6f64));
    assert_strict_eq!(Number::from_str("3e6").unwrap(), Number::from(3e6f64));
  }

  #[test]
  fn test_number_repr() {
    assert_eq!(Number::zero().repr(), NumberRepr::Integer);
    assert_eq!(Number::one().repr(), NumberRepr::Integer);
    assert_eq!(Number::from(BigInt::from(9)).repr(), NumberRepr::Integer);
    assert_eq!(Number::from(999).repr(), NumberRepr::Integer);
    assert_eq!(Number::ratio(BigInt::from(1), BigInt::from(2)).repr(), NumberRepr::Ratio);
    assert_eq!(Number::ratio(BigInt::from(-1), BigInt::from(9)).repr(), NumberRepr::Ratio);
    assert_eq!(Number::from(BigRational::new(BigInt::from(-1), BigInt::from(9))).repr(), NumberRepr::Ratio);
    assert_eq!(Number::from(9.9).repr(), NumberRepr::Float);
  }

  #[test]
  fn test_ratio_repr_simplification() {
    // If we explicitly construct a rational number but it can be
    // represented as an integer, we should use the integer repr.
    assert_eq!(Number::ratio(BigInt::from(2), BigInt::from(1)).repr(), NumberRepr::Integer);
    assert_eq!(Number::ratio(BigInt::from(3), BigInt::from(3)).repr(), NumberRepr::Integer);
    assert_eq!(Number::ratio(BigInt::from(9), BigInt::from(-3)).repr(), NumberRepr::Integer);
  }

  #[test]
  fn test_strict_eq() {
    assert_strict_eq!(Number::from(3), Number::from(3));
    assert_strict_ne!(Number::from(3), Number::from(3.0));
    assert_strict_eq!(Number::from(3), Number::ratio(9, 3));
    assert_strict_ne!(Number::from(0.5), Number::ratio(1, 2));
    assert_strict_ne!(Number::from(3), Number::from(3.001));
    assert_strict_eq!(Number::ratio(1, 2), Number::ratio(2, 4));
    assert_strict_eq!(Number::ratio(-1, 2), Number::ratio(1, -2));
  }

  #[test]
  fn test_partial_eq() {
    assert_eq!(Number::from(3), Number::from(3));
    assert_eq!(Number::from(3), Number::from(3.0));
    assert_eq!(Number::from(3), Number::ratio(9, 3));
    assert_eq!(Number::from(0.5), Number::ratio(1, 2));
    assert_ne!(Number::from(3), Number::from(3.001));
  }

  #[test]
  fn test_add() {
    assert_strict_eq!(Number::from(3) + Number::from(3), Number::from(6));
    assert_strict_eq!(Number::from(3) + Number::ratio(1, 2), Number::ratio(7, 2));
    assert_strict_eq!(Number::ratio(1, 2) + Number::ratio(1, 2), Number::from(1));
    assert_strict_eq!(Number::from(3) + Number::from(3.0), Number::from(6.0));
    assert_strict_eq!(Number::ratio(1, 2) + Number::from(3.0), Number::from(3.5));
  }

  #[test]
  fn test_sub() {
    assert_strict_eq!(Number::from(3) - Number::from(3), Number::from(0));
    assert_strict_eq!(Number::from(3) - Number::ratio(1, 2), Number::ratio(5, 2));
    assert_strict_eq!(Number::ratio(1, 2) - Number::ratio(1, 2), Number::from(0));
    assert_strict_eq!(Number::ratio(1, 3) - Number::ratio(2, 3), Number::ratio(-1, 3));
    assert_strict_eq!(Number::from(3) - Number::from(3.0), Number::from(0.0));
    assert_strict_eq!(Number::ratio(1, 2) - Number::from(3.0), Number::from(-2.5));
  }

  #[test]
  fn test_mul() {
    assert_strict_eq!(Number::from(3) * Number::from(3), Number::from(9));
    assert_strict_eq!(Number::from(3) * Number::ratio(1, 2), Number::ratio(3, 2));
    assert_strict_eq!(Number::ratio(1, 2) * Number::ratio(1, 2), Number::ratio(1, 4));
    assert_strict_eq!(Number::ratio(1, 3) * Number::ratio(2, 3), Number::ratio(2, 9));
    assert_strict_eq!(Number::from(3) * Number::from(3.0), Number::from(9.0));
    assert_strict_eq!(Number::ratio(1, 2) * Number::from(3.0), Number::from(1.5));
    assert_strict_eq!(Number::from(0) * Number::from(9.9), Number::from(0.0));
    assert_strict_eq!(Number::from(0) * Number::ratio(2, 3), Number::from(0));
  }

  #[test]
  fn test_div() {
    assert_strict_eq!(Number::from(3) / Number::from(3), Number::from(1));
    assert_strict_eq!(Number::from(3) / Number::from(2), Number::ratio(3, 2));
    assert_strict_eq!(Number::from(3) / Number::ratio(1, 2), Number::from(6));
    assert_strict_eq!(Number::ratio(1, 2) / Number::ratio(1, 2), Number::from(1));
    assert_strict_eq!(Number::from(3) / Number::from(3.0), Number::from(1.0));
    assert_strict_eq!(Number::ratio(1, 2) / Number::from(2.0), Number::from(0.25));
    assert_strict_eq!(Number::from(0) / Number::from(9.9), Number::from(0.0));
    assert_strict_eq!(Number::from(0) / Number::from(9), Number::from(0));
  }

  #[test]
  fn test_mod() {
    assert_strict_eq!(Number::from(3) % Number::from(3), Number::from(0));
    assert_strict_eq!(Number::from(3) % Number::from(2), Number::from(1));
    assert_strict_eq!(Number::from(3) % Number::from(3.0), Number::from(0.0));
    assert_strict_eq!(Number::from(3.0) % Number::from(2), Number::from(1.0));
    assert_strict_eq!(Number::ratio(5, 4) % Number::ratio(1, 2), Number::ratio(1, 4));
    assert_strict_eq!(Number::from(0) % Number::ratio(1, 2), Number::from(0));
  }

  #[test]
  fn test_mod_on_negatives() {
    assert_strict_eq!(Number::from(-4) % Number::from(3), Number::from(2));
    assert_strict_eq!(Number::from(4) % Number::from(-3), Number::from(-2));
    assert_strict_eq!(Number::from(-4) % Number::from(-3), Number::from(-1));
    assert_strict_eq!(Number::ratio(-1, 2) % Number::from(3), Number::ratio(5, 2));
  }

  #[test]
  fn test_div_floor() {
    assert_strict_eq!(Number::from(3).div_floor(&Number::from(3)), Number::from(1));
    assert_strict_eq!(Number::from(0).div_floor(&Number::from(3)), Number::from(0));
    assert_strict_eq!(Number::from(3).div_floor(&Number::from(2)), Number::from(1));
    assert_strict_eq!(Number::from(2).div_floor(&Number::from(3)), Number::from(0));
    assert_strict_eq!(Number::from(8).div_floor(&Number::from(3)), Number::from(2));
    assert_strict_eq!(Number::ratio(8, 3).div_floor(&Number::from(2)), Number::from(1));
    assert_strict_eq!(Number::ratio(8, 3).div_floor(&Number::ratio(1, 2)), Number::from(5));
    assert_strict_eq!(Number::ratio(8, 3).div_floor(&Number::from(0.5)), Number::from(5));
    assert_strict_eq!(Number::from(8.0).div_floor(&Number::from(3.1)), Number::from(2));
  }

  #[test]
  fn test_div_floor_on_negatives() {
    assert_strict_eq!(Number::from(-3).div_floor(&Number::from(2)), Number::from(-2));
    assert_strict_eq!(Number::from(3).div_floor(&Number::from(-2)), Number::from(-2));
    assert_strict_eq!(Number::from(-3).div_floor(&Number::from(-2)), Number::from(1));
    assert_strict_eq!(Number::from(-3).div_floor(&Number::from(-2.0)), Number::from(1));
    assert_strict_eq!(Number::ratio(3, -1).div_floor(&Number::from(-2.0)), Number::from(1));
  }

  #[test]
  fn test_neg() {
    assert_strict_eq!(- Number::from(3), Number::from(-3));
    assert_strict_eq!(- Number::ratio(-1, 2), Number::ratio(1, 2));
    assert_strict_eq!(- Number::from(3.5), Number::from(-3.5));
    assert_strict_eq!(- Number::from(0.0), Number::from(-0.0));
  }

  #[test]
  fn test_is_zero() {
    assert!(Number::from(0).is_zero());
    assert!(Number::from(0.0).is_zero());
    assert!(!Number::from(1.0).is_zero());
    assert!(!Number::from(-3).is_zero());
    assert!(!Number::ratio(-3, 2).is_zero());
  }

  #[test]
  fn test_is_one() {
    assert!(Number::from(1).is_one());
    assert!(Number::from(1.0).is_one());
    assert!(!Number::from(-1).is_one());
    assert!(!Number::ratio(-3, 2).is_one());
    assert!(!Number::from(0).is_one());
    assert!(!Number::from(0.0).is_one());
  }

  #[test]
  fn test_powi_zero_exponent() {
    assert_strict_eq!(Number::from(3).powi(BigInt::zero()), Number::from(1));
    assert_strict_eq!(Number::ratio(1, 2).powi(BigInt::zero()), Number::from(1));
    assert_strict_eq!(Number::from(0).powi(BigInt::zero()), Number::from(1));
    assert_strict_eq!(Number::from(3.2).powi(BigInt::zero()), Number::from(1));
    assert_strict_eq!(Number::from(-1).powi(BigInt::zero()), Number::from(1));
    assert_strict_eq!(Number::from(0.0).powi(BigInt::zero()), Number::from(1));
  }

  #[test]
  fn test_powi_positive_exponent() {
    assert_strict_eq!(Number::from(3).powi(BigInt::from(1)), Number::from(3));
    assert_strict_eq!(Number::from(3).powi(BigInt::from(2)), Number::from(9));
    assert_strict_eq!(Number::from(3).powi(BigInt::from(10)), Number::from(59049));
    assert_strict_eq!(Number::ratio(3, 2).powi(BigInt::from(10)), Number::ratio(59049, 1024));
    assert_strict_eq!(Number::from(3.0).powi(BigInt::from(2)), Number::from(9.0));
  }

  #[test]
  fn test_powi_negative_exponent() {
    assert_strict_eq!(Number::from(3).powi(BigInt::from(-1)), Number::ratio(1, 3));
    assert_strict_eq!(Number::ratio(1, 3).powi(BigInt::from(-2)), Number::from(9));
    assert_strict_eq!(Number::from(3).powi(BigInt::from(-10)), Number::ratio(1, 59049));
    assert_strict_eq!(Number::from(2.0).powi(BigInt::from(-2)), Number::from(0.25));
  }

  #[test]
  fn test_powf_zero_exponent() {
    assert_abs_diff_eq!(Number::from(3).powf(0.0), 1.0);
    assert_abs_diff_eq!(Number::ratio(1, 2).powf(0.0), 1.0);
    assert_abs_diff_eq!(Number::from(3.2).powf(0.0), 1.0);
  }

  #[test]
  fn test_powf_positive_exponent() {
    assert_abs_diff_eq!(Number::from(3).powf(1.0), 3.0);
    assert_abs_diff_eq!(Number::from(3).powf(2.0), 9.0);
    assert_abs_diff_eq!(Number::from(3).powf(2.5), 15.5884572, epsilon = 0.001);
  }

  #[test]
  fn test_number_eq() {
    assert_eq!(Number::from(0), Number::from(0));
    assert_eq!(Number::from(0), Number::from(0.0));
    assert_eq!(Number::ratio(1, 2), Number::from(0.5));
    assert_ne!(Number::ratio(1, 3), Number::from(0.5));
  }

  #[test]
  fn test_number_stricteq() {
    assert!(Number::from(0).strict_eq(&Number::from(0)));
    assert!(!Number::from(0).strict_eq(&Number::from(0.0)));
  }
}
