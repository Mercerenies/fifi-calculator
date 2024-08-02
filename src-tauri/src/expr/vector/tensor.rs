
//! Provides helpers for tensors of various ranks, including
//! broadcasting operations and lifting tensors to higher ranks.

use super::{Vector, ExprToVector, LengthError};
use crate::expr::Expr;
use crate::expr::number::{Number, ComplexNumber, ComplexLike};
use crate::expr::prisms::ExprToComplex;
use crate::util;
use crate::util::prism::{Prism, PrismExt};

use num::{Zero, One};
use either::Either;

use std::ops::{Add, Sub, Mul, Div};

/// A `Tensor` is an expression whose tensor rank is known at runtime.
/// Currently, this system only supports scalars (rank 0) and vectors
/// (rank 1), so a `Tensor` is either a value which is known to be a
/// scalar quantity (such as a real or complex number) or a value
/// which is literally a vector expression.
#[derive(Debug, Clone, PartialEq)]
pub struct Tensor {
  data: TensorImpl,
}

/// Prism which attempts to read an [`Expr`] as a [`Tensor`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExprToTensor;

#[derive(Debug, Clone, PartialEq)]
enum TensorImpl {
  RealScalar(Number),
  ComplexScalar(ComplexNumber),
  Vector(Vector),
}

impl Tensor {
  /// Constructs a `Tensor` which is a literal real number.
  pub fn real_scalar(n: Number) -> Self {
    Tensor {
      data: TensorImpl::RealScalar(n),
    }
  }

  /// Constructs a `Tensor` which is a literal complex number.
  pub fn complex_scalar(c: ComplexNumber) -> Self {
    Tensor {
      data: TensorImpl::ComplexScalar(c),
    }
  }

  /// Constructs a `Tensor` which is a vector of arbitrary
  /// expressions.
  pub fn vector(v: Vector) -> Self {
    Tensor {
      data: TensorImpl::Vector(v),
    }
  }

  /// The real value zero.
  pub fn zero() -> Self {
    Tensor::real_scalar(Number::zero())
  }

  /// The real value one.
  pub fn one() -> Self {
    Tensor::real_scalar(Number::one())
  }

  /// Returns the rank of the `Tensor`.
  ///
  /// The rank of a scalar is zero, and the rank of a vector is one.
  pub fn rank(&self) -> usize {
    match &self.data {
      TensorImpl::RealScalar(_) => 0,
      TensorImpl::ComplexScalar(_) => 0,
      TensorImpl::Vector(_) => 1,
    }
  }

  /// If `self` is a scalar, produces a [`Vector`] which repeats
  /// `self` the specified number of times. If `self` is a vector,
  /// this function returns that vector if the length is correct, or
  /// an error if not.
  pub fn extend_to(self, len: usize) -> Result<Vector, LengthError> {
    match self.data {
      TensorImpl::RealScalar(n) => Ok(util::repeated(Expr::from(n), len)),
      TensorImpl::ComplexScalar(c) => Ok(util::repeated(Expr::from(c), len)),
      TensorImpl::Vector(v) => {
        if v.len() == len {
          Ok(v)
        } else {
          Err(LengthError { expected: len, actual: v.len() })
        }
      }
    }
  }

  /// Returns the vector contained within `self`. If `self` is a
  /// scalar, it's wrapped in a one-element vector.
  pub fn into_vector(self) -> Vector {
    match self.data {
      TensorImpl::RealScalar(n) => Vector::from(vec![Expr::from(n)]),
      TensorImpl::ComplexScalar(c) => Vector::from(vec![Expr::from(c)]),
      TensorImpl::Vector(v) => v,
    }
  }

