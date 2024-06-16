
use super::{Expr, TryFromExprError};
use super::number::{Number, ComplexNumber, ComplexLike};
use super::vector::Vector;
use super::prisms::{ExprToComplex, ExprToVector};
use crate::util::prism::Prism;

use std::convert::TryFrom;

/// The `Literal` type is the subset of the [`Expr`] type which
/// represents literal values. This subset is defined inductively.
///
/// * Real or complex number literals are literal values.
///
/// * A vector (as defined by the [`Vector`] type) is a literal value
/// iff all of its elements are literals.
#[derive(Debug, Clone, PartialEq)]
pub struct Literal {
  data: LiteralImpl,
}

#[derive(Debug, Clone, PartialEq)]
enum LiteralImpl {
  Numerical(ComplexLike),
  Vector(Vec<Literal>),
}

/// A prism which parses an expression as a [`Literal`].
#[derive(Debug, Clone, Copy, Default)]
pub struct ExprToLiteral;

impl Literal {
  pub fn from_vec(v: Vec<Literal>) -> Self {
    Literal { data: LiteralImpl::Vector(v) }
  }

  fn try_from_as_complex(expr: Expr) -> Result<Self, Expr> {
    ExprToComplex.narrow_type(expr).map(|c| Literal { data: LiteralImpl::Numerical(c) })
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
    Literal { data: LiteralImpl::Numerical(c) }
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

impl From<Literal> for Expr {
  fn from(lit: Literal) -> Self {
    match lit.data {
      LiteralImpl::Numerical(n) => n.into(),
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
    Literal::try_from_as_complex(expr).or_else(|expr| {
      Literal::try_from_as_vector(expr)
    }).map_err(|expr| {
      TryFromExprError::new("Literal", expr)
    })
  }
}

impl Prism<Expr, Literal> for ExprToLiteral {
  fn narrow_type(&self, expr: Expr) -> Result<Literal, Expr> {
    Literal::try_from(expr).map_err(|err| err.original_expr)
  }

  fn widen_type(&self, lit: Literal) -> Expr {
    lit.into()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn expect_roundtrip_through_prism(literal: Literal) {
    let expr = ExprToLiteral.widen_type(literal.clone());
    let literal2 = ExprToLiteral.narrow_type(expr).expect("Roundtrip through prism failed");
    assert_eq!(literal2, literal);
  }

  #[test]
  fn test_roundtrip_number_through_prism() {
    expect_roundtrip_through_prism(Literal::from(Number::from(3)));
    expect_roundtrip_through_prism(Literal::from(Number::from(-2.5)));
    expect_roundtrip_through_prism(Literal::from(ComplexNumber::new(Number::from(1), Number::from(1))));
    expect_roundtrip_through_prism(Literal::from(ComplexNumber::new(Number::from(-2), Number::ratio(1, 2))));
  }

  #[test]
  fn test_roundtrip_vec_through_prism() {
    expect_roundtrip_through_prism(Literal::from_vec(vec![]));
    expect_roundtrip_through_prism(Literal::from_vec(vec![
      Literal::from(Number::from(1)),
      Literal::from(Number::from(2)),
    ]));
  }

  #[test]
  fn test_roundtrip_nested_vec_through_prism() {
    expect_roundtrip_through_prism(Literal::from_vec(vec![
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
