
mod complex;
mod grouped;
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

use num::{BigInt, Zero, One};

use std::ops::MulAssign;

// TODO: Consider using try_traits for some of our failable ops like Div.

// Precondition: exp > 0.
fn powi_by_repeated_square<T>(mut input: T, mut exp: BigInt) -> T
where T: One + MulAssign + Clone {
  assert!(exp > BigInt::zero());
  let mut result = T::one();
  while exp > BigInt::one() {
    if exp.clone() % BigInt::from(2) == BigInt::zero() {
      input *= input.clone();
      exp /= BigInt::from(2);
    } else {
      result *= input.clone();
      exp -= BigInt::one();
    }
  }
  result *= input;
  result
}