  /// Checks whether arithmetic operations can be safely applied among
  /// the sequence of [`Tensor`] values. Specifically, this
  /// function succeeds if every `Tensor` which is a vector has
  /// the same length. This also means that an empty sequence, or a
  /// sequence consisting of only scalars, vacuously passes this test.
  ///
  /// If this check succeeds, then arithmetic can be performed on the
  /// vector. [`try_add`](Tensor::try_add),
  /// [`try_sub`](Tensor::try_sub), and company will always produce
  /// `Ok` values when reduced on the collection using `try_fold`. In
  /// this way, it's possible to validate, in advance, whether or not
  /// arithmetic will succeed, without partially consuming a
  /// collection in the case that it fails.
  pub fn check_compatible_lengths<'a, I>(values: I) -> Result<(), LengthError>
  where I: IntoIterator<Item = &'a Tensor> {
    let mut identified_length: Option<usize> = None;
    for value in values {
      if let TensorImpl::Vector(v) = &value.data {
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

  /// Adds two `Tensor` values together. Produces an error if
  /// both are vectors and they have different lengths.
  pub fn try_add(self, other: Tensor) -> Result<Tensor, LengthError> {
    self.try_broadcasted_op(
      other,
      Number::add,
      ComplexNumber::add,
      "+",
    )
  }

  /// Subtracts two `Tensor` values together. Produces an error
  /// if both are vectors and they have different lengths.
  pub fn try_sub(self, other: Tensor) -> Result<Tensor, LengthError> {
    self.try_broadcasted_op(
      other,
      Number::sub,
      ComplexNumber::sub,
      "-",
    )
  }

  /// Multiplies two `Tensor` values together. Produces an
  /// error if both are vectors and they have different lengths.
  pub fn try_mul(self, other: Tensor) -> Result<Tensor, LengthError> {
    self.try_broadcasted_op(
      other,
      Number::mul,
      ComplexNumber::mul,
      "*",
    )
  }

  /// Divides two `Tensor` values together. Produces an error
  /// if both are vectors and they have different lengths.
  ///
  /// If both are scalars and the denominator is equal to zero, this
  /// function will panic, since it delegates to [`Number::div`] or
  /// [`ComplexNumber::div`] in that case.
  pub fn try_div(self, other: Tensor) -> Result<Tensor, LengthError> {
    self.try_broadcasted_op(
      other,
      Number::div,
      ComplexNumber::div,
      "/",
    )
  }

  fn try_broadcasted_op(
    self,
    other: Tensor,
    real_op: impl FnOnce(Number, Number) -> Number,
    complex_op: impl FnOnce(ComplexNumber, ComplexNumber) -> ComplexNumber,
    function_name: &str,
  ) -> Result<Tensor, LengthError> {
    use TensorImpl::*;
    match (self.data, other.data) {
      (RealScalar(left), RealScalar(right)) => {
        Ok(Tensor::real_scalar(real_op(left, right)))
      }
      (RealScalar(left), ComplexScalar(right)) => {
        Ok(Tensor::complex_scalar(complex_op(ComplexNumber::from_real(left), right)))
      }
      (ComplexScalar(left), RealScalar(right)) => {
        Ok(Tensor::complex_scalar(complex_op(left, ComplexNumber::from_real(right))))
      }
      (ComplexScalar(left), ComplexScalar(right)) => {
        Ok(Tensor::complex_scalar(complex_op(left, right)))
      }
      (Vector(left), Vector(right)) => {
        left.zip_with(right, |a, b| Expr::call(function_name, vec![a, b]))
          .map(Tensor::vector)
      }
      (Vector(left), right) => {
        let right = Tensor { data: right };
        let right = right.extend_to(left.len()).expect("expected `right` to be a scalar");
        let res = left.zip_with(right, |a, b| Expr::call(function_name, vec![a, b])).expect("length must be correct");
        Ok(Tensor::vector(res))
      }
      (left, Vector(right)) => {
        let left = Tensor { data: left };
        let left = left.extend_to(right.len()).expect("expected `left` to be a scalar");
        let res = left.zip_with(right, |a, b| Expr::call(function_name, vec![a, b])).expect("length must be correct");
        Ok(Tensor::vector(res))
      }
    }
  }
}

impl Add for Tensor {
  type Output = Tensor;

  fn add(self, other: Tensor) -> Tensor {
    self.try_add(other).unwrap_or_else(|err| panic!("{err}"))
  }
}

impl Sub for Tensor {
  type Output = Tensor;

  fn sub(self, other: Tensor) -> Tensor {
    self.try_sub(other).unwrap_or_else(|err| panic!("{err}"))
  }
}

impl Mul for Tensor {
  type Output = Tensor;

  fn mul(self, other: Tensor) -> Tensor {
    self.try_mul(other).unwrap_or_else(|err| panic!("{err}"))
  }
}

impl Div for Tensor {
  type Output = Tensor;

  fn div(self, other: Tensor) -> Tensor {
    self.try_div(other).unwrap_or_else(|err| panic!("{err}"))
  }
}

impl From<Tensor> for Expr {
  fn from(b: Tensor) -> Expr {
    match b.data {
      TensorImpl::RealScalar(n) => n.into(),
      TensorImpl::ComplexScalar(c) => c.into(),
      TensorImpl::Vector(v) => v.into_expr(),
    }
  }
}

impl From<ComplexLike> for Tensor {
  fn from(b: ComplexLike) -> Tensor {
    match b {
      ComplexLike::Real(n) => Tensor::real_scalar(n),
      ComplexLike::Complex(c) => Tensor::complex_scalar(c),
    }
  }
}

impl Prism<Expr, Tensor> for ExprToTensor {
  fn narrow_type(&self, expr: Expr) -> Result<Tensor, Expr> {
    let prism = ExprToComplex.or(ExprToVector);
    prism.narrow_type(expr).map(|either| match either {
      Either::Left(ComplexLike::Real(r)) => Tensor::real_scalar(r),
      Either::Left(ComplexLike::Complex(c)) => Tensor::complex_scalar(c),
      Either::Right(v) => Tensor::vector(v),
    })
  }
  fn widen_type(&self, b: Tensor) -> Expr {
    b.into()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use num::Zero;

  #[test]
  fn test_shape_of_broadcastable_constructors() {
    let real_scalar = Tensor::real_scalar(Number::from(99));
    assert_eq!(real_scalar.data, TensorImpl::RealScalar(Number::from(99)));
    let complex_scalar = Tensor::complex_scalar(ComplexNumber::zero());
    assert_eq!(complex_scalar.data, TensorImpl::ComplexScalar(ComplexNumber::zero()));
    let vector = Tensor::vector(Vector::default());
    assert_eq!(vector.data, TensorImpl::Vector(Vector::default()));
  }

  #[test]
  fn test_rank() {
    let real_scalar = Tensor::real_scalar(Number::from(99));
    assert_eq!(real_scalar.rank(), 0);
    let complex_scalar = Tensor::complex_scalar(ComplexNumber::zero());
    assert_eq!(complex_scalar.rank(), 0);
    let vector = Tensor::vector(Vector::default());
    assert_eq!(vector.rank(), 1);
  }

  #[test]
  fn test_extend_to_on_real_scalar() {
    let x = Tensor::real_scalar(Number::from(99));
    assert_eq!(x.clone().extend_to(0), Ok(Vector::from(vec![])));
    assert_eq!(x.clone().extend_to(1), Ok(Vector::from(vec![Expr::from(99)])));
    assert_eq!(x.extend_to(2), Ok(Vector::from(vec![Expr::from(99), Expr::from(99)])));
  }

  #[test]
  fn test_extend_to_on_complex_scalar() {
    let z = ComplexNumber::new(2, 1);
    let x = Tensor::complex_scalar(z.clone());
    assert_eq!(x.clone().extend_to(0), Ok(Vector::from(vec![])));
    assert_eq!(x.clone().extend_to(1), Ok(Vector::from(vec![Expr::from(z.clone())])));
    assert_eq!(x.extend_to(2), Ok(Vector::from(vec![Expr::from(z.clone()), Expr::from(z)])));
  }

  #[test]
  fn test_add_vec_to_scalar() {
    let x = Tensor::vector(Vector::from(vec![Expr::from(1), Expr::from(2), Expr::from(3)]));
    let y = Tensor::real_scalar(Number::from(99));
    assert_eq!(x.try_add(y).unwrap(), Tensor::vector(Vector::from(vec![
      Expr::call("+", vec![Expr::from(1), Expr::from(99)]),
      Expr::call("+", vec![Expr::from(2), Expr::from(99)]),
      Expr::call("+", vec![Expr::from(3), Expr::from(99)]),
    ])));
  }

  #[test]
  fn test_add_scalar_to_vec() {
    let x = Tensor::real_scalar(Number::from(99));
    let y = Tensor::vector(Vector::from(vec![Expr::from(1), Expr::from(2), Expr::from(3)]));
    assert_eq!(x.try_add(y).unwrap(), Tensor::vector(Vector::from(vec![
      Expr::call("+", vec![Expr::from(99), Expr::from(1)]),
      Expr::call("+", vec![Expr::from(99), Expr::from(2)]),
      Expr::call("+", vec![Expr::from(99), Expr::from(3)]),
    ])));
  }

  #[test]
  fn test_add_two_real_scalars() {
    let x = Tensor::real_scalar(Number::from(1));
    let y = Tensor::real_scalar(Number::from(2));
    assert_eq!(x.try_add(y).unwrap(), Tensor::real_scalar(Number::from(3)));
  }

  #[test]
  fn test_add_real_scalar_to_complex_scalar() {
    let x = Tensor::real_scalar(Number::from(1));
    let y = Tensor::complex_scalar(ComplexNumber::from_imag(Number::from(2)));
    assert_eq!(
      x.try_add(y).unwrap(),
      Tensor::complex_scalar(ComplexNumber::new(1, 2)),
    );
  }

  #[test]
  fn test_add_matching_vecs() {
    let x = Tensor::vector(Vector::from(vec![Expr::from(1), Expr::from(2), Expr::from(3)]));
    let y = Tensor::vector(Vector::from(vec![Expr::from(9), Expr::from(8), Expr::from(7)]));
    assert_eq!(
      x.try_add(y).unwrap(),
      Tensor::vector(Vector::from(vec![
        Expr::call("+", vec![Expr::from(1), Expr::from(9)]),
        Expr::call("+", vec![Expr::from(2), Expr::from(8)]),
        Expr::call("+", vec![Expr::from(3), Expr::from(7)]),
      ])),
    );
  }

  #[test]
  fn test_add_nonmatching_vecs() {
    let x = Tensor::vector(Vector::from(vec![Expr::from(1), Expr::from(2), Expr::from(3)]));
    let y = Tensor::vector(Vector::from(vec![Expr::from(9), Expr::from(8), Expr::from(7), Expr::from(6)]));
    assert_eq!(
      x.try_add(y).unwrap_err(),
      LengthError { expected: 3, actual: 4 },
    );
  }

  #[test]
  fn test_widen_broadcastable_with_prism() {
    let real = Tensor::real_scalar(Number::from(1));
    assert_eq!(ExprToTensor.widen_type(real), Expr::from(1));
    let complex = Tensor::complex_scalar(ComplexNumber::from_imag(Number::from(1)));
    assert_eq!(ExprToTensor.widen_type(complex), Expr::from(ComplexNumber::from_imag(Number::from(1))));
    let vector = Tensor::vector(Vector::from(vec![Expr::from(1), Expr::from(2)]));
    assert_eq!(
      ExprToTensor.widen_type(vector),
      Expr::call("vector", vec![Expr::from(1), Expr::from(2)]),
    );
  }

  #[test]
  fn test_narrow_prism_to_scalar() {
    let number = Expr::from(19);
    assert_eq!(ExprToTensor.narrow_type(number), Ok(Tensor::real_scalar(Number::from(19))));
    let complex_number = Expr::from(ComplexNumber::from_imag(Number::from(19)));
    assert_eq!(
      ExprToTensor.narrow_type(complex_number),
      Ok(Tensor::complex_scalar(ComplexNumber::from_imag(Number::from(19)))),
    );
  }

  #[test]
  fn test_narrow_prism_to_vector() {
    let vector = Expr::call("vector", vec![Expr::from(1), Expr::from(2)]);
    assert_eq!(
      ExprToTensor.narrow_type(vector),
      Ok(Tensor::vector(Vector::from(vec![Expr::from(1), Expr::from(2)]))),
    );
  }

  #[test]
  fn test_narrow_prism_failure() {
    let not_broadcastable = Expr::call("foobar", vec![]);
    assert_eq!(
      ExprToTensor.narrow_type(not_broadcastable.clone()),
      Err(not_broadcastable),
    );
  }
}
