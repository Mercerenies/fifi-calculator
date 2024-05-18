
use super::{Number, NumberRepr};
use crate::util::stricteq::StrictEq;

use num::{Zero, One, BigInt};
use approx::{AbsDiffEq, RelativeEq};

use std::fmt::{self, Formatter, Display};
use std::ops;
use std::cmp::Ordering;

/// A complex number has a real part and an imaginary part.
///
/// Note that each component of a `ComplexNumber` independently has a
/// representation and may or may not be exact. So it is possible to
/// use this one type to do both engineering approximations (with
/// representation [`NumberRepr::Float`]) and exact Gaussian integer
/// mathematics (with representation [`NumberRepr::Integer`]).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ComplexNumber {
  real: Number,
  imag: Number,
}

impl ComplexNumber {
  pub fn new(real: Number, imag: Number) -> Self {
    Self { real, imag }
  }

  pub fn real(&self) -> &Number {
    &self.real
  }

  pub fn imag(&self) -> &Number {
    &self.imag
  }

  pub fn real_repr(&self) -> NumberRepr {
    self.real.repr()
  }

  pub fn imag_repr(&self) -> NumberRepr {
    self.imag.repr()
  }

  pub fn from_real(real: Number) -> Self {
    Self { real, imag: Number::zero() }
  }

  pub fn from_imag(imag: Number) -> Self {
    Self { real: Number::zero(), imag }
  }

  /// Constructs an (inexact) complex number from polar coordinates,
  /// with `phi` represented in radians.
  pub fn from_polar_inexact(r: f64, phi: f64) -> Self {
    Self {
      real: Number::from(r * phi.cos()),
      imag: Number::from(r * phi.sin()),
    }
  }

  /// Computes the square of the absolute value of this complex
  /// number.
  pub fn abs_sqr(&self) -> Number {
    &self.real * &self.real + &self.imag * &self.imag
  }

  /// Computes the absolute value of this complex number. Note that,
  /// for now, this is always an inexact quantity.
  pub fn abs(&self) -> f64 {
    // TODO Do this exactly, when we can.
    self.abs_sqr().powf(0.5)
  }

  pub fn recip(&self) -> ComplexNumber {
    let abs_sqr = self.abs_sqr();
    ComplexNumber {
      real: &self.real / &abs_sqr,
      imag: - &self.imag / abs_sqr,
    }
  }
}

impl StrictEq for ComplexNumber {
  /// Compares the inner components using [`Number::strict_eq`].
  fn strict_eq(&self, other: &ComplexNumber) -> bool {
    self.real.strict_eq(&other.real) && self.imag.strict_eq(&other.imag)
  }
}

impl Display for ComplexNumber {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "({}, {})", self.real, self.imag)
  }
}

impl ops::Add for ComplexNumber {
  type Output = ComplexNumber;

  fn add(self, other: ComplexNumber) -> ComplexNumber {
    ComplexNumber {
      real: self.real + other.real,
      imag: self.imag + other.imag,
    }
  }
}

impl ops::Add for &ComplexNumber {
  type Output = ComplexNumber;

  fn add(self, other: &ComplexNumber) -> ComplexNumber {
    self.to_owned() + other.to_owned()
  }
}

impl ops::Sub for ComplexNumber {
  type Output = ComplexNumber;

  fn sub(self, other: ComplexNumber) -> ComplexNumber {
    ComplexNumber {
      real: self.real - other.real,
      imag: self.imag - other.imag,
    }
  }
}

impl ops::Sub for &ComplexNumber {
  type Output = ComplexNumber;

  fn sub(self, other: &ComplexNumber) -> ComplexNumber {
    self.to_owned() - other.to_owned()
  }
}

impl ops::Mul for ComplexNumber {
  type Output = ComplexNumber;

  fn mul(self, other: ComplexNumber) -> ComplexNumber {
    ComplexNumber {
      real: &self.real * &other.real - &self.imag * &other.imag,
      imag: &self.imag * &other.real + &self.real * &other.imag,
    }
  }
}

impl ops::Mul for &ComplexNumber {
  type Output = ComplexNumber;

  fn mul(self, other: &ComplexNumber) -> ComplexNumber {
    self.to_owned() * other.to_owned()
  }
}

