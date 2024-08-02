
mod matcher;

pub use matcher::{MatcherSpec, MatchedExpr};

use super::Expr;
use super::var::Var;
use super::atom::Atom;
use super::number::{Number, ComplexNumber, ComplexLike};
use super::interval::{RawInterval, IntervalOrScalar};
use super::literal::Literal;
use super::incomplete::IncompleteObject;
use super::algebra::formula::{Formula, Equation};
use super::algebra::infinity::InfiniteConstant;
use crate::util::prism::{Prism, PrismExt, Iso, OnVec, OnTuple2, Only, Conversion,
                         LosslessConversion, VecToArray};
use crate::graphics::GRAPHICS_NAME;

use num::{Zero, One};
use either::Either;

// Re-export some useful expression-adjacent prisms.
pub use super::var::StringToVar;
pub use super::vector::ExprToVector;
pub use super::vector::matrix::{ExprToTypedMatrix, expr_to_matrix};
pub use super::vector::tensor::ExprToTensor;
pub use super::number::prisms::{NumberToUsize, NumberToI64};
pub use super::algebra::infinity::{ExprToInfinity, UnboundedNumber,
                                   infinity_to_signed_infinity,
                                   expr_to_signed_infinity, expr_to_unbounded_number};

/// An expression which is literally equal to the value zero.
#[derive(Debug, Clone)]
pub struct LiteralZero {
  expr: Expr,
}

/// An expression which is literally equal to the value one.
#[derive(Debug, Clone)]
pub struct LiteralOne {
  expr: Expr,
}

/// An expression which is a 2D graphics primitive.
#[derive(Debug, Clone)]
pub struct Graphics2D {
  inner_expr: MatchedExpr<Graphics2DSpec>,
}

pub struct Graphics2DSpec;

/// Prism which only accepts the literal numerical value zero.
#[derive(Debug, Clone, Copy, Default)]
pub struct ExprToZero;

/// Prism which only accepts the literal numerical value one.
#[derive(Debug, Clone, Copy, Default)]
pub struct ExprToOne;

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
/// result type of the [`expr_to_positive_number`] prism.
#[derive(Debug, Clone)]
pub struct PositiveNumber {
  data: Number,
}

/// Prism which reads a string as a non-negative integer.
#[derive(Debug, Clone)]
pub struct StringToUsize;

/// Prism which reads a string as an arbitrary integer.
#[derive(Debug, Clone)]
pub struct StringToI64;

/// Equivalent to `usize` but also keeps track of the string used to
/// construct it. This ensures that the [`StringToUsize`] prism is
/// lawful and can recover the original string on `widen_type`.
#[derive(Debug, Clone)]
pub struct ParsedUsize {
  value: usize,
  input: String,
}

/// Equivalent to `i64` but also keeps track of the string used to
/// construct it. This ensures that the [`StringToI64`] prism is
/// lawful and can recover the original string on `widen_type`.
#[derive(Debug, Clone)]
pub struct ParsedI64 {
  value: i64,
  input: String,
}

/// Prism which accepts only a specific variable as the expression.
pub fn must_be_var(var: Var) -> Only<Expr> {
  let expr = Expr::Atom(Atom::Var(var));
  Only::new(expr)
}

/// Prism which only accepts real numerical literals.
pub fn expr_to_number() -> impl Prism<Expr, Number> + Clone {
  Conversion::new()
}

/// Prism which only accepts string literals.
pub fn expr_to_string() -> impl Prism<Expr, String> + Clone {
  Conversion::new()
}

pub fn expr_to_incomplete_object() -> impl Prism<Expr, IncompleteObject> + Clone {
  Conversion::new()
}

/// Prism which only accepts variables.
pub fn expr_to_var() -> impl Prism<Expr, Var> + Clone {
  Conversion::new()
}

