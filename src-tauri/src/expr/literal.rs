
use super::{Expr, TryFromExprError};
use super::number::{Number, ComplexNumber, ComplexLike, Quaternion, QuaternionLike};
use super::algebra::infinity::{InfiniteConstant, UnboundedNumber};
use super::vector::Vector;
use super::incomplete::IncompleteObject;
use super::interval::RawInterval;
use super::prisms::{ExprToQuaternion, ExprToVector, ExprToInfinity,
                    expr_to_string, expr_to_incomplete_object, expr_to_unbounded_interval};
use crate::util::prism::Prism;

use std::convert::TryFrom;

/// The `Literal` type is the subset of the [`Expr`] type which
/// represents literal values. This subset is defined inductively.
///
/// * Real, complex, or quaternion number literals are literal values.
///
/// * Strings are literal values.
///
/// * Known infinity constants are literal values. (Note:
///   [`InfiniteConstant::NotANumber`] is equal to itself and is NOT
///   treated specially by this type)
///
/// * Intervals, where each bound is either a real number or a
///   *signed* infinity, are literal values.
///
/// * Incomplete objects are literal values.
///
/// * A vector (as defined by the [`Vector`] type) is a literal value
///   iff all of its elements are literals.
#[derive(Debug, Clone, PartialEq)]
pub struct Literal {
  data: LiteralImpl,
}

#[derive(Debug, Clone, PartialEq)]
enum LiteralImpl {
  Numerical(QuaternionLike),
  String(String),
  IncompleteObject(IncompleteObject),
  Interval(RawInterval<UnboundedNumber>),
  Infinity(InfiniteConstant),
  Vector(Vec<Literal>),
}

impl Literal {
  pub fn from_vec(v: Vec<Literal>) -> Self {
    Literal { data: LiteralImpl::Vector(v) }
  }

  fn try_from_as_quat(expr: Expr) -> Result<Self, Expr> {
    ExprToQuaternion.narrow_type(expr).map(|c| Literal { data: LiteralImpl::Numerical(c) })
  }

  fn try_from_as_string(expr: Expr) -> Result<Self, Expr> {
    expr_to_string().narrow_type(expr).map(|s| Literal { data: LiteralImpl::String(s) })
  }

  fn try_from_as_infinity(expr: Expr) -> Result<Self, Expr> {
    ExprToInfinity.narrow_type(expr).map(|c| Literal { data: LiteralImpl::Infinity(c) })
  }

  fn try_from_as_incomplete_object(expr: Expr) -> Result<Self, Expr> {
    expr_to_incomplete_object().narrow_type(expr).map(|c| Literal { data: LiteralImpl::IncompleteObject(c) })
  }

  fn try_from_as_interval(expr: Expr) -> Result<Self, Expr> {
    expr_to_unbounded_interval().narrow_type(expr).map(|c| Literal { data: LiteralImpl::Interval(c) })
  }

  fn try_from_as_vector(expr: Expr) -> Result<Self, Expr> {
    ExprToVector.narrow_type(expr).and_then(|v| {
      v.into_iter()
        .map(Literal::try_from)
        .collect::<Result<Vec<_>, _>>()
        .map(|v| Literal { data: LiteralImpl::Vector(v) })
        .map_err(|err| err.original_expr)
    })
  }
}

impl From<ComplexLike> for Literal {
  fn from(c: ComplexLike) -> Self {
    Literal { data: LiteralImpl::Numerical(c.into()) }
  }
}

impl From<QuaternionLike> for Literal {
  fn from(q: QuaternionLike) -> Self {
    Literal { data: LiteralImpl::Numerical(q) }
  }
}

impl From<String> for Literal {
  fn from(s: String) -> Self {
    Literal { data: LiteralImpl::String(s) }
  }
}

impl From<Number> for Literal {
  fn from(n: Number) -> Self {
    ComplexLike::Real(n).into()
  }
}

impl From<ComplexNumber> for Literal {
  fn from(c: ComplexNumber) -> Self {
    ComplexLike::Complex(c).into()
  }
}

