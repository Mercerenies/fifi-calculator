
//! Provides helpers for broadcasting various operations across
//! vectors.

use super::{Vector, ExprToVector, LengthError};
use crate::expr::Expr;
use crate::expr::atom::Atom;
use crate::expr::number::{Number, ComplexNumber};
use crate::util;
use crate::util::prism::Prism;

use num::{Zero, One};

use std::ops::{Add, Sub, Mul, Div};

/// A `Broadcastable` is a value whose rank (as a tensor) is known.
/// Currently, our system only supports scalars (rank 0) and vectors
/// (rank 1), so a `Broadcastable` is either a value which is known to
/// be a scalar quantity (such as a real or complex number) or a value
/// which is literally a vector expression.
#[derive(Debug, Clone, PartialEq)]
pub struct Broadcastable {
  data: BroadcastableImpl,
}

/// Prism which attempts to read an [`Expr`] as a [`Broadcastable`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExprToBroadcastable;

#[derive(Debug, Clone, PartialEq)]
enum BroadcastableImpl {
  RealScalar(Number),
  ComplexScalar(ComplexNumber),
  Vector(Vector),
}

impl Broadcastable {
  /// Constructs a `Broadcastable` which is a literal real number.
  pub fn real_scalar(n: Number) -> Self {
    Broadcastable {
      data: BroadcastableImpl::RealScalar(n),
    }
  }

  /// Constructs a `Broadcastable` which is a literal complex number.
  pub fn complex_scalar(c: ComplexNumber) -> Self {
    Broadcastable {
      data: BroadcastableImpl::ComplexScalar(c),
    }
  }

  /// Constructs a `Broadcastable` which is a vector of arbitrary
  /// expressions.
  pub fn vector(v: Vector) -> Self {
    Broadcastable {
      data: BroadcastableImpl::Vector(v),
    }
  }

  /// The real value zero.
  pub fn zero() -> Self {
    Broadcastable::real_scalar(Number::zero())
  }

  /// The real value one.
  pub fn one() -> Self {
    Broadcastable::real_scalar(Number::one())
  }

  /// Returns the rank of the `Broadcastable`.
  ///
  /// The rank of a scalar is zero, and the rank of a vector is one.
  pub fn rank(&self) -> usize {
    match &self.data {
      BroadcastableImpl::RealScalar(_) => 0,
      BroadcastableImpl::ComplexScalar(_) => 0,
      BroadcastableImpl::Vector(_) => 1,
    }
  }

  /// If `self` is a scalar, produces a [`Vector`] which repeats
  /// `self` the specified number of times. If `self` is a vector,
  /// this function returns that vector if the length is correct, or
  /// an error if not.
  pub fn extend_to(self, len: usize) -> Result<Vector, LengthError> {
    match self.data {
      BroadcastableImpl::RealScalar(n) => Ok(util::repeated(Expr::from(n), len)),
      BroadcastableImpl::ComplexScalar(c) => Ok(util::repeated(Expr::from(c), len)),
      BroadcastableImpl::Vector(v) => {
        if v.len() == len {
          Ok(v)
        } else {
          Err(LengthError { expected: len, actual: v.len() })
        }
      }
    }
  }