/// Prism which accepts only positive real numbers.
pub fn expr_to_positive_number() -> impl Prism<Expr, PositiveNumber> + Clone {
  expr_to_number().composed(NumberToPositiveNumber)
}

/// Prism which only accepts expressions containing [`Number`] values
/// representable by a `usize`.
pub fn expr_to_usize() -> impl Prism<Expr, usize> + Clone {
  expr_to_number().composed(NumberToUsize)
}

/// Prism which only accepts expressions containing [`Number`] values
/// representable by an `i64`.
pub fn expr_to_i64() -> impl Prism<Expr, i64> + Clone {
  expr_to_number().composed(NumberToI64)
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
pub fn expr_to_equation() -> impl Prism<Expr, Equation> + Clone {
  expr_to_formula().composed(Conversion::new())
}

pub fn expr_to_any_interval() -> Conversion<Expr, RawInterval<Expr>> {
  Conversion::new()
}

pub fn expr_to_interval() -> Conversion<Expr, RawInterval<Number>> {
  Conversion::new()
}

pub fn expr_to_unbounded_interval() -> Conversion<Expr, RawInterval<UnboundedNumber>> {
  Conversion::new()
}

pub fn expr_to_interval_like() -> impl Prism<Expr, IntervalOrScalar<Number>> + Clone {
  expr_to_interval().or(expr_to_number()).composed(either_to_interval_like())
}

pub fn expr_to_unbounded_interval_like() -> impl Prism<Expr, IntervalOrScalar<UnboundedNumber>> + Clone {
  expr_to_unbounded_interval().or(expr_to_unbounded_number()).composed(either_to_interval_like())
}

#[allow(clippy::type_complexity)] // It's an internal function for code reuse purposes; users never see this type.
fn either_to_interval_like<T>() -> Iso<Either<RawInterval<T>, T>, IntervalOrScalar<T>, fn(Either<RawInterval<T>, T>) -> IntervalOrScalar<T>, fn(IntervalOrScalar<T>) -> Either<RawInterval<T>, T>> {
  Iso::new(|either| match either {
    Either::Left(i) => IntervalOrScalar::Interval(i),
    Either::Right(n) => IntervalOrScalar::Scalar(n),
  }, |interval_like| match interval_like {
    IntervalOrScalar::Interval(i) => Either::Left(i),
    IntervalOrScalar::Scalar(n) => Either::Right(n),
  })
}

pub fn expr_to_number_or_inf() -> impl Prism<Expr, Either<Number, InfiniteConstant>> + Clone {
  expr_to_number().or(ExprToInfinity)
}

pub fn expr_to_complex_or_inf() -> impl Prism<Expr, Either<ComplexLike, InfiniteConstant>> + Clone {
  ExprToComplex.or(ExprToInfinity)
}

/// Prism which parses an [`Expr`] as a vector (in the expression
/// language) whose constituents each pass the specified prism
/// `inner`.
pub fn expr_to_typed_vector<T, P>(inner: P) -> impl Prism<Expr, Vec<T>> + Clone
where P: Prism<Expr, T> + Clone {
  ExprToVector
    .composed(LosslessConversion::new())
    .composed(OnVec::new(inner))
}

/// Prism which parses an [`Expr`] as a vector (in the expression
/// language), as though through [`expr_to_typed_vector`], but which
/// only accepts vectors of the given length.
pub fn expr_to_typed_array<const N: usize, T, P>(inner: P) -> impl Prism<Expr, [T; N]> + Clone
where P: Prism<Expr, T> + Clone {
  expr_to_typed_vector(inner)
    .composed(VecToArray::new())
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

  pub fn is_zero(&self) -> bool {
    self.data.is_zero()
  }

  pub fn is_one(&self) -> bool {
    self.data.is_one()
  }
}

impl LiteralZero {
  pub fn new(arg: Expr) -> Result<LiteralZero, Expr> {
    ExprToZero.narrow_type(arg)
  }
}

impl LiteralOne {
  pub fn new(arg: Expr) -> Result<LiteralOne, Expr> {
    ExprToOne.narrow_type(arg)
  }
}

impl Graphics2D {
  pub fn into_args(self) -> Vec<Expr> {
    self.inner_expr.into_args()
  }

  pub fn args(&self) -> &[Expr] {
    self.inner_expr.args()
  }

  pub fn prism() -> impl Prism<Expr, Graphics2D> + Clone {
    Graphics2DSpec::prism().rmap(|inner_expr| Graphics2D { inner_expr }, |gfx| gfx.inner_expr)
  }
}

impl From<Graphics2D> for MatchedExpr<Graphics2DSpec> {
  fn from(gfx: Graphics2D) -> Self {
    gfx.inner_expr
  }
}

impl From<MatchedExpr<Graphics2DSpec>> for Graphics2D {
  fn from(arg: MatchedExpr<Graphics2DSpec>) -> Self {
    Graphics2D { inner_expr: arg }
  }
}

impl From<ParsedUsize> for usize {
  fn from(arg: ParsedUsize) -> Self {
    arg.value
  }
}

impl From<ParsedI64> for i64 {
  fn from(arg: ParsedI64) -> Self {
    arg.value
  }
}

impl From<PositiveNumber> for Number {
  fn from(arg: PositiveNumber) -> Self {
    arg.data
  }
}

impl From<LiteralZero> for Expr {
  fn from(arg: LiteralZero) -> Self {
    arg.expr
  }
}

impl MatcherSpec for Graphics2DSpec {
  const MIN_ARITY: usize = 0;
  const MAX_ARITY: usize = usize::MAX;
  const FUNCTION_NAME: &'static str = GRAPHICS_NAME;
}

impl Prism<Expr, LiteralZero> for ExprToZero {
  fn narrow_type(&self, input: Expr) -> Result<LiteralZero, Expr> {
    if input.is_zero() {
      Ok(LiteralZero { expr: input })
    } else {
      Err(input)
    }
  }
  fn widen_type(&self, input: LiteralZero) -> Expr {
    input.expr
  }
}

impl Prism<Expr, LiteralOne> for ExprToOne {
  fn narrow_type(&self, input: Expr) -> Result<LiteralOne, Expr> {
    if input.is_one() {
      Ok(LiteralOne { expr: input })
    } else {
      Err(input)
    }
  }
  fn widen_type(&self, input: LiteralOne) -> Expr {
    input.expr
  }
}

impl Prism<Expr, ComplexLike> for ExprToComplex {
  fn narrow_type(&self, input: Expr) -> Result<ComplexLike, Expr> {
    match input {
      Expr::Atom(Atom::Number(r)) => Ok(ComplexLike::Real(r)),
      Expr::Call(function_name, args) => {
        if function_name == ComplexNumber::FUNCTION_NAME && args.len() == 2 {
          let [a, b] = args.try_into().unwrap();
          match OnTuple2::both(expr_to_number()).narrow_type((a, b)) {
            Err((a, b)) => Err(Expr::Call(function_name, vec![a, b])),
            Ok((a, b)) => Ok(ComplexLike::Complex(ComplexNumber::new(a, b))),
          }
        } else {
          Err(Expr::Call(function_name, args))
        }
      }
      _ => Err(input),
    }
  }

  fn widen_type(&self, input: ComplexLike) -> Expr {
    match input {
      ComplexLike::Real(r) => r.into(),
      ComplexLike::Complex(z) => z.into(),
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

impl Prism<String, ParsedI64> for StringToI64 {
  fn narrow_type(&self, input: String) -> Result<ParsedI64, String> {
    if let Ok(value) = input.parse() {
      Ok(ParsedI64 { value, input })
    } else {
      Err(input)
    }
  }
  fn widen_type(&self, input: ParsedI64) -> String {
    input.input
  }
}
