
mod matcher;

pub use matcher::{MatcherSpec, MatchedExpr};

use super::Expr;
use super::call::CallExpr;
use super::var::Var;
use super::atom::Atom;
use super::number::{Number, ComplexNumber, Quaternion, ComplexLike, QuaternionLike};
use super::interval::{RawInterval, IntervalOrScalar};
use super::literal::Literal;
use super::incomplete::IncompleteObject;
use super::algebra::formula::{Formula, Equation};
use super::algebra::infinity::InfiniteConstant;
use crate::util::prism::{Prism, PrismExt, Iso, OnVec, OnTuple2, Only, Conversion,
                         LosslessConversion, VecToArray};
use crate::util::tuple::binder::{PrismTupleList, narrow_vec};
use crate::graphics::GRAPHICS_NAME;

use num::{Zero, One};
use either::Either;
use tuple_list::{Tuple, TupleList};

// Re-export some useful expression-adjacent prisms.
pub use super::var::StringToVar;
pub use super::vector::ExprToVector;
pub use super::vector::matrix::{ExprToTypedMatrix, expr_to_matrix};
pub use super::vector::tensor::ExprToTensor;
pub use super::number::prisms::{number_to_usize, number_to_i64, number_to_i32, number_to_u8, number_to_u32};
pub use super::algebra::infinity::{ExprToInfinity, UnboundedNumber,
                                   infinity_to_signed_infinity,
                                   expr_to_signed_infinity, expr_to_unbounded_number};
pub use super::datetime::prisms::{expr_to_datetime, expr_to_datetime_or_real};

/// An expression which is literally equal to the value zero.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiteralZero {
  expr: Expr,
}

/// An expression which is literally equal to the value one.
#[derive(Debug, Clone, PartialEq, Eq)]
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

/// Prism which downcasts an [`Expr`] to a [`QuaternionLike`]: a real
/// number, complex number, or a quaternion.
#[derive(Debug, Clone, Copy, Default)]
pub struct ExprToQuaternion;

/// Prism which only accepts expressions which are a [`Var`].
#[derive(Debug, Clone, Copy, Default)]
pub struct ExprToVar;

/// Prism which downcasts a [`Number`] to a [`PositiveNumber`]. Fails
/// on negative numbers.
#[derive(Debug, Clone, Copy, Default)]
pub struct NumberToPositiveNumber;

/// A real number which is guaranteed to be positive. This is the
/// result type of the [`expr_to_positive_number`] prism.
#[derive(Debug, Clone, PartialEq, Eq)]
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

/// Prism which only call expressions.
pub fn expr_to_functor_call() -> impl Prism<Expr, CallExpr> + Clone {
  Conversion::new()
}

/// Prism which accepts only positive real numbers.
pub fn expr_to_positive_number() -> impl Prism<Expr, PositiveNumber> + Clone {
  expr_to_number().composed(NumberToPositiveNumber)
}

/// Prism which only accepts expressions containing [`Number`] values
/// representable by a `usize`.
pub fn expr_to_usize() -> impl Prism<Expr, usize> + Clone {
  expr_to_number().composed(number_to_usize())
}

/// Prism which only accepts expressions containing [`Number`] values
/// representable by an `i64`.
pub fn expr_to_i64() -> impl Prism<Expr, i64> + Clone {
  expr_to_number().composed(number_to_i64())
}

pub fn expr_to_i32() -> impl Prism<Expr, i32> + Clone {
  expr_to_number().composed(number_to_i32())
}

pub fn expr_to_u8() -> impl Prism<Expr, u8> + Clone {
  expr_to_number().composed(number_to_u8())
}

