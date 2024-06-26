
//! Power functions on real and complex numbers.
//!
//! See also [`Number::powi`](super::Number::powi) and
//! [`Number::powf`](super::Number::powf).

use super::real::{Number, NumberImpl};
use super::complex::ComplexNumber;
use super::ComplexLike;
use crate::util::angles::Radians;

use num::{BigInt, Zero, One, ToPrimitive};

use std::f64::consts::PI;

/// Raises one real number to another real number, producing the
/// principal value of `x^y`.
///
/// If `x < 0`, then the result may be a complex number.
pub fn pow_real(x: Number, y: Number) -> ComplexLike {
  match y.inner {
    NumberImpl::Integer(y) => {
      ComplexLike::Real(x.powi(*y))
    }
    NumberImpl::Ratio(y) => {
      let big_x = x.powi(y.numer().clone());
      root_real(big_x, y.denom().clone())
    }
    NumberImpl::Float(y) => {
      if x > Number::zero() {
        // Just do floating exponentiation.
        let result = x.powf(y);
        ComplexLike::Real(Number::from(result))
      } else {
        // Calculate the result in polar coordinates.
        let magnitude = x.abs().powf(y);
        let angle = Radians(PI * y);
        ComplexLike::Complex(ComplexNumber::from_polar_inexact(magnitude, angle))
      }
    }
  }
}

/// Finds the principal nth root of a real number. The result may be
/// a complex number.
///
/// Precondition: `n > 0`
pub fn root_real(x: Number, n: BigInt) -> ComplexLike {
  assert!(n > BigInt::zero());

  // Corner cases: x == 0 or n == 1.
  if x.is_zero() {
    return ComplexLike::Real(x);
  }
  if n == BigInt::one() {
    return ComplexLike::Real(x);
  }

  // TODO Currently, for nontrivial roots, we just delegate to inexact
  // computations as a matter of course. In principle, we should try
  // to stay exact in situatons where it's reasonable to do so.
  let n = n.to_f64().unwrap_or(f64::NAN);
  if x > Number::zero() {
    // x is positive, so the result will be a real, positive value.
    let pow = x.powf(n.recip());
    ComplexLike::Real(Number::from(pow))
  } else {
    // x is negative, so the result will be complex.
    let magnitude = x.abs().powf(n.recip());
    let angle = Radians(PI / n);
    ComplexLike::Complex(ComplexNumber::from_polar_inexact(magnitude, angle))
  }
}

/// Raises a complex number to a real numbered power, producing the
/// principal value of `x^y`.
pub fn pow_complex_to_real(x: ComplexNumber, y: Number) -> ComplexNumber {
  match y.inner {
    NumberImpl::Integer(y) => {
      x.powi(*y)
    }
    NumberImpl::Ratio(y) => {
      let big_x = x.powi(y.numer().clone());
      root_complex(big_x, y.denom().clone())
    }
    NumberImpl::Float(y) => {
      // Calculate the result in polar coordinates.
      let magnitude = x.abs().powf(y);
      let angle = x.angle() * y;
      ComplexNumber::from_polar_inexact(magnitude, angle)
    }
  }
}

fn raise_to_power_i(x: ComplexNumber) -> ComplexNumber {
  let abs = x.abs();
  let angle = x.angle();
  let new_abs = (- angle.0).exp();
  let new_angle = Radians(abs.ln());
  ComplexNumber::from_polar_inexact(new_abs, new_angle)
}

/// Raises an arbitrary complex number to another complex
/// numbered-power.
pub fn pow_complex(x: ComplexNumber, y: ComplexNumber) -> ComplexNumber {
  // Distribute y = a + b i, and do x^a and (x^i)^b separately.
  let first_term = pow_complex_to_real(x.clone(), y.real().clone());
  let second_term = pow_complex_to_real(raise_to_power_i(x), y.imag().clone());
  first_term * second_term
}

