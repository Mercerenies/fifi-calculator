
use super::Expr;
use super::atom::Atom;
use super::number::{Number, ComplexNumber};
use crate::util::prism::Prism;

use num::Zero;

/// Prism which downcasts an [`Expr`] to a contained [`Number`].
#[derive(Debug, Clone, Copy, Default)]
pub struct ExprToNumber;

/// Prism which downcasts an [`Expr`] to a [`ComplexLike`], either a
/// real or a complex number.
#[derive(Debug, Clone, Copy, Default)]
pub struct ExprToComplex;

/// Either a real number or a complex number. This is used as a target
/// for the [`ExprToComplex`] and can be safely converted (via
/// [`From::from`]) into a [`ComplexNumber`] if desired.
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

impl ComplexLike {
  pub fn is_zero(&self) -> bool {
    match self {
      ComplexLike::Real(r) => r.is_zero(),
      ComplexLike::Complex(z) => z.is_zero(),
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