pub fn expr_to_u32() -> impl Prism<Expr, u32> + Clone {
  expr_to_number().composed(number_to_u32())
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

pub fn expr_to_quaternion_or_inf() -> impl Prism<Expr, Either<QuaternionLike, InfiniteConstant>> + Clone {
  ExprToQuaternion.or(ExprToInfinity)
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

/// As [`narrow_vec`] but operating on an [`Expr`]. If the expression
/// is atomic or is a call to the wrong function, this narrowing
/// always fails. Otherwise, the narrowing operation is applied (via
/// `narrow_vec`) to the arguments list.
pub fn narrow_args<Xs, Ps>(expected_head: &str, prisms: Ps, expr: Expr) -> Result<Xs::Tuple, Expr>
where Ps: Tuple,
      Ps::TupleList: PrismTupleList<Expr, Xs>,
      Xs: TupleList {
  match expr {
    Expr::Call(head, args) if head == expected_head => {
      match narrow_vec(prisms, args) {
        Ok(args_tuple) => Ok(args_tuple),
        Err(args) => Err(Expr::Call(head, args)),
      }
    }
    expr => Err(expr),
  }
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

impl Prism<Expr, QuaternionLike> for ExprToQuaternion {
  fn narrow_type(&self, input: Expr) -> Result<QuaternionLike, Expr> {
    fn quaternion_prism() -> impl Prism<(((Expr, Expr), Expr), Expr), (((Number, Number), Number), Number)> {
      expr_to_number()
        .and(expr_to_number())
        .and(expr_to_number())
        .and(expr_to_number())
    }

    match input {
      Expr::Atom(Atom::Number(r)) => Ok(QuaternionLike::Real(r)),
      Expr::Call(function_name, args) => {
        if function_name == ComplexNumber::FUNCTION_NAME && args.len() == 2 {
          let [a, b] = args.try_into().unwrap();
          match OnTuple2::both(expr_to_number()).narrow_type((a, b)) {
            Err((a, b)) => Err(Expr::Call(function_name, vec![a, b])),
            Ok((a, b)) => Ok(QuaternionLike::Complex(ComplexNumber::new(a, b))),
          }
        } else if function_name == Quaternion::FUNCTION_NAME && args.len() == 4 {
          let [r, i, j, k] = args.try_into().unwrap();
          match quaternion_prism().narrow_type((((r, i), j), k)) {
            Err((((r, i), j), k)) => Err(Expr::Call(function_name, vec![r, i, j, k])),
            Ok((((r, i), j), k)) => Ok(QuaternionLike::Quaternion(Quaternion::new(r, i, j, k))),
          }
        } else {
          Err(Expr::Call(function_name, args))
        }
      }
      _ => Err(input),
    }
  }

  fn widen_type(&self, input: QuaternionLike) -> Expr {
    match input {
      QuaternionLike::Real(r) => r.into(),
      QuaternionLike::Complex(z) => z.into(),
      QuaternionLike::Quaternion(q) => q.into(),
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_expr_to_zero_widen() {
    let prism = ExprToZero;
    assert_eq!(
      prism.widen_type(LiteralZero { expr: Expr::zero() }),
      Expr::zero(),
    );
  }

  #[test]
  fn test_expr_to_one_widen() {
    let prism = ExprToOne;
    assert_eq!(
      prism.widen_type(LiteralOne { expr: Expr::one() }),
      Expr::one(),
    );
  }

  #[test]
  fn test_expr_to_zero_narrow() {
    let prism = ExprToZero;
    assert!(prism.narrow_type(Expr::zero()).is_ok());
    assert!(prism.narrow_type(Expr::call("complex", vec![Expr::zero(), Expr::zero()])).is_ok());
    assert!(prism.narrow_type(Expr::call("quat", vec![Expr::zero(), Expr::zero(), Expr::zero(), Expr::zero()])).is_ok());
    assert!(prism.narrow_type(Expr::one()).is_err());
    assert!(prism.narrow_type(Expr::call("complex", vec![Expr::zero(), Expr::one()])).is_err());
    assert!(prism.narrow_type(Expr::call("quat", vec![Expr::zero(), Expr::zero(), Expr::one(), Expr::zero()])).is_err());
    assert!(prism.narrow_type(Expr::call("quat", vec![Expr::zero(), Expr::zero()])).is_err());
    assert!(prism.narrow_type(Expr::call("foobar", vec![Expr::zero(), Expr::zero()])).is_err());
    assert!(prism.narrow_type(Expr::var("abc").unwrap()).is_err());
  }

  #[test]
  fn test_expr_to_one_narrow() {
    let prism = ExprToOne;
    assert!(prism.narrow_type(Expr::one()).is_ok());
    assert!(prism.narrow_type(Expr::call("complex", vec![Expr::one(), Expr::zero()])).is_ok());
    assert!(prism.narrow_type(Expr::call("quat", vec![Expr::one(), Expr::zero(), Expr::zero(), Expr::zero()])).is_ok());
    assert!(prism.narrow_type(Expr::from(9)).is_err());
    assert!(prism.narrow_type(Expr::call("complex", vec![Expr::from(1), Expr::from(1)])).is_err());
    assert!(prism.narrow_type(Expr::call("quat", vec![Expr::one(), Expr::one(), Expr::one(), Expr::one()])).is_err());
    assert!(prism.narrow_type(Expr::call("quat", vec![Expr::one(), Expr::zero()])).is_err());
    assert!(prism.narrow_type(Expr::call("foobar", vec![Expr::one(), Expr::zero()])).is_err());
    assert!(prism.narrow_type(Expr::var("abc").unwrap()).is_err());
  }

  #[test]
  fn test_expr_to_complex_widen() {
    let prism = ExprToComplex;
    assert_eq!(
      prism.widen_type(ComplexLike::Complex(ComplexNumber::new(1, 4))),
      Expr::call("complex", vec![Expr::from(1), Expr::from(4)]),
    );
    assert_eq!(
      prism.widen_type(ComplexLike::Real(Number::from(9))),
      Expr::from(9),
    );
  }

  #[test]
  fn test_expr_to_quaternion_widen() {
    let prism = ExprToQuaternion;
    assert_eq!(
      prism.widen_type(QuaternionLike::Real(Number::from(9))),
      Expr::from(9),
    );
    assert_eq!(
      prism.widen_type(QuaternionLike::Complex(ComplexNumber::new(9, 4))),
      Expr::call("complex", vec![Expr::from(9), Expr::from(4)]),
    );
    assert_eq!(
      prism.widen_type(QuaternionLike::Quaternion(Quaternion::new(1, 4, 7, 10))),
      Expr::call("quat", vec![Expr::from(1), Expr::from(4), Expr::from(7), Expr::from(10)]),
    );
  }

  #[test]
  fn test_expr_to_complex_narrow() {
    let prism = ExprToComplex;
    assert_eq!(
      prism.narrow_type(Expr::call("complex", vec![Expr::from(1), Expr::from(4)])).unwrap(),
      ComplexLike::Complex(ComplexNumber::new(1, 4)),
    );
    assert_eq!(
      prism.narrow_type(Expr::from(99)).unwrap(),
      ComplexLike::Real(Number::from(99)),
    );
    assert!(prism.narrow_type(Expr::call("complex", vec![Expr::from(1), Expr::from(4), Expr::from(5)])).is_err());
    assert!(prism.narrow_type(Expr::call("quat", vec![Expr::from(1), Expr::from(4), Expr::from(8), Expr::from(5)])).is_err());
    assert!(prism.narrow_type(Expr::call("complex", vec![Expr::from(1), Expr::var("xyz").unwrap()])).is_err());
    assert!(prism.narrow_type(Expr::var("abc").unwrap()).is_err());
  }

  #[test]
  fn test_expr_to_quaternion_narrow() {
    let prism = ExprToQuaternion;
    assert_eq!(
      prism.narrow_type(Expr::call("complex", vec![Expr::from(1), Expr::from(4)])).unwrap(),
      QuaternionLike::Complex(ComplexNumber::new(1, 4)),
    );
    assert_eq!(
      prism.narrow_type(Expr::call("quat", vec![Expr::from(1), Expr::from(4), Expr::from(7), Expr::from(9)])).unwrap(),
      QuaternionLike::Quaternion(Quaternion::new(1, 4, 7, 9)),
    );
    assert_eq!(
      prism.narrow_type(Expr::from(99)).unwrap(),
      QuaternionLike::Real(Number::from(99)),
    );
    assert!(prism.narrow_type(Expr::call("complex", vec![Expr::from(1), Expr::from(4), Expr::from(5)])).is_err());
    assert!(prism.narrow_type(Expr::call("quat", vec![Expr::from(1), Expr::from(4), Expr::from(5)])).is_err());
    assert!(prism.narrow_type(Expr::call("quat", vec![Expr::from(1), Expr::from(4), Expr::from(5), Expr::var("a").unwrap()])).is_err());
    assert!(prism.narrow_type(Expr::call("complex", vec![Expr::from(1), Expr::var("xyz").unwrap()])).is_err());
    assert!(prism.narrow_type(Expr::var("abc").unwrap()).is_err());
  }

  #[test]
  fn test_expr_to_var() {
    fn var(s: &str) -> Var {
      Var::new(s).unwrap()
    }
    fn evar(s: &str) -> Expr {
      Expr::var(s).unwrap()
    }

    assert_eq!(ExprToVar.widen_type(var("abc")), evar("abc"));
    assert_eq!(ExprToVar.narrow_type(evar("abc")).unwrap(), var("abc"));
    assert!(ExprToVar.narrow_type(Expr::from(9)).is_err());
    assert!(ExprToVar.narrow_type(Expr::string("abc")).is_err());
    assert!(ExprToVar.narrow_type(Expr::call("foobar", vec![])).is_err());
    assert!(ExprToVar.narrow_type(Expr::call("foobar", vec![evar("x")])).is_err());
  }

  #[test]
  fn test_number_to_positive_number() {
    assert_eq!(
      NumberToPositiveNumber.widen_type(PositiveNumber { data: Number::from(9) }),
      Number::from(9),
    );
    assert_eq!(
      NumberToPositiveNumber.narrow_type(Number::from(9)).unwrap(),
      PositiveNumber { data: Number::from(9) },
    );
    assert_eq!(
      NumberToPositiveNumber.narrow_type(Number::from(0)).unwrap_err(),
      Number::from(0),
    );
    assert_eq!(
      NumberToPositiveNumber.narrow_type(Number::from(-9)).unwrap_err(),
      Number::from(-9),
    );
  }

  #[test]
  fn test_string_to_usize() {
    assert_eq!(StringToUsize.widen_type(ParsedUsize { value: 3, input: String::from("3") }), String::from("3"));
    assert_eq!(usize::from(StringToUsize.narrow_type(String::from("84")).unwrap()), 84);
    assert!(StringToUsize.narrow_type(String::from("-3")).is_err());
    assert!(StringToUsize.narrow_type(String::from("zzz")).is_err());
  }

  #[test]
  fn test_string_to_i64() {
    assert_eq!(StringToI64.widen_type(ParsedI64 { value: 3, input: String::from("3") }), String::from("3"));
    assert_eq!(i64::from(StringToI64.narrow_type(String::from("84")).unwrap()), 84);
    assert_eq!(i64::from(StringToI64.narrow_type(String::from("-3")).unwrap()), -3);
    assert!(StringToI64.narrow_type(String::from("zzz")).is_err());
    assert!(StringToI64.narrow_type(String::from("")).is_err());
  }
}
