
//! Provides helpers for broadcasting various operations across
//! vectors.

use super::{Vector, ExprToVector, LengthError};
use crate::expr::Expr;
use crate::expr::atom::Atom;
use crate::expr::number::{Number, ComplexNumber};
use crate::util;
use crate::util::prism::Prism;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExprToBroadcastable;

#[derive(Debug, Clone, PartialEq)]
enum BroadcastableImpl {
  RealScalar(Number),
  ComplexScalar(ComplexNumber),
  Vector(Vector),
}

impl Broadcastable {
  pub fn real_scalar(n: Number) -> Self {
    Broadcastable {
      data: BroadcastableImpl::RealScalar(n),
    }
  }

  pub fn complex_scalar(c: ComplexNumber) -> Self {
    Broadcastable {
      data: BroadcastableImpl::ComplexScalar(c),
    }
  }

  pub fn vector(v: Vector) -> Self {
    Broadcastable {
      data: BroadcastableImpl::Vector(v),
    }
  }

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

impl Prism<Expr, Broadcastable> for ExprToBroadcastable {
  fn narrow_type(&self, expr: Expr) -> Result<Broadcastable, Expr> {
    match expr {
      Expr::Atom(Atom::Number(n)) => Ok(Broadcastable::real_scalar(n)),
      Expr::Atom(Atom::Complex(c)) => Ok(Broadcastable::complex_scalar(c)),
      expr => {
        ExprToVector.narrow_type(expr).map(|v| Broadcastable::vector(v))
      }
    }
  }
  fn widen_type(&self, b: Broadcastable) -> Expr {
    match b.data {
      BroadcastableImpl::RealScalar(n) => Expr::Atom(Atom::Number(n)),
      BroadcastableImpl::ComplexScalar(c) => Expr::Atom(Atom::Complex(c)),
      BroadcastableImpl::Vector(v) => v.into_expr(),
    }
  }
}
