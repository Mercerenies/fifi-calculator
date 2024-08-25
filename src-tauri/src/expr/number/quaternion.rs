
use super::{Number, ComplexNumber, powi_by_repeated_square};
use super::inexact::DivInexact;
use crate::util::stricteq::StrictEq;

use serde::{Serialize, Deserialize};
use num::{BigInt, Zero, One};
use approx::{AbsDiffEq, RelativeEq};

use std::fmt::{self, Display, Formatter};
use std::{ops, iter};
use std::cmp::Ordering;

/// A quaternion has four components: a real part and three non-real
/// parts associated with the constants `i`, `j`, and `k`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Quaternion {
  r: Number,
  i: Number,
  j: Number,
  k: Number,
}

impl Quaternion {
  pub const FUNCTION_NAME: &'static str = "quat";

  pub fn new(r: impl Into<Number>, i: impl Into<Number>, j: impl Into<Number>, k: impl Into<Number>) -> Self {
    Self {
      r: r.into(),
      i: i.into(),
      j: j.into(),
      k: k.into(),
    }
  }

  /// The imaginary constant `i`.
  pub fn ii() -> Self {
    Self::new(0, 1, 0, 0)
  }

  /// The imaginary constant `j`.
  pub fn jj() -> Self {
    Self::new(0, 0, 1, 0)
  }

  /// The imaginary constant `k`.
  pub fn kk() -> Self {
    Self::new(0, 0, 0, 1)
  }

  pub fn into_parts(self) -> (Number, Number, Number, Number) {
    (self.r, self.i, self.j, self.k)
  }

  pub fn from_real(real: impl Into<Number>) -> Self {
    Self::new(real, 0, 0, 0)
  }

  pub fn to_inexact(&self) -> Self {
    Self {
      r: self.r.to_inexact(),
      i: self.i.to_inexact(),
      j: self.j.to_inexact(),
      k: self.k.to_inexact(),
    }
  }

  /// The conjugate of `self`.
  pub fn conj(self) -> Self {
    Self::new(self.r, -self.i, -self.j, -self.k)
  }

  /// Computes the square of the absolute value
  pub fn abs_sqr(&self) -> Number {
    &self.r * &self.r + &self.i * &self.i + &self.j * &self.j + &self.k * &self.k
  }

  /// Computes the absolute value of this complex number. Note that,
  /// for now, this is always an inexact quantity.
  pub fn abs(&self) -> f64 {
    // TODO Do this exactly, when we can.
    self.abs_sqr().powf(0.5)
  }

  /// The multiplicative inverse of this quaternion. Panics if
  /// `self.is_zero()`.
  pub fn recip(self) -> Self {
    assert!(!self.is_zero(), "Attempted to take reciprocal of zero quaternion");
    let abs_sqr = self.abs_sqr();
    let (r, i, j, k) = self.conj().into_parts();
    Self::new(r / &abs_sqr, i / &abs_sqr, j / &abs_sqr, k / &abs_sqr)
  }

  /// The multiplicative inverse of this quaternion. Panics if
  /// `self.is_zero()`. Uses inexact division, so division of integers
  /// which does NOT produce an integer will fall back to
  /// floating-point rather than rational numbers.
  pub fn recip_inexact(self) -> Self {
    assert!(!self.is_zero(), "Attempted to take reciprocal of zero quaternion");
    let abs_sqr = self.abs_sqr();
    let (r, i, j, k) = self.conj().into_parts();
    Self::new(
      r.div_inexact(&abs_sqr),
      i.div_inexact(&abs_sqr),
      j.div_inexact(&abs_sqr),
      k.div_inexact(&abs_sqr),
    )
  }

  pub fn powi(&self, exp: BigInt) -> Quaternion {
    match exp.cmp(&BigInt::zero()) {
      Ordering::Equal => {
        Quaternion::one()
      }
      Ordering::Greater => {
        powi_by_repeated_square(self.clone(), exp)
      }
      Ordering::Less => {
        powi_by_repeated_square(self.clone().recip(), -exp)
      }
    }
  }

  /// Returns a normalized vector in the same direction as this
  /// quaternion. Returns zero if the input is zero.
  pub fn signum(&self) -> Quaternion {
    if self.is_zero() {
      Quaternion::zero()
    } else {
      // TODO: Revisit exactness once abs() can be exact.
      let magnitude = Quaternion::from_real(self.abs());
      self.clone() / magnitude
    }
  }

  /// True if any of the components of this quaternion are a proper
  /// (non-integer) ratio.
  pub fn has_proper_ratio(&self) -> bool {
    self.r.is_proper_ratio() || self.i.is_proper_ratio() || self.j.is_proper_ratio() || self.k.is_proper_ratio()
  }

}

