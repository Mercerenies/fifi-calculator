
use super::Expr;
use super::var::Var;
use super::atom::Atom;
use super::number::{Number, ComplexLike};
use crate::util::prism::{Prism, Only, Composed};

use num::Zero;

// Re-export some useful expression-adjacent prisms.
pub use super::var::StringToVar;
pub use super::vector::ExprToVector;
pub use super::vector::tensor::ExprToTensor;
pub use super::number::prisms::{NumberToUsize, NumberToI64};
pub use super::literal::ExprToLiteral;
pub use super::algebra::formula::ExprToFormula;

/// Prism which downcasts an [`Expr`] to a contained [`Number`].
#[derive(Debug, Clone, Copy, Default)]
pub struct ExprToNumber;

/// Prism which downcasts an [`Expr`] to a [`ComplexLike`], either a
/// real or a complex number.
#[derive(Debug, Clone, Copy, Default)]
pub struct ExprToComplex;

/// Prism which only accepts expressions which are a [`Var`].
#[derive(Debug, Clone, Copy, Default)]
pub struct ExprToVar;

/// Prism which downcasts a [`Number`] to a [`PositiveNumber`]. Fails
/// on negative numbers.
#[derive(Debug, Clone, Copy, Default)]
pub struct NumberToPositiveNumber;

/// A real number which is guaranteed to be positive. This is the
/// result type of the [`ExprToPositiveNumber`] prism.
#[derive(Debug, Clone)]
pub struct PositiveNumber {
  data: Number,
}

/// Prism which accepts only a specific variable as the expression.
pub fn must_be_var(var: Var) -> Only<Expr> {
  let expr = Expr::Atom(Atom::Var(var));
  Only::new(expr)
}

/// Prism which accepts only positive real numbers.
pub fn expr_to_positive_number() -> Composed<ExprToNumber, NumberToPositiveNumber, Number> {
  Composed::new(ExprToNumber, NumberToPositiveNumber)
}

/// Prism which only accepts expressions containing [`Number`] values
/// representable by a `usize`.
pub fn expr_to_usize() -> Composed<ExprToNumber, NumberToUsize, Number> {
  Composed::new(ExprToNumber, NumberToUsize)
}

/// Prism which only accepts expressions containing [`Number`] values
/// representable by an `i64`.
pub fn expr_to_i64() -> Composed<ExprToNumber, NumberToI64, Number> {
  Composed::new(ExprToNumber, NumberToI64)
}

impl PositiveNumber {
  /// Creates a `PositiveNumber`, or returns the input number
  /// unmodified if the value is not positive.
  pub fn new(number: Number) -> Result<Self, Number> {
    if number > Number::zero() {
      Ok(PositiveNumber { data: number })
    } else {
      Err(number)
    }
  }
}

impl From<PositiveNumber> for Number {
  fn from(arg: PositiveNumber) -> Self {
    arg.data
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

impl Prism<Expr, Var> for ExprToVar {
  fn narrow_type(&self, input: Expr) -> Result<Var, Expr> {
    if let Expr::Atom(Atom::Var(var)) = input {
      Ok(var)
    } else {
      Err(input)
    }
  }
  fn widen_type(&self, input: Var) -> Expr {
    Expr::Atom(Atom::Var(input))
  }
}

impl Prism<Number, PositiveNumber> for NumberToPositiveNumber {
  fn narrow_type(&self, input: Number) -> Result<PositiveNumber, Number> {
    PositiveNumber::new(input)
  }
  fn widen_type(&self, input: PositiveNumber) -> Number {
    input.into()
  }
}
