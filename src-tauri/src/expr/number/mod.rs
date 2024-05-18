
mod complex;
mod visitor;
mod real;
mod repr;

pub use real::{Number, ParseNumberError};
pub use complex::ComplexNumber;
pub use repr::NumberRepr;

use num::{BigInt, Zero, One};

use std::ops::MulAssign;

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