impl StrictEq for Quaternion {
  fn strict_eq(&self, other: &Self) -> bool {
    self.r.strict_eq(&other.r)
      && self.i.strict_eq(&other.i)
      && self.j.strict_eq(&other.j)
      && self.k.strict_eq(&other.k)
  }
}

impl From<Number> for Quaternion {
  fn from(n: Number) -> Self {
    Self::from_real(n)
  }
}

impl From<ComplexNumber> for Quaternion {
  fn from(c: ComplexNumber) -> Self {
    let (real, imag) = c.into_parts();
    Self::new(real, imag, 0, 0)
  }
}

impl Display for Quaternion {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "({}, {}, {}, {})", self.r, self.i, self.j, self.k)
  }
}

impl ops::Add for Quaternion {
  type Output = Quaternion;

  fn add(self, other: Self) -> Self {
    Self::new(
      self.r + other.r,
      self.i + other.i,
      self.j + other.j,
      self.k + other.k,
    )
  }
}

impl ops::Sub for Quaternion {
  type Output = Quaternion;

  fn sub(self, other: Self) -> Self {
    Self::new(
      self.r - other.r,
      self.i - other.i,
      self.j - other.j,
      self.k - other.k,
    )
  }
}

impl ops::Sub for &Quaternion {
  type Output = Quaternion;

  fn sub(self, other: Self) -> Self::Output {
    (*self).clone() - (*other).clone()
  }
}

impl ops::Mul for Quaternion {
  type Output = Quaternion;

  fn mul(self, other: Self) -> Self {
    let (a1, b1, c1, d1) = self.into_parts();
    let (a2, b2, c2, d2) = other.into_parts();
    let a = &a1 * &a2 - &b1 * &b2 - &c1 * &c2 - &d1 * &d2;
    let b = &a1 * &b2 + &b1 * &a2 + &c1 * &d2 - &d1 * &c2;
    let c = &a1 * &c2 - &b1 * &d2 + &c1 * &a2 + &d1 * &b2;
    let d = &a1 * &d2 + &b1 * &c2 - &c1 * &b2 + &d1 * &a2;
    Self::new(a, b, c, d)
  }
}

// Needed to call powi_by_repeated_square. We'll implement the other
// ops::*Assign traits on an as-needed basis.
impl ops::MulAssign for Quaternion {
  fn mul_assign(&mut self, other: Self) {
    *self = self.clone() * other
  }
}

impl ops::Div for Quaternion {
  type Output = Quaternion;

  #[allow(clippy::suspicious_arithmetic_impl)] // Multiplication by reciprocal is correct
  fn div(self, other: Self) -> Self {
    self * other.recip()
  }
}

impl ops::Neg for Quaternion {
  type Output = Quaternion;

  fn neg(self) -> Self {
    Self::new(-self.r, -self.i, -self.j, -self.k)
  }
}
impl iter::Sum for Quaternion {
  fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
    iter.fold(Quaternion::zero(), |a, b| a + b)
  }
}

impl Zero for Quaternion {
  fn zero() -> Self {
    Self::new(0, 0, 0, 0)
  }

  fn is_zero(&self) -> bool {
    self.r.is_zero() && self.i.is_zero() && self.j.is_zero() && self.k.is_zero()
  }
}

impl One for Quaternion {
  fn one() -> Self {
    Self::new(1, 0, 0, 0)
  }

  fn is_one(&self) -> bool {
    self.r.is_one() && self.i.is_zero() && self.j.is_zero() && self.k.is_zero()
  }
}

impl AbsDiffEq for Quaternion {
  type Epsilon = f64;

  fn default_epsilon() -> f64 {
    <f64 as AbsDiffEq>::default_epsilon()
  }

  fn abs_diff_eq(&self, other: &Self, epsilon: f64) -> bool {
    self.abs() - other.abs() <= epsilon
  }
}

impl RelativeEq for Quaternion {
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
  fn test_basic_multiplication() {
    assert_eq!(Quaternion::ii() * Quaternion::ii(), Quaternion::from_real(-1));
    assert_eq!(Quaternion::jj() * Quaternion::jj(), Quaternion::from_real(-1));
    assert_eq!(Quaternion::kk() * Quaternion::kk(), Quaternion::from_real(-1));

    assert_eq!(Quaternion::ii() * Quaternion::jj(), Quaternion::kk());
    assert_eq!(Quaternion::jj() * Quaternion::kk(), Quaternion::ii());
    assert_eq!(Quaternion::kk() * Quaternion::ii(), Quaternion::jj());

    assert_eq!(Quaternion::jj() * Quaternion::ii(), - Quaternion::kk());
    assert_eq!(Quaternion::kk() * Quaternion::jj(), - Quaternion::ii());
    assert_eq!(Quaternion::ii() * Quaternion::kk(), - Quaternion::jj());
  }