/// Exactness-preserving division.
impl ops::Div for ComplexNumber {
  type Output = ComplexNumber;

  fn div(self, other: ComplexNumber) -> ComplexNumber {
    let denominator = &other.real * &other.real + &other.imag * &other.imag;
    ComplexNumber {
      real: (&self.real * &other.real + &self.imag * &other.imag) / denominator.clone(),
      imag: (&self.imag * &other.real - &self.real * &other.imag) / denominator,
    }
  }
}

impl ops::Div for &ComplexNumber {
  type Output = ComplexNumber;

  fn div(self, other: &ComplexNumber) -> ComplexNumber {
    self.to_owned() / other.to_owned()
  }
}

impl Zero for ComplexNumber {
  fn zero() -> Self {
    Self::new(Number::zero(), Number::zero())
  }

  fn is_zero(&self) -> bool {
    self.real.is_zero() && self.imag.is_zero()
  }
}

impl One for ComplexNumber {
  fn one() -> Self {
    Self::new(Number::one(), Number::zero())
  }

  fn is_one(&self) -> bool {
    self.real.is_one() && self.imag.is_zero()
  }
}

impl AbsDiffEq for ComplexNumber {
  type Epsilon = f64;

  fn default_epsilon() -> f64 {
    <f64 as AbsDiffEq>::default_epsilon()
  }

  fn abs_diff_eq(&self, other: &Self, epsilon: f64) -> bool {
    self.abs() - other.abs() <= epsilon
  }
}

impl RelativeEq for ComplexNumber {
  fn default_max_relative() -> f64 {
    <f64 as RelativeEq>::default_max_relative()
  }

  fn relative_eq(&self, other: &Self, epsilon: f64, max_relative: f64) -> bool {
    (self - other).abs() <= epsilon * f64::max(self.abs(), other.abs()) * max_relative
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::assert_strict_eq;

  #[test]
  fn test_add() {
    assert_strict_eq!(
      ComplexNumber::new(Number::from(1), Number::from(2)) +
        ComplexNumber::new(Number::from(3), Number::from(4)),
      ComplexNumber::new(Number::from(4), Number::from(6))
    );
    assert_strict_eq!(
      ComplexNumber::new(Number::from(1.0), Number::from(2)) +
        ComplexNumber::new(Number::from(3), Number::from(4)),
      ComplexNumber::new(Number::from(4.0), Number::from(6))
    );
  }

  #[test]
  fn test_sub() {
    assert_strict_eq!(
      ComplexNumber::new(Number::from(1), Number::from(10)) -
        ComplexNumber::new(Number::from(3), Number::from(4)),
      ComplexNumber::new(Number::from(-2), Number::from(6))
    );
    assert_strict_eq!(
      ComplexNumber::new(Number::from(1.0), Number::from(2)) -
        ComplexNumber::new(Number::from(3), Number::from(5)),
      ComplexNumber::new(Number::from(-2.0), Number::from(-3))
    );
  }

  #[test]
  fn test_mul() {
    assert_strict_eq!(
      ComplexNumber::new(Number::from(1), Number::from(10)) *
        ComplexNumber::new(Number::from(2), Number::from(20)),
      ComplexNumber::new(Number::from(-198), Number::from(40)),
    );
    assert_strict_eq!(
      ComplexNumber::new(Number::from(1), Number::from(10.0)) *
        ComplexNumber::new(Number::from(2), Number::from(20)),
      ComplexNumber::new(Number::from(-198.0), Number::from(40.0)),
    );
  }

  #[test]
  fn test_div() {
    assert_strict_eq!(
      ComplexNumber::new(Number::from(1), Number::from(10)) /
        ComplexNumber::new(Number::from(2), Number::from(20)),
      ComplexNumber::new(Number::ratio(1, 2), Number::from(0)),
    );
    assert_strict_eq!(
      ComplexNumber::new(Number::from(1.0), Number::from(10)) /
        ComplexNumber::new(Number::from(2), Number::from(20)),
      ComplexNumber::new(Number::from(0.5), Number::from(0.0)),
    );
  }

  #[test]
  fn test_recip() {
    assert_strict_eq!(
      ComplexNumber::new(Number::from(2), Number::from(10)).recip(),
      ComplexNumber::new(Number::ratio(2, 104), Number::ratio(-10, 104)),
    );
  }
}