impl From<Quaternion> for Literal {
  fn from(q: Quaternion) -> Self {
    QuaternionLike::Quaternion(q).into()
  }
}

impl From<InfiniteConstant> for Literal {
  fn from(c: InfiniteConstant) -> Self {
    Literal { data: LiteralImpl::Infinity(c) }
  }
}

impl From<IncompleteObject> for Literal {
  fn from(c: IncompleteObject) -> Self {
    Literal { data: LiteralImpl::IncompleteObject(c) }
  }
}

impl From<RawInterval<UnboundedNumber>> for Literal {
  fn from(c: RawInterval<UnboundedNumber>) -> Self {
    Literal { data: LiteralImpl::Interval(c) }
  }
}

impl<T: Into<Literal>> From<Vec<T>> for Literal {
  fn from(v: Vec<T>) -> Self {
    Literal {
      data: LiteralImpl::Vector(v.into_iter().map(|x| x.into()).collect()),
    }
  }
}

impl From<Literal> for Expr {
  fn from(lit: Literal) -> Self {
    match lit.data {
      LiteralImpl::String(s) => s.into(),
      LiteralImpl::Numerical(n) => n.into(),
      LiteralImpl::Infinity(inf) => inf.into(),
      LiteralImpl::IncompleteObject(inc) => inc.into(),
      LiteralImpl::Interval(i) => i.into(),
      LiteralImpl::Vector(v) => {
        let v: Vector = v.into_iter().map(Expr::from).collect();
        v.into()
      }
    }
  }
}

impl TryFrom<Expr> for Literal {
  type Error = TryFromExprError;

  fn try_from(expr: Expr) -> Result<Self, Self::Error> {
    Literal::try_from_as_quat(expr).or_else(|expr| {
      Literal::try_from_as_vector(expr)
    }).or_else(|expr| {
      Literal::try_from_as_string(expr)
    }).or_else(|expr| {
      Literal::try_from_as_infinity(expr)
    }).or_else(|expr| {
      Literal::try_from_as_incomplete_object(expr)
    }).or_else(|expr| {
      Literal::try_from_as_interval(expr)
    }).map_err(|expr| {
      TryFromExprError::new("Literal", expr)
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::expr::incomplete::ObjectType;
  use crate::expr::interval::{Interval, IntervalType};

  fn expect_roundtrip(literal: Literal) {
    let expr = Expr::from(literal.clone());
    let literal2 = Literal::try_from(expr).expect("Roundtrip through TryFrom and From failed");
    assert_eq!(literal2, literal);
  }

  #[test]
  fn test_roundtrip_number() {
    expect_roundtrip(Literal::from(Number::from(3)));
    expect_roundtrip(Literal::from(Number::from(-2.5)));
    expect_roundtrip(Literal::from(ComplexNumber::new(1, 1)));
    expect_roundtrip(Literal::from(ComplexNumber::new(-2, Number::ratio(1, 2))));
    expect_roundtrip(Literal::from(Quaternion::new(1, 2, 3, 4)));
    expect_roundtrip(Literal::from(InfiniteConstant::NegInfinity));
    expect_roundtrip(Literal::from(Interval::new(UnboundedNumber::finite(3), IntervalType::Closed, UnboundedNumber::POS_INFINITY).into_raw()));
    expect_roundtrip(Literal::from(IncompleteObject::new(ObjectType::LeftParen)));
  }

  #[test]
  fn test_roundtrip_vec() {
    expect_roundtrip(Literal::from_vec(vec![]));
    expect_roundtrip(Literal::from_vec(vec![
      Literal::from(Number::from(1)),
      Literal::from(Number::from(2)),
    ]));
  }

  #[test]
  fn test_roundtrip_nested_vec() {
    expect_roundtrip(Literal::from_vec(vec![
      Literal::from_vec(vec![
        Literal::from(Number::from(1)),
        Literal::from(Number::from(2)),
      ]),
      Literal::from_vec(vec![
        Literal::from_vec(vec![
          Literal::from(Number::from(3)),
        ]),
      ]),
      Literal::from(Number::from(4)),
    ]));

  }
}
