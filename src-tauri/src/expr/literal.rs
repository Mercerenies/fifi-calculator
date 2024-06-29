
use super::{Expr, TryFromExprError};
use super::number::{Number, ComplexNumber, ComplexLike};
use super::vector::Vector;
use super::prisms::{ExprToComplex, ExprToVector, expr_to_string};
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
  String(String),
  Vector(Vec<Literal>),
}

impl Literal {
  pub fn from_vec(v: Vec<Literal>) -> Self {
    Literal { data: LiteralImpl::Vector(v) }
  }

  fn try_from_as_complex(expr: Expr) -> Result<Self, Expr> {
    ExprToComplex.narrow_type(expr).map(|c| Literal { data: LiteralImpl::Numerical(c) })
  }

  fn try_from_as_string(expr: Expr) -> Result<Self, Expr> {
    expr_to_string().narrow_type(expr).map(|s| Literal { data: LiteralImpl::String(s) })
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

impl From<Literal> for Expr {
  fn from(lit: Literal) -> Self {
    match lit.data {
      LiteralImpl::String(s) => s.into(),
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
    }).or_else(|expr| {
      Literal::try_from_as_string(expr)
    }).map_err(|expr| {
      TryFromExprError::new("Literal", expr)
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

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
