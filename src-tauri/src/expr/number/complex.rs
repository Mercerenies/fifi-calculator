
use super::{Number, NumberRepr, powi_by_repeated_square};
use crate::util::stricteq::StrictEq;
use crate::util::angles::Radians;

use num::{Zero, One, BigInt};
use approx::{AbsDiffEq, RelativeEq};
use serde::{Serialize, Deserialize};

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
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComplexNumber {
  real: Number,
  imag: Number,
}

impl ComplexNumber {
  pub const FUNCTION_NAME: &'static str = "complex";

  pub fn new(real: impl Into<Number>, imag: impl Into<Number>) -> Self {
    Self { real: real.into(), imag: imag.into() }
  }

  /// The imaginary constant.
  pub fn ii() -> Self {
    Self::new(0, 1)
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

  pub fn into_parts(self) -> (Number, Number) {
    (self.real, self.imag)
  }

  pub fn from_real(real: Number) -> Self {
    Self { real, imag: Number::zero() }
  }

  pub fn from_imag(imag: Number) -> Self {
    Self { real: Number::zero(), imag }
  }

  /// Complex conjugate of `self`.
  pub fn conj(self) -> Self {
    Self { real: self.real, imag: -self.imag }
  }

  /// Constructs an (inexact) complex number from polar coordinates,
  /// with `phi` represented in radians.
  pub fn from_polar_inexact(r: f64, phi: Radians<f64>) -> Self {
    Self::new(r * phi.cos(), r * phi.sin())
  }

  pub fn to_polar(&self) -> (f64, Radians<f64>) {
    (self.abs(), self.angle())
  }

  pub fn to_inexact(&self) -> Self {
    Self {
      real: self.real.to_inexact(),
      imag: self.imag.to_inexact(),
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

  /// Computes the polar angle of this complex number, as an `f64`.
  /// The returned angle is in radians from `-pi` to `pi` (including
  /// the upper bound but not the lower).
  ///
  /// If `self.is_zero()`, then this returns zero.
  pub fn angle(&self) -> Radians<f64> {
    if self.is_zero() {
      Radians::zero()
    } else {
      let real = self.real.to_f64().unwrap_or(f64::NAN);
      let imag = self.imag.to_f64().unwrap_or(f64::NAN);
      Radians::atan2(imag, real)
    }
  }

  /// Returns the natural logarithm, with the imaginary part in the
  /// `(-pi, pi]` branch cut. Panics if `self.is_zero()`.
  pub fn ln(&self) -> ComplexNumber {
    let (magn, angle) = self.to_polar();
    ComplexNumber::new(magn.ln(), angle.0)
  }

  pub fn recip(&self) -> ComplexNumber {
    let abs_sqr = self.abs_sqr();
    ComplexNumber::new(
      &self.real / &abs_sqr,
      - &self.imag / abs_sqr,
    )
  }

  pub fn powi(&self, exp: BigInt) -> ComplexNumber {
    match exp.cmp(&BigInt::zero()) {
      Ordering::Equal => {
        ComplexNumber::one()
      }
      Ordering::Greater => {
        powi_by_repeated_square(self.clone(), exp)
      }
      Ordering::Less => {
        powi_by_repeated_square(self.recip(), -exp)
      }
    }
  }

  pub fn powf(&self, exp: f64) -> ComplexNumber {
    ComplexNumber::from_polar_inexact(self.abs().powf(exp), self.angle() * exp)
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

// Needed to call powi_by_repeated_square. We'll implement the other
// ops::*Assign traits on an as-needed basis.
impl ops::MulAssign for ComplexNumber {
  fn mul_assign(&mut self, other: ComplexNumber) {
    *self = self.clone() * other
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

impl ops::Neg for ComplexNumber {
  type Output = ComplexNumber;

  fn neg(self) -> ComplexNumber {
    ComplexNumber { real: - self.real, imag: - self.imag }
  }
}

impl ops::Neg for &ComplexNumber {
  type Output = ComplexNumber;

  fn neg(self) -> ComplexNumber {
    (*self).clone().neg()
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

  use approx::assert_abs_diff_eq;

  #[test]
  fn test_add() {
    assert_strict_eq!(
      ComplexNumber::new(1, 2) + ComplexNumber::new(3, 4),
      ComplexNumber::new(4, 6),
    );
    assert_strict_eq!(
      ComplexNumber::new(1.0, 2) + ComplexNumber::new(3, 4),
      ComplexNumber::new(4.0, 6)
    );
  }

  #[test]
  fn test_sub() {
    assert_strict_eq!(
      ComplexNumber::new(1, 10) - ComplexNumber::new(3, 4),
      ComplexNumber::new(-2, 6)
    );
    assert_strict_eq!(
      ComplexNumber::new(1.0, 2) - ComplexNumber::new(3, 5),
      ComplexNumber::new(-2.0, -3)
    );
  }

  #[test]
  fn test_mul() {
    assert_strict_eq!(
      ComplexNumber::new(1, 10) * ComplexNumber::new(2, 20),
      ComplexNumber::new(-198, 40),
    );
    assert_strict_eq!(
      ComplexNumber::new(1, 10.0) * ComplexNumber::new(2, 20),
      ComplexNumber::new(-198.0, 40.0),
    );
  }

  #[test]
  fn test_div() {
    assert_strict_eq!(
      ComplexNumber::new(1, 10) / ComplexNumber::new(2, 20),
      ComplexNumber::new(Number::ratio(1, 2), 0),
    );
    assert_strict_eq!(
      ComplexNumber::new(1.0, 10) / ComplexNumber::new(2, 20),
      ComplexNumber::new(0.5, 0.0),
    );
  }

  #[test]
  fn test_recip() {
    assert_strict_eq!(
      ComplexNumber::new(2, 10).recip(),
      ComplexNumber::new(Number::ratio(2, 104), Number::ratio(-10, 104)),
    );
  }

  #[test]
  fn test_powi_zero_exponent() {
    assert_strict_eq!(
      ComplexNumber::new(2, 3).powi(BigInt::zero()),
      ComplexNumber::new(1, 0),
    );
  }

  #[test]
  fn test_powi_positive_exponent() {
    assert_strict_eq!(
      ComplexNumber::new(2, 3).powi(BigInt::from(4)),
      ComplexNumber::new(-119, -120),
    );
  }

  #[test]
  fn test_powi_negative_exponent() {
    assert_strict_eq!(
      ComplexNumber::new(2, 3).powi(BigInt::from(-3)),
      ComplexNumber::new(Number::ratio(-46, 2197), Number::ratio(-9, 2197)),
    );
  }

  #[test]
  fn test_powf() {
    assert_abs_diff_eq!(
      ComplexNumber::new(2, 3).powf(0.0),
      ComplexNumber::new(1, 0),
      epsilon = 0.0001,
    );

    assert_abs_diff_eq!(
      ComplexNumber::new(2, 3).powf(4.0),
      ComplexNumber::new(-119, -120),
      epsilon = 0.0001,
    );

    assert_abs_diff_eq!(
      ComplexNumber::new(2, 3).powf(-3.0),
      ComplexNumber::new(Number::ratio(-46, 2197), Number::ratio(-9, 2197)),
      epsilon = 0.0001,
    );

    assert_abs_diff_eq!(
      ComplexNumber::new(2, 3).powf(1.8),
      ComplexNumber::new(-1.980959, 9.861875),
      epsilon = 0.0001,
    );
  }
}
