
mod complex;
mod visitor;
mod power;
pub mod prisms;
mod real;
mod repr;

pub use real::{Number, ParseNumberError};
pub use complex::ComplexNumber;
pub use repr::NumberRepr;
pub use power::{pow_real, pow_complex_to_real, pow_complex, root_real, root_complex};

use super::Expr;
use crate::util::stricteq::StrictEq;

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
/// for the [`ExprToComplex`](super::prisms::ExprToComplex) prism and
/// can be safely converted (via [`From::from`]) into a
/// [`ComplexNumber`] if desired.
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

  /// Panics if `self` is a [`ComplexLike::Complex`].
  pub fn unwrap_real(self) -> Number {
    match self {
      ComplexLike::Real(r) => r,
      ComplexLike::Complex(_) => panic!("Cannot unwrap a complex number as a real number"),
    }
  }

  /// Panics if `self` is a [`ComplexLike::Real`].
  pub fn unwrap_complex(self) -> ComplexNumber {
    match self {
      ComplexLike::Real(_) => panic!("Cannot unwrap a real number as a complex number"),
      ComplexLike::Complex(z) => z,
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

impl From<ComplexLike> for Expr {
  fn from(input: ComplexLike) -> Expr {
    match input {
      ComplexLike::Real(real) => real.into(),
      ComplexLike::Complex(complex) => complex.into(),
    }
  }
}

impl TryFrom<ComplexLike> for Number {
  type Error = ComplexLike;

  fn try_from(input: ComplexLike) -> Result<Number, ComplexLike> {
    match input {
      ComplexLike::Real(real) => Ok(real),
      ComplexLike::Complex(_) => Err(input),
    }
  }
}

impl PartialEq for ComplexLike {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (ComplexLike::Real(a), ComplexLike::Real(b)) => a == b,
      (ComplexLike::Complex(a), ComplexLike::Complex(b)) => a == b,
      (a, b) => {
        // This third case technically always works, but the first two
        // cases avoid an extra clone on `self` and `other`.
        ComplexNumber::from(a.to_owned()) == ComplexNumber::from(b.to_owned())
      }
    }
  }
}

impl StrictEq for ComplexLike {
  fn strict_eq(&self, other: &Self) -> bool {
    match (self, other) {
      (ComplexLike::Real(a), ComplexLike::Real(b)) => a.strict_eq(b),
      (ComplexLike::Complex(a), ComplexLike::Complex(b)) => a.strict_eq(b),
      _ => false
    }
  }
}
