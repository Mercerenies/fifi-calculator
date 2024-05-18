
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

/// Either a real number or a complex number. This is used as a target
/// for the [`ExprToComplex`] and can be safely converted (via
/// [`From::from`]) into a [`ComplexNumber`] if desired.
///
/// If we directly wrote a prism for narrowing `Expr` to
/// `ComplexNumber`, then that prism would fail to catch non-complex
/// `Number`s. And a prism which captures both `ComplexNumber` and
/// `Number` as `ComplexNumber` would not be lawful. So this enum
/// gives us the best of both worlds: We get the implicit upcast of a
/// real number into a `ComplexNumber` while still having a lawful
/// `ExprToComplex` prism.
#[derive(Clone, Debug)]
pub enum ComplexLike {
  Real(Number),
  Complex(ComplexNumber),
}

impl ComplexLike {
  pub fn is_zero(&self) -> bool {
    match self {
      ComplexLike::Real(r) => r.is_zero(),
      ComplexLike::Complex(z) => z.is_zero(),
    }
  }
}

impl From<ComplexLike> for ComplexNumber {
  fn from(input: ComplexLike) -> ComplexNumber {
    match input {
      ComplexLike::Real(real) => ComplexNumber::from_real(real),
      ComplexLike::Complex(complex) => complex,
    }
  }
}