  /// Checks whether arithmetic operations can be safely applied among
  /// the sequence of [`Broadcastable`] values. Specifically, this
  /// function succeeds if every `Broadcastable` which is a vector has
  /// the same length. This also means that an empty sequence, or a
  /// sequence consisting of only scalars, vacuously passes this test.
  ///
  /// If this check succeeds, then arithmetic can be performed on the
  /// vector. [`try_add`](Broadcasting::try_add),
  /// [`try_sub`](Broadcasting::try_sub), and company will always
  /// produce `Ok` values when reduced on the collection using
  /// `try_fold`. In this way, it's possible to validate, in advance,
  /// whether or not arithmetic will succeed, without partially
  /// consuming a collection in the case that it fails.
  pub fn check_compatible_lengths<'a, I>(values: I) -> Result<(), LengthError>
  where I: IntoIterator<Item = &'a Broadcastable> {
    let mut identified_length: Option<usize> = None;
    for value in values {
      if let BroadcastableImpl::Vector(v) = &value.data {
        match identified_length {
          None => {
            identified_length = Some(v.len());
          }
          Some(len) => {
            if len != v.len() {
              return Err(LengthError { expected: len, actual: v.len() });
            }
          }
        }
      }
    }
    Ok(())
  }

  /// Adds two `Broadcastable` values together. Produces an error if
  /// both are vectors and they have different lengths.
  pub fn try_add(self, other: Broadcastable) -> Result<Broadcastable, LengthError> {
    self.try_broadcasted_op(
      other,
      Number::add,
      ComplexNumber::add,
      "+",
    )
  }

  /// Subtracts two `Broadcastable` values together. Produces an error
  /// if both are vectors and they have different lengths.
  pub fn try_sub(self, other: Broadcastable) -> Result<Broadcastable, LengthError> {
    self.try_broadcasted_op(
      other,
      Number::sub,
      ComplexNumber::sub,
      "-",
    )
  }

  /// Multiplies two `Broadcastable` values together. Produces an
  /// error if both are vectors and they have different lengths.
  pub fn try_mul(self, other: Broadcastable) -> Result<Broadcastable, LengthError> {
    self.try_broadcasted_op(
      other,
      Number::mul,
      ComplexNumber::mul,
      "*",
    )
  }

  /// Divides two `Broadcastable` values together. Produces an error
  /// if both are vectors and they have different lengths.
  ///
  /// If both are scalars and the denominator is equal to zero, this
  /// function will panic, since it delegates to [`Number::div`] or
  /// [`ComplexNumber::div`] in that case.
  pub fn try_div(self, other: Broadcastable) -> Result<Broadcastable, LengthError> {
    self.try_broadcasted_op(
      other,
      Number::div,
      ComplexNumber::div,
      "/",
    )
  }

  fn try_broadcasted_op(
    self,
    other: Broadcastable,
    real_op: impl FnOnce(Number, Number) -> Number,
    complex_op: impl FnOnce(ComplexNumber, ComplexNumber) -> ComplexNumber,
    function_name: &str,
  ) -> Result<Broadcastable, LengthError> {
    use BroadcastableImpl::*;
    match (self.data, other.data) {
      (RealScalar(left), RealScalar(right)) => {
        Ok(Broadcastable::real_scalar(real_op(left, right)))
      }
      (RealScalar(left), ComplexScalar(right)) => {
        Ok(Broadcastable::complex_scalar(complex_op(ComplexNumber::from_real(left), right)))
      }
      (ComplexScalar(left), RealScalar(right)) => {
        Ok(Broadcastable::complex_scalar(complex_op(left, ComplexNumber::from_real(right))))
      }
      (ComplexScalar(left), ComplexScalar(right)) => {
        Ok(Broadcastable::complex_scalar(complex_op(left, right)))
      }
      (Vector(left), Vector(right)) => {
        left.zip_with(right, |a, b| Expr::call(function_name, vec![a, b]))
          .map(Broadcastable::vector)
      }
      (Vector(left), right) => {
        let right = Broadcastable { data: right };
        let right = right.extend_to(left.len()).expect("expected `right` to be a scalar");
        let res = left.zip_with(right, |a, b| Expr::call(function_name, vec![a, b])).expect("length must be correct");
        Ok(Broadcastable::vector(res))
      }
      (left, Vector(right)) => {
        let left = Broadcastable { data: left };
        let left = left.extend_to(right.len()).expect("expected `left` to be a scalar");
        let res = left.zip_with(right, |a, b| Expr::call(function_name, vec![a, b])).expect("length must be correct");
        Ok(Broadcastable::vector(res))
      }
    }
  }
}

impl Add for Broadcastable {
  type Output = Broadcastable;

  fn add(self, other: Broadcastable) -> Broadcastable {
    self.try_add(other).unwrap_or_else(|err| panic!("{err}"))
  }
}

impl Sub for Broadcastable {
  type Output = Broadcastable;

  fn sub(self, other: Broadcastable) -> Broadcastable {
    self.try_sub(other).unwrap_or_else(|err| panic!("{err}"))
  }
}

impl Mul for Broadcastable {
  type Output = Broadcastable;

