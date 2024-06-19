
use super::Expr;
use super::var::Var;
use super::atom::Atom;
use super::number::{Number, ComplexLike};
use super::interval::{Interval, IntervalAny, IntervalOrNumber};
use super::literal::Literal;
use super::algebra::formula::{Formula, Equation};
use crate::util::prism::{Prism, Only, Composed, Conversion, ErrorWithPayload};

use num::Zero;

// Re-export some useful expression-adjacent prisms.
pub use super::var::StringToVar;
pub use super::vector::ExprToVector;
pub use super::vector::tensor::ExprToTensor;
pub use super::number::prisms::{NumberToUsize, NumberToI64};

/// Prism which downcasts an [`Expr`] to a contained [`Number`].
#[derive(Debug, Clone, Copy, Default)]
pub struct ExprToNumber; // TODO: This is just a Conversion prism :)

/// Prism which downcasts an [`Expr`] to a [`ComplexLike`], either a
/// real or a complex number.
#[derive(Debug, Clone, Copy, Default)]
pub struct ExprToComplex;

/// Prism which accepts either [`Interval`] values or real numbers.
#[derive(Debug, Clone, Copy, Default)]
pub struct ExprToIntervalLike;

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

/// Prism which reads a string as a non-negative integer.
#[derive(Debug, Clone)]
pub struct StringToUsize;

/// Equivalent to `usize` but also keeps track of the string used to
/// construct it. This ensures that the [`StringToUsize`] prism is
/// lawful and can recover the original string on `widen_type`.
pub struct ParsedUsize {
  value: usize,
  input: String,
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

/// Prism which accepts [`Literal`] values.
pub fn expr_to_literal() -> Conversion<Expr, Literal> {
  Conversion::new()
}

/// Prism which accepts [`Formula`] values.
pub fn expr_to_formula() -> Conversion<Expr, Formula> {
  Conversion::new()
}

/// Prism which accepts specifically [`Equation`] values.
pub fn expr_to_equation() -> Composed<Conversion<Expr, Formula>, Conversion<Formula, Equation>, Formula> {
  Composed::new(expr_to_formula(), Conversion::new())
}

pub fn expr_to_any_interval() -> Conversion<Expr, IntervalAny> {
  Conversion::new()
}

pub fn expr_to_interval() -> Conversion<Expr, Interval> {
  Conversion::new()
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

impl From<ParsedUsize> for usize {
  fn from(arg: ParsedUsize) -> Self {
    arg.value
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

impl Prism<String, ParsedUsize> for StringToUsize {
  fn narrow_type(&self, input: String) -> Result<ParsedUsize, String> {
    if let Ok(value) = input.parse() {
      Ok(ParsedUsize { value, input })
    } else {
      Err(input)
    }
  }
  fn widen_type(&self, input: ParsedUsize) -> String {
    input.input
  }
}

impl Prism<Expr, IntervalOrNumber> for ExprToIntervalLike {
  fn narrow_type(&self, input: Expr) -> Result<IntervalOrNumber, Expr> {
    match Interval::try_from(input) {
      Ok(interval) => Ok(IntervalOrNumber::Interval(interval)),
      Err(err) => {
        let input = err.recover_payload();
        match Number::try_from(input) {
          Ok(number) => Ok(IntervalOrNumber::Number(number)),
          Err(err) => Err(err.recover_payload()),
        }
      }
    }
  }
  fn widen_type(&self, input: IntervalOrNumber) -> Expr {
    match input {
      IntervalOrNumber::Interval(interval) => interval.into(),
      IntervalOrNumber::Number(number) => number.into(),
    }
  }
}