/// Finds the principal nth root of a complex number.
///
/// Precondition: `n > 0`
pub fn root_complex(x: ComplexNumber, n: BigInt) -> ComplexNumber {
  assert!(n > BigInt::zero());

  // Corner cases: x == 0 or n == 1.
  if x.is_zero() || n == BigInt::one() {
    return x;
  }

  // TODO Exactness here as well, just like root_real. It should be
  // possible in some situations.
  let n = n.to_f64().unwrap_or(f64::NAN);
  let magnitude = x.abs().powf(n.recip());
  let angle = x.angle() / n;
  ComplexNumber::from_polar_inexact(magnitude, angle)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::assert_strict_eq;

  use approx::assert_abs_diff_eq;

  #[test]
  #[should_panic]
  fn test_root_real_negative_power() {
    root_real(Number::from(2), BigInt::from(-1));
  }

  #[test]
  #[should_panic]
  fn test_root_real_zero_power() {
    root_real(Number::from(2), BigInt::zero());
  }

  #[test]
  #[should_panic]
  fn test_root_real_zero_base_and_zero_power() {
    root_real(Number::zero(), BigInt::zero());
  }

  #[test]
  fn test_root_real_with_zero_base() {
    assert_strict_eq!(
      root_real(Number::zero(), BigInt::from(3)),
      ComplexLike::Real(Number::zero()),
    );
    assert_strict_eq!(
      root_real(Number::zero(), BigInt::from(99)),
      ComplexLike::Real(Number::zero()),
    );
    assert_strict_eq!(
      root_real(Number::zero(), BigInt::from(1)),
      ComplexLike::Real(Number::zero()),
    );
  }

  #[test]
  fn test_root_real_with_exponent_one() {
    assert_strict_eq!(
      root_real(Number::from(9), BigInt::from(1)),
      ComplexLike::Real(Number::from(9)),
    );
    assert_strict_eq!(
      root_real(Number::from(-9), BigInt::from(1)),
      ComplexLike::Real(Number::from(-9)),
    );
    assert_strict_eq!(
      root_real(Number::from(-9.0), BigInt::from(1)),
      ComplexLike::Real(Number::from(-9.0)),
    );
  }

  #[test]
  fn test_root_real_with_exponent_two() {
    let value = root_real(Number::from(4), BigInt::from(2)).unwrap_real();
    assert_abs_diff_eq!(value, Number::from(2));

    let value = root_real(Number::from(99), BigInt::from(2)).unwrap_real();
    assert_abs_diff_eq!(value, Number::from(9.94987437107), epsilon = 0.0001);

    let value = root_real(Number::from(-99), BigInt::from(2)).unwrap_complex();
    assert_abs_diff_eq!(
      value,
      ComplexNumber::new(0, 9.94987437107),
      epsilon = 0.0001,
    );
  }

  #[test]
  fn test_root_real_with_exponent_three() {
    let value = root_real(Number::from(8), BigInt::from(3)).unwrap_real();
    assert_abs_diff_eq!(value, Number::from(2));

    let value = root_real(Number::from(5), BigInt::from(3)).unwrap_real();
    assert_abs_diff_eq!(value, Number::from(1.70997594), epsilon = 0.0001);

    let value = root_real(Number::from(-5), BigInt::from(3)).unwrap_complex();
    assert_abs_diff_eq!(
      value,
      ComplexNumber::new(0.85498797, 1.4808826),
      epsilon = 0.0001,
    );
  }

  #[test]
  fn test_pow_real_with_integer_exponent() {
    assert_strict_eq!(
      pow_real(Number::from(2), Number::from(2)),
      ComplexLike::Real(Number::from(4)),
    );
    assert_strict_eq!(
      pow_real(Number::from(3), Number::from(10)),
      ComplexLike::Real(Number::from(59049)),
    );
    assert_strict_eq!(
      pow_real(Number::ratio(2, 3), Number::from(2)),
      ComplexLike::Real(Number::ratio(4, 9)),
    );
    assert_strict_eq!(
      pow_real(Number::ratio(2, 3), Number::from(-2)),
      ComplexLike::Real(Number::ratio(9, 4)),
    );
    assert_strict_eq!(
      pow_real(Number::ratio(2, 3), Number::from(0)),
      ComplexLike::Real(Number::from(1)),
    );
  }

  #[test]
  fn test_pow_real_with_rational_exponent() {
    let value = pow_real(Number::from(2), Number::ratio(2, 3)).unwrap_real();
    assert_abs_diff_eq!(value, Number::from(1.5874010519), epsilon = 0.0001);

    let value = pow_real(Number::from(-2), Number::ratio(1, 3)).unwrap_complex();
    assert_abs_diff_eq!(
      value,
      ComplexNumber::new(0.6299605, 1.0911236),
      epsilon = 0.0001,
    );
  }

  #[test]
  fn test_pow_real_with_floating_exponent() {
    let value = pow_real(Number::from(2), Number::from(0.666666)).unwrap_real();
    assert_abs_diff_eq!(value, Number::from(1.5874010519), epsilon = 0.0001);

    let value = pow_real(Number::from(-2), Number::from(0.333333)).unwrap_complex();
    assert_abs_diff_eq!(
      value,
      ComplexNumber::new(0.6299605, 1.0911236),
      epsilon = 0.0001,
    );
  }

  #[test]
  fn test_pow_complex() {
    let value = pow_complex(
      ComplexNumber::new(0.3, 0.8),
      ComplexNumber::new(0.1, 0.2),
    );
    assert_abs_diff_eq!(
      value,
      ComplexNumber::new(0.7693787, 0.069223389),
      epsilon = 0.0001,
    );
  }
}
