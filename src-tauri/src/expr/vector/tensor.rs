
//! Provides helpers for tensors of various ranks, including
//! broadcasting operations and lifting tensors to higher ranks.

use super::{Vector, ExprToVector, LengthError};
use crate::expr::Expr;
use crate::expr::number::{Number, ComplexNumber, Quaternion, ComplexLike, QuaternionLike};
use crate::expr::prisms::ExprToQuaternion;
use crate::util;
use crate::util::prism::{Prism, PrismExt};

use num::{Zero, One};
use either::Either;
use try_traits::ops::{TryAdd, TrySub, TryMul, TryDiv};

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
  Scalar(QuaternionLike),
  Vector(Vector),
}

impl Tensor {
  /// Constructs a `Tensor` which is a literal real number.
  pub fn real_scalar(n: Number) -> Self {
    Tensor {
      data: TensorImpl::Scalar(QuaternionLike::Real(n)),
    }
  }

  /// Constructs a `Tensor` which is a literal complex number.
  pub fn complex_scalar(c: ComplexNumber) -> Self {
    Tensor {
      data: TensorImpl::Scalar(QuaternionLike::Complex(c)),
    }
  }

  /// Constructs a `Tensor` which is a literal quaternion.
  pub fn quaternion_scalar(q: Quaternion) -> Self {
    Tensor {
      data: TensorImpl::Scalar(QuaternionLike::Quaternion(q)),
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
      TensorImpl::Scalar(_) => 0,
      TensorImpl::Vector(_) => 1,
    }
  }

  /// If `self` is a scalar, produces a [`Vector`] which repeats
  /// `self` the specified number of times. If `self` is a vector,
  /// this function returns that vector if the length is correct, or
  /// an error if not.
  pub fn extend_to(self, len: usize) -> Result<Vector, LengthError> {
    match self.data {
      TensorImpl::Scalar(n) => Ok(util::repeated(Expr::from(n), len)),
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
      TensorImpl::Scalar(n) => Vector::from(vec![Expr::from(n)]),
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

  fn try_broadcasted_op(
    self,
    other: Tensor,
    scalar_op: impl FnOnce(QuaternionLike, QuaternionLike) -> QuaternionLike,
    function_name: &str,
  ) -> Result<Tensor, LengthError> {
    use TensorImpl::*;
    match (self.data, other.data) {
      (Scalar(left), Scalar(right)) => {
        Ok(Tensor::from(scalar_op(left, right)))
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

impl TryAdd for Tensor {
  type Output = Tensor;
  type Error = LengthError;

  /// Adds two `Tensor` values together. Produces an error if
  /// both are vectors and they have different lengths.
  fn try_add(self, other: Tensor) -> Result<Tensor, LengthError> {
    self.try_broadcasted_op(
      other,
      QuaternionLike::add,
      "+",
    )
  }
}

impl TrySub for Tensor {
  type Output = Tensor;
  type Error = LengthError;

  /// Subtracts two `Tensor` values together. Produces an error
  /// if both are vectors and they have different lengths.
  fn try_sub(self, other: Tensor) -> Result<Tensor, LengthError> {
    self.try_broadcasted_op(
      other,
      QuaternionLike::sub,
      "-",
    )
  }
}

impl TryMul for Tensor {
  type Output = Tensor;
  type Error = LengthError;

  /// Multiplies two `Tensor` values together. Produces an
  /// error if both are vectors and they have different lengths.
  fn try_mul(self, other: Tensor) -> Result<Tensor, LengthError> {
    self.try_broadcasted_op(
      other,
      QuaternionLike::mul,
      "*",
    )
  }
}

impl TryDiv for Tensor {
  type Output = Tensor;
  type Error = LengthError;

  /// Divides two `Tensor` values together. Produces an error
  /// if both are vectors and they have different lengths.
  ///
  /// If both are scalars and the denominator is equal to zero, this
  /// function will panic, since it delegates to [`Number::div`],
  /// [`ComplexNumber::div`], or [`Quaternion::div`] in that case.
  fn try_div(self, other: Tensor) -> Result<Tensor, LengthError> {
    self.try_broadcasted_op(
      other,
      QuaternionLike::div,
      "/",
    )
  }
}

impl Mul<QuaternionLike> for Tensor {
  type Output = Tensor;

  fn mul(self, other: QuaternionLike) -> Tensor {
    // unwrap: At least one of the arguments is a scalar.
    self.try_mul(Tensor::from(other)).unwrap()
  }
}

impl Mul<Tensor> for QuaternionLike {
  type Output = Tensor;

  fn mul(self, other: Tensor) -> Tensor {
    // unwrap: At least one of the arguments is a scalar.
    Tensor::from(self).try_mul(other).unwrap()
  }
}

impl From<Tensor> for Expr {
  fn from(b: Tensor) -> Expr {
    match b.data {
      TensorImpl::Scalar(n) => n.into(),
      TensorImpl::Vector(v) => v.into_expr(),
    }
  }
}

impl From<ComplexLike> for Tensor {
  fn from(b: ComplexLike) -> Tensor {
    Tensor::from(QuaternionLike::from(b))
  }
}

impl From<QuaternionLike> for Tensor {
  fn from(b: QuaternionLike) -> Tensor {
    Tensor { data: TensorImpl::Scalar(b) }
  }
}

impl Prism<Expr, Tensor> for ExprToTensor {
  fn narrow_type(&self, expr: Expr) -> Result<Tensor, Expr> {
    let prism = ExprToQuaternion.or(ExprToVector);
    prism.narrow_type(expr).map(|either| match either {
      Either::Left(q) => Tensor::from(q),
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
    assert_eq!(real_scalar.data, TensorImpl::Scalar(QuaternionLike::Real(Number::from(99))));
    let complex_scalar = Tensor::complex_scalar(ComplexNumber::zero());
    assert_eq!(complex_scalar.data, TensorImpl::Scalar(QuaternionLike::Complex(ComplexNumber::zero())));
    let quat_scalar = Tensor::quaternion_scalar(Quaternion::zero());
    assert_eq!(quat_scalar.data, TensorImpl::Scalar(QuaternionLike::Quaternion(Quaternion::zero())));
    let vector = Tensor::vector(Vector::default());
    assert_eq!(vector.data, TensorImpl::Vector(Vector::default()));
  }

  #[test]
  fn test_rank() {
    let real_scalar = Tensor::real_scalar(Number::from(99));
    assert_eq!(real_scalar.rank(), 0);
    let complex_scalar = Tensor::complex_scalar(ComplexNumber::zero());
    assert_eq!(complex_scalar.rank(), 0);
    let quat_scalar = Tensor::quaternion_scalar(Quaternion::zero());
    assert_eq!(quat_scalar.rank(), 0);
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
  fn test_extend_to_on_quat_scalar() {
    let q = Quaternion::new(2, 1, 7, 8);
    let x = Tensor::quaternion_scalar(q.clone());
    assert_eq!(x.clone().extend_to(0), Ok(Vector::from(vec![])));
    assert_eq!(x.clone().extend_to(1), Ok(Vector::from(vec![Expr::from(q.clone())])));
    assert_eq!(x.extend_to(2), Ok(Vector::from(vec![Expr::from(q.clone()), Expr::from(q)])));
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
