
use super::Expr;
use super::atom::Atom;
use super::number::{Number, ComplexLike};
use crate::util::prism::Prism;

/// Prism which downcasts an [`Expr`] to a contained [`Number`].
#[derive(Debug, Clone, Copy, Default)]
pub struct ExprToNumber;

/// Prism which downcasts an [`Expr`] to a [`ComplexLike`], either a
/// real or a complex number.
#[derive(Debug, Clone, Copy, Default)]
pub struct ExprToComplex;

impl ExprToNumber {
  pub fn new() -> Self {
    ExprToNumber
  }
}

impl ExprToComplex {
  pub fn new() -> Self {
    ExprToComplex
  }
}

impl Prism<Expr, Number> for ExprToNumber {
  fn narrow_type(&self, input: Expr) -> Result<Number, Expr> {
    Number::try_from(input).map_err(|err| err.original_expr)
  }

  fn widen_type(&self, input: Number) -> Expr {
    Expr::from(input)
  }
}

impl Prism<Expr, ComplexLike> for ExprToComplex {
  fn narrow_type(&self, input: Expr) -> Result<ComplexLike, Expr> {
    match input {
      Expr::Atom(Atom::Number(r)) => Ok(ComplexLike::Real(r)),
      Expr::Atom(Atom::Complex(z)) => Ok(ComplexLike::Complex(z)),
      _ => Err(input),
    }
  }

  fn widen_type(&self, input: ComplexLike) -> Expr {
    match input {
      ComplexLike::Real(r) => Expr::Atom(Atom::Number(r)),
      ComplexLike::Complex(z) => Expr::Atom(Atom::Complex(z)),
    }
  }
}
