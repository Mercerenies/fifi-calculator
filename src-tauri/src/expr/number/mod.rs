
mod complex;
mod grouped;
pub mod inexact;
mod power;
mod quaternion;
mod real;
mod repr;
mod visitor;
pub mod prisms;

pub use real::{Number, ParseNumberError};
pub use complex::ComplexNumber;
pub use quaternion::Quaternion;
pub use repr::NumberRepr;
pub use power::{pow_real, pow_complex_to_real, pow_complex, root_real, root_complex};
pub use grouped::{ComplexLike, QuaternionLike};

use crate::util::{unwrap_infallible, PreOne};

use num::{BigInt, Zero, One};
use try_traits::ops::TryMulAssign;

use std::ops::MulAssign;

// TODO: Consider using try_traits for some of our failable ops like Div.

fn powi_by_repeated_square_err_with_one<T>(mut input: T, mut exp: BigInt, one: T) -> Result<T, T::Error>
where T: TryMulAssign + Clone {
  assert!(exp >= BigInt::zero());
  if exp.is_zero() {
    return Ok(one);
  }
  let mut result = one;
  while exp > BigInt::one() {
    if exp.clone() % BigInt::from(2) == BigInt::zero() {
      input.try_mul_assign(input.clone())?;
      exp /= BigInt::from(2);
    } else {
      result.try_mul_assign(input.clone())?;
      exp -= BigInt::one();
    }
  }
  result.try_mul_assign(input)?;
  Ok(result)
}

/// Raises `input` to the power of `exp`, using the repeated squaring
/// technique.
///
/// `exp` must be a nonnegative integer, or else this function panics.
/// If `exp` is equal to zero, this function returns `T::one()`.
pub fn powi_by_repeated_square_err<T>(input: T, exp: BigInt) -> Result<T, T::Error>
where T: PreOne + TryMulAssign + Clone {
  powi_by_repeated_square_err_with_one(input, exp, T::pre_one())
}

pub fn powi_by_repeated_square<T>(input: T, exp: BigInt) -> T
where T: PreOne + MulAssign + Clone {
  let result = powi_by_repeated_square_err(input, exp);
  unwrap_infallible(result)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_powi_by_repeated_square() {
    assert_eq!(
      powi_by_repeated_square(BigInt::from(3), BigInt::from(10)),
      BigInt::from(59_049),
    );
  }

  #[test]
  fn test_powi_by_repeated_square_on_zero_exponent() {
    assert_eq!(powi_by_repeated_square(BigInt::from(1), BigInt::from(0)), BigInt::from(1));
    assert_eq!(powi_by_repeated_square(BigInt::from(-1), BigInt::from(0)), BigInt::from(1));
    assert_eq!(powi_by_repeated_square(BigInt::from(0), BigInt::from(0)), BigInt::from(1));
  }

  #[test]
  #[should_panic]
  fn test_powi_by_repeated_square_on_negative_exponent() {
    powi_by_repeated_square(BigInt::from(1), BigInt::from(-1));
  }
}