  fn mul(self, other: Broadcastable) -> Broadcastable {
    self.try_mul(other).unwrap_or_else(|err| panic!("{err}"))
  }
}

impl Div for Broadcastable {
  type Output = Broadcastable;

  fn div(self, other: Broadcastable) -> Broadcastable {
    self.try_div(other).unwrap_or_else(|err| panic!("{err}"))
  }
}

impl From<Broadcastable> for Expr {
  fn from(b: Broadcastable) -> Expr {
    match b.data {
      BroadcastableImpl::RealScalar(n) => Expr::Atom(Atom::Number(n)),
      BroadcastableImpl::ComplexScalar(c) => Expr::Atom(Atom::Complex(c)),
      BroadcastableImpl::Vector(v) => v.into_expr(),
    }
  }
}

impl Prism<Expr, Broadcastable> for ExprToBroadcastable {
  fn narrow_type(&self, expr: Expr) -> Result<Broadcastable, Expr> {
    match expr {
      Expr::Atom(Atom::Number(n)) => Ok(Broadcastable::real_scalar(n)),
      Expr::Atom(Atom::Complex(c)) => Ok(Broadcastable::complex_scalar(c)),
      expr => {
        ExprToVector.narrow_type(expr).map(Broadcastable::vector)
      }
    }
  }
  fn widen_type(&self, b: Broadcastable) -> Expr {
    b.into()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use num::Zero;

  #[test]
  fn test_shape_of_broadcastable_constructors() {
    let real_scalar = Broadcastable::real_scalar(Number::from(99));
    assert_eq!(real_scalar.data, BroadcastableImpl::RealScalar(Number::from(99)));
    let complex_scalar = Broadcastable::complex_scalar(ComplexNumber::zero());
    assert_eq!(complex_scalar.data, BroadcastableImpl::ComplexScalar(ComplexNumber::zero()));
    let vector = Broadcastable::vector(Vector::default());
    assert_eq!(vector.data, BroadcastableImpl::Vector(Vector::default()));
  }

  #[test]
  fn test_rank() {
    let real_scalar = Broadcastable::real_scalar(Number::from(99));
    assert_eq!(real_scalar.rank(), 0);
    let complex_scalar = Broadcastable::complex_scalar(ComplexNumber::zero());
    assert_eq!(complex_scalar.rank(), 0);
    let vector = Broadcastable::vector(Vector::default());
    assert_eq!(vector.rank(), 1);
  }

  #[test]
  fn test_extend_to_on_real_scalar() {
    let x = Broadcastable::real_scalar(Number::from(99));
    assert_eq!(x.clone().extend_to(0), Ok(Vector::from(vec![])));
    assert_eq!(x.clone().extend_to(1), Ok(Vector::from(vec![Expr::from(99)])));
    assert_eq!(x.extend_to(2), Ok(Vector::from(vec![Expr::from(99), Expr::from(99)])));
  }

  #[test]
  fn test_extend_to_on_complex_scalar() {
    let z = ComplexNumber::new(Number::from(2), Number::from(1));
    let x = Broadcastable::complex_scalar(z.clone());
    assert_eq!(x.clone().extend_to(0), Ok(Vector::from(vec![])));
    assert_eq!(x.clone().extend_to(1), Ok(Vector::from(vec![Expr::from(z.clone())])));
    assert_eq!(x.extend_to(2), Ok(Vector::from(vec![Expr::from(z.clone()), Expr::from(z)])));
  }

  #[test]
  fn test_add_vec_to_scalar() {
    let x = Broadcastable::vector(Vector::from(vec![Expr::from(1), Expr::from(2), Expr::from(3)]));
    let y = Broadcastable::real_scalar(Number::from(99));
    assert_eq!(x.try_add(y).unwrap(), Broadcastable::vector(Vector::from(vec![
      Expr::call("+", vec![Expr::from(1), Expr::from(99)]),
      Expr::call("+", vec![Expr::from(2), Expr::from(99)]),
      Expr::call("+", vec![Expr::from(3), Expr::from(99)]),
    ])));
  }

  #[test]
  fn test_add_scalar_to_vec() {
    let x = Broadcastable::real_scalar(Number::from(99));
    let y = Broadcastable::vector(Vector::from(vec![Expr::from(1), Expr::from(2), Expr::from(3)]));
    assert_eq!(x.try_add(y).unwrap(), Broadcastable::vector(Vector::from(vec![
      Expr::call("+", vec![Expr::from(99), Expr::from(1)]),
      Expr::call("+", vec![Expr::from(99), Expr::from(2)]),
      Expr::call("+", vec![Expr::from(99), Expr::from(3)]),
    ])));
  }

  #[test]
  fn test_add_two_real_scalars() {
    let x = Broadcastable::real_scalar(Number::from(1));
    let y = Broadcastable::real_scalar(Number::from(2));
    assert_eq!(x.try_add(y).unwrap(), Broadcastable::real_scalar(Number::from(3)));
  }

  #[test]
  fn test_add_real_scalar_to_complex_scalar() {
    let x = Broadcastable::real_scalar(Number::from(1));
    let y = Broadcastable::complex_scalar(ComplexNumber::from_imag(Number::from(2)));
    assert_eq!(
      x.try_add(y).unwrap(),
      Broadcastable::complex_scalar(ComplexNumber::new(Number::from(1), Number::from(2))),
    );
  }

  #[test]
  fn test_add_matching_vecs() {
    let x = Broadcastable::vector(Vector::from(vec![Expr::from(1), Expr::from(2), Expr::from(3)]));
    let y = Broadcastable::vector(Vector::from(vec![Expr::from(9), Expr::from(8), Expr::from(7)]));
    assert_eq!(
      x.try_add(y).unwrap(),
      Broadcastable::vector(Vector::from(vec![
        Expr::call("+", vec![Expr::from(1), Expr::from(9)]),
        Expr::call("+", vec![Expr::from(2), Expr::from(8)]),
        Expr::call("+", vec![Expr::from(3), Expr::from(7)]),
      ])),
    );
  }

  #[test]
  fn test_add_nonmatching_vecs() {
    let x = Broadcastable::vector(Vector::from(vec![Expr::from(1), Expr::from(2), Expr::from(3)]));
    let y = Broadcastable::vector(Vector::from(vec![Expr::from(9), Expr::from(8), Expr::from(7), Expr::from(6)]));
    assert_eq!(
      x.try_add(y).unwrap_err(),
      LengthError { expected: 3, actual: 4 },
    );
  }

  #[test]
  fn test_widen_broadcastable_with_prism() {
    let real = Broadcastable::real_scalar(Number::from(1));
    assert_eq!(ExprToBroadcastable.widen_type(real), Expr::from(1));
    let complex = Broadcastable::complex_scalar(ComplexNumber::from_imag(Number::from(1)));
    assert_eq!(ExprToBroadcastable.widen_type(complex), Expr::from(ComplexNumber::from_imag(Number::from(1))));
    let vector = Broadcastable::vector(Vector::from(vec![Expr::from(1), Expr::from(2)]));
    assert_eq!(
      ExprToBroadcastable.widen_type(vector),
      Expr::call("vector", vec![Expr::from(1), Expr::from(2)]),
    );
  }

  #[test]
  fn test_narrow_prism_to_scalar() {
    let number = Expr::from(19);
    assert_eq!(ExprToBroadcastable.narrow_type(number), Ok(Broadcastable::real_scalar(Number::from(19))));
    let complex_number = Expr::from(ComplexNumber::from_imag(Number::from(19)));
    assert_eq!(
      ExprToBroadcastable.narrow_type(complex_number),
      Ok(Broadcastable::complex_scalar(ComplexNumber::from_imag(Number::from(19)))),
    );
  }

  #[test]
  fn test_narrow_prism_to_vector() {
    let vector = Expr::call("vector", vec![Expr::from(1), Expr::from(2)]);
    assert_eq!(
      ExprToBroadcastable.narrow_type(vector),
      Ok(Broadcastable::vector(Vector::from(vec![Expr::from(1), Expr::from(2)]))),
    );
  }

  #[test]
  fn test_narrow_prism_failure() {
    let not_broadcastable = Expr::call("foobar", vec![]);
    assert_eq!(
      ExprToBroadcastable.narrow_type(not_broadcastable.clone()),
      Err(not_broadcastable),
    );
  }
}