  #[test]
  fn test_into_parts() {
    let quat = Quaternion::new(1, 2, 3, 4);
    let (r, i, j, k) = quat.into_parts();
    assert_strict_eq!(r, Number::from(1));
    assert_strict_eq!(i, Number::from(2));
    assert_strict_eq!(j, Number::from(3));
    assert_strict_eq!(k, Number::from(4));
  }

  #[test]
  fn test_to_inexact() {
    let quat = Quaternion::new(1, 2, 3, 4);
    let (r, i, j, k) = quat.to_inexact().into_parts();
    assert_strict_eq!(r, Number::from(1.0));
    assert_strict_eq!(i, Number::from(2.0));
    assert_strict_eq!(j, Number::from(3.0));
    assert_strict_eq!(k, Number::from(4.0));
  }

  #[test]
  fn test_conjugate() {
    let quat = Quaternion::new(1, 2, 3, 4);
    let conj = quat.conj();
    assert_strict_eq!(conj, Quaternion::new(1, -2, -3, -4));
  }

  #[test]
  fn test_abs_sqr() {
    let quat = Quaternion::new(1, 2, 3, 4);
    assert_eq!(quat.abs_sqr(), Number::from(30));
  }

  #[test]
  fn test_abs() {
    let quat = Quaternion::new(1, 2, 3, 4);
    assert_abs_diff_eq!(quat.abs(), 5.47722557505, epsilon = 0.0001);
  }

  #[test]
  fn test_recip() {
    let quat = Quaternion::new(1, 2, 3, 4);
    assert_strict_eq!(quat.recip(), Quaternion::new(
      Number::ratio(1, 30),
      Number::ratio(-1, 15),
      Number::ratio(-1, 10),
      Number::ratio(-2, 15),
    ));
  }

  #[test]
  fn test_powi() {
    let quat = Quaternion::new(1, 2, 3, 4);
    assert_eq!(quat.powi(BigInt::from(5)), Quaternion::new(3916, 1112, 1668, 2224));
    assert_eq!(quat.powi(BigInt::from(-1)), quat.clone().recip());
    assert_eq!(quat.powi(BigInt::from(0)), Quaternion::from_real(1));
  }

  #[test]
  fn test_signum_on_basis_elements() {
    assert_eq!(Quaternion::ii().signum(), Quaternion::ii());
    assert_eq!((- Quaternion::ii()).signum(), - Quaternion::ii());
    assert_eq!(Quaternion::jj().signum(), Quaternion::jj());
    assert_eq!((- Quaternion::jj()).signum(), - Quaternion::jj());
    assert_eq!(Quaternion::kk().signum(), Quaternion::kk());
    assert_eq!((- Quaternion::kk()).signum(), - Quaternion::kk());
  }

  #[test]
  fn test_signum_on_real() {
    assert_eq!(Quaternion::from_real(999).signum(), Quaternion::from_real(1));
    assert_eq!(Quaternion::from_real(-18).signum(), Quaternion::from_real(-1));
    assert_eq!(Quaternion::from_real(0).signum(), Quaternion::from_real(0));
  }

  #[test]
  fn test_signum() {
    assert_abs_diff_eq!(
      Quaternion::new(1, 2, 3, 4).signum(),
      Quaternion::new(
        1.0 / 5.47722557505,
        2.0 / 5.47722557505,
        3.0 / 5.47722557505,
        4.0 / 5.47722557505,
      ),
      epsilon = 0.0001,
    );
  }

  #[test]
  fn test_add() {
    assert_strict_eq!(
      Quaternion::new(1, 2, 3, 4) + Quaternion::new(5, 6, 7, 8),
      Quaternion::new(6, 8, 10, 12)
    );
  }

  #[test]
  fn test_sub() {
    assert_strict_eq!(
      Quaternion::new(1, 3, 5, 39) - Quaternion::new(10, 19, 29, 7),
      Quaternion::new(-9, -16, -24, 32)
    );
  }

  #[test]
  fn test_mul() {
    assert_strict_eq!(
      Quaternion::new(1, 2, 3, 4) * Quaternion::new(3, 10, 20, -4),
      Quaternion::new(-61, -76, 77, 18),
    );
  }
}
