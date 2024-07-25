
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

use std::ops::{Add, Sub, Neg, Mul, Div, MulAssign};

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

  pub fn is_real(&self) -> bool {
    match self {
      ComplexLike::Real(_) => true,
      ComplexLike::Complex(_) => false,
    }
  }

  pub fn to_inexact(&self) -> Self {
    match self {
      ComplexLike::Real(r) => ComplexLike::Real(r.to_inexact()),
      ComplexLike::Complex(z) => ComplexLike::Complex(z.to_inexact()),
    }
  }

  pub fn abs(&self) -> Number {
    match self {
      ComplexLike::Real(r) => r.abs(),
      ComplexLike::Complex(z) => Number::from(z.abs()),
    }
  }

  pub fn abs_sqr(&self) -> Number {
    match self {
      ComplexLike::Real(r) => r.abs() * r.abs(),
      ComplexLike::Complex(z) => z.abs_sqr(),
    }
  }

  pub fn recip(&self) -> ComplexLike {
    match self {
      ComplexLike::Real(r) => ComplexLike::Real(r.recip()),
      ComplexLike::Complex(z) => ComplexLike::Complex(z.recip()),
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

impl From<i64> for ComplexLike {
  fn from(input: i64) -> ComplexLike {
    ComplexLike::Real(Number::from(input))
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

impl Add for ComplexLike {
  type Output = ComplexLike;

  fn add(self, other: Self) -> Self::Output {
    match (self, other) {
      (ComplexLike::Real(a), ComplexLike::Real(b)) => ComplexLike::Real(a + b),
      (a, b) => ComplexLike::Complex(ComplexNumber::from(a) + ComplexNumber::from(b)),
    }
  }
}

impl Sub for ComplexLike {
  type Output = ComplexLike;

  fn sub(self, other: Self) -> Self::Output {
    match (self, other) {
      (ComplexLike::Real(a), ComplexLike::Real(b)) => ComplexLike::Real(a - b),
      (a, b) => ComplexLike::Complex(ComplexNumber::from(a) - ComplexNumber::from(b)),
    }
  }
}

impl Neg for ComplexLike {
  type Output = ComplexLike;

  fn neg(self) -> Self::Output {
    match self {
      ComplexLike::Real(a) => ComplexLike::Real(-a),
      ComplexLike::Complex(a) => ComplexLike::Complex(-a),
    }
  }
}

impl Mul for ComplexLike {
  type Output = ComplexLike;

  fn mul(self, other: Self) -> Self::Output {
    match (self, other) {
      (ComplexLike::Real(a), ComplexLike::Real(b)) => ComplexLike::Real(a * b),
      (a, b) => ComplexLike::Complex(ComplexNumber::from(a) * ComplexNumber::from(b)),
    }
  }
}

impl MulAssign for ComplexLike {
  fn mul_assign(&mut self, other: Self) {
    *self = self.clone() * other
  }
}

impl Div for ComplexLike {
  type Output = ComplexLike;

  fn div(self, other: Self) -> Self::Output {
    match (self, other) {
      (ComplexLike::Real(a), ComplexLike::Real(b)) => ComplexLike::Real(a / b),
      (a, b) => ComplexLike::Complex(ComplexNumber::from(a) / ComplexNumber::from(b)),
    }
  }
}

impl Zero for ComplexLike {
  fn zero() -> Self {
    ComplexLike::Real(Number::zero())
  }

  fn is_zero(&self) -> bool {
    match self {
      ComplexLike::Real(r) => r.is_zero(),
      ComplexLike::Complex(z) => z.is_zero(),
    }
  }
}

impl One for ComplexLike {
  fn one() -> Self {
    ComplexLike::Real(Number::one())
  }

  fn is_one(&self) -> bool {
    dbg!(self);
    match self {
      ComplexLike::Real(r) => r.is_one(),
      ComplexLike::Complex(z) => z.is_one(),
    }
  }
}
