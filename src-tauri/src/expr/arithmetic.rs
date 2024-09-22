
use super::Expr;
use super::atom::Atom;
use super::number::Number;

use serde::{Serialize, Deserialize};
use num::{Zero, One};
use num::pow::Pow;

use std::ops::{Add, Sub, Mul, Div};

/// This struct forms a thin wrapper around [`Expr`] and can be used
/// in arithmetic expressions, such as `+` and `*`.
///
/// Simple real numbered expressions will be simplified inline when
/// arithmetic is performed, while more complex expressions will
/// simply apply the corresponding operator at the expression level.
/// Additionally, expressions which are mathematically invalid will
/// apply the corresponding operator at the expression level, so the
/// full simplification engine can handle the problem (usually by
/// reporting an error).
///
/// This structure solves two problems.
///
/// 1. Performing arithmetic on `Expr` values directly can be
/// annoying, so this structure allows us to use built-in Rust
/// operators to do so more cleanly.
///
/// 2. Arithmetic on simple numbers is simplified immediately, which
/// makes this struct very useful in simplifiers. It is no longer
/// necessary to perform intermediate
/// [`FunctionEvaluator`](crate::expr::simplifier::FunctionEvaluator)
/// cycles to ensure simplification.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ArithExpr {
  inner: Expr,
}

impl ArithExpr {
  /// Equivalent to `ArithExpr::into` but with an explicit type, to
  /// aid in type inference in some situations.
  pub fn into_expr(self) -> Expr {
    self.into()
  }

  /// Convenience wrapper around [`Expr::call`] that works on
  /// `ArithExpr`.
  pub fn call(name: &str, args: Vec<impl Into<Expr>>) -> Self {
    let expr = Expr::call(name, args.into_iter().map(|e| e.into()).collect());
    ArithExpr::from(expr)
  }

  /// Sums a vector of `ArithExpr` values. If all of the values are
  /// real numbers, then the result will be a simple real-numbered
  /// expression. Otherwise, the result will be an addition expression
  /// consisting of the arguments.
  ///
  /// A vector of one value will always result in that value, not an
  /// addition expression containing it.
  pub fn sum(exprs: Vec<impl Into<Expr>>) -> Self {
    let exprs: Vec<_> = exprs.into_iter().map(|e| e.into()).collect();
    if exprs.iter().all(Expr::is_real) {
      let sum = exprs.into_iter().fold(Number::zero(), |acc, expr| {
        acc + ArithExpr::from(expr).unwrap_real()
      });
      sum.into()
    } else {
      ArithExpr::from(
        Expr::call_on_several("+", exprs, Expr::zero),
      )
    }
  }

  /// Returns the product of `ArithExpr` values. If all of the values
  /// are real numbers, then the result will be a simple real-numbered
  /// expression. Otherwise, the result will be a multiplication
  /// expression consisting of the arguments.
  ///
  /// A vector of one value will always result in that value, not a
  /// multiplication expression containing it.
  pub fn product(exprs: Vec<impl Into<Expr>>) -> Self {
    let exprs: Vec<_> = exprs.into_iter().map(|e| e.into()).collect();
    if exprs.iter().all(Expr::is_real) {
      let product = exprs.into_iter().fold(Number::one(), |acc, expr| {
        acc * ArithExpr::from(expr).unwrap_real()
      });
      product.into()
    } else {
      ArithExpr::from(
        Expr::call_on_several("*", exprs, Expr::one),
      )
    }
  }

  pub fn is_real(&self) -> bool {
    self.inner.is_real()
  }

  /// Unwraps the real numbered value contained immediately inside of
  /// this expression. Panics if [`ArithExpr::is_real`] is false.
  pub fn unwrap_real(self) -> Number {
    match self.inner {
      Expr::Atom(Atom::Number(n)) => n,
      expr => panic!("Expected ArithExpr to be real, but it was {:?}", expr),
    }
  }

  fn binary_op<F>(self, rhs: Self, op_name: &str, op: F) -> Self
  where F: FnOnce(Number, Number) -> Number {
    if self.is_real() && rhs.is_real() {
      ArithExpr::from(op(self.unwrap_real(), rhs.unwrap_real()))
    } else {
      ArithExpr::from(Expr::call(op_name, vec![self.into(), rhs.into()]))
    }
  }
}

impl From<Expr> for ArithExpr {
  fn from(inner: Expr) -> Self {
    Self { inner }
  }
}

impl From<Number> for ArithExpr {
  fn from(n: Number) -> Self {
    ArithExpr::from(Expr::from(n))
  }
}

impl From<i64> for ArithExpr {
  fn from(n: i64) -> Self {
    ArithExpr::from(Expr::from(n))
  }
}

impl From<f64> for ArithExpr {
  fn from(n: f64) -> Self {
    ArithExpr::from(Expr::from(n))
  }
}

impl From<ArithExpr> for Expr {
  fn from(arith_expr: ArithExpr) -> Self {
    arith_expr.inner
  }
}

impl AsRef<Expr> for ArithExpr {
  fn as_ref(&self) -> &Expr {
    &self.inner
  }
}

impl Zero for ArithExpr {
  fn zero() -> Self {
    ArithExpr::from(Expr::zero())
  }

  fn is_zero(&self) -> bool {
    self.as_ref().is_zero()
  }
}

impl One for ArithExpr {
  fn one() -> Self {
    ArithExpr::from(Expr::one())
  }

  fn is_one(&self) -> bool {
    self.as_ref().is_one()
  }
}

impl Add for ArithExpr {
  type Output = ArithExpr;

  fn add(self, rhs: Self) -> Self::Output {
    ArithExpr::binary_op(self, rhs, "+", Number::add)
  }
}

impl Sub for ArithExpr {
  type Output = ArithExpr;

  fn sub(self, rhs: Self) -> Self::Output {
    ArithExpr::binary_op(self, rhs, "-", Number::sub)
  }
}

impl Mul for ArithExpr {
  type Output = ArithExpr;

  fn mul(self, rhs: Self) -> Self::Output {
    ArithExpr::binary_op(self, rhs, "*", Number::mul)
  }
}

impl Div for ArithExpr {
  type Output = ArithExpr;

  fn div(self, rhs: Self) -> Self::Output {
    if rhs.is_zero() {
      ArithExpr::from(Expr::call("/", vec![self.into(), rhs.into()]))
    } else {
      ArithExpr::binary_op(self, rhs, "/", Number::div)
    }
  }
}

impl Pow<ArithExpr> for ArithExpr {
  type Output = ArithExpr;

  fn pow(self, rhs: Self) -> Self::Output {
    // We may expand this later and simplify positive constants. But
    // `^` has a lot of corner cases to be cognizant of (anything that
    // doesn't produce a real result), so for now we keep it simple
    // and just eliminate exponents of 0 and 1.
    if rhs.is_zero() {
      ArithExpr::one()
    } else if rhs.is_one() {
      self
    } else {
      ArithExpr::from(Expr::call("^", vec![self.into(), rhs.into()]))
    }
  }
}

impl Pow<Number> for ArithExpr {
  type Output = ArithExpr;

  fn pow(self, rhs: Number) -> Self::Output {
    self.pow(ArithExpr::from(rhs))
  }
}

macro_rules! impl_mixed_arith {
  (impl $trait: ident for ArithExpr { fn $method: ident };) => {
    impl $trait<Expr> for ArithExpr {
      type Output = ArithExpr;

      fn $method(self, rhs: Expr) -> Self::Output {
        ArithExpr::$method(self, ArithExpr::from(rhs))
      }
    }

    impl $trait<ArithExpr> for Expr {
      type Output = ArithExpr;

      fn $method(self, rhs: ArithExpr) -> Self::Output {
        ArithExpr::$method(ArithExpr::from(self), rhs)
      }
    }
  }
}

impl_mixed_arith! { impl Add for ArithExpr { fn add }; }
impl_mixed_arith! { impl Sub for ArithExpr { fn sub }; }
impl_mixed_arith! { impl Mul for ArithExpr { fn mul }; }
impl_mixed_arith! { impl Div for ArithExpr { fn div }; }
impl_mixed_arith! { impl Pow for ArithExpr { fn pow }; }

#[cfg(test)]
mod tests {
  use super::*;

  fn avar(s: &str) -> ArithExpr {
    ArithExpr::from(Expr::var(s).unwrap())
  }

  fn var(s: &str) -> Expr {
    Expr::var(s).unwrap()
  }

  #[test]
  fn test_sum_on_reals() {
    let input = Vec::<ArithExpr>::new();
    assert_eq!(ArithExpr::sum(input), ArithExpr::zero());
    let input = vec![ArithExpr::from(Expr::from(9))];
    assert_eq!(ArithExpr::sum(input), ArithExpr::from(9));
    let input = vec![ArithExpr::from(Expr::from(9)), ArithExpr::from(Expr::from(1))];
    assert_eq!(ArithExpr::sum(input), ArithExpr::from(10));
    let input = vec![ArithExpr::from(Expr::from(9)), ArithExpr::from(Expr::from(1)), ArithExpr::from(Expr::from(2))];
    assert_eq!(ArithExpr::sum(input), ArithExpr::from(12));
  }

  #[test]
  fn test_sum_on_vars() {
    let input = vec![avar("x"), avar("y")];
    assert_eq!(ArithExpr::sum(input), ArithExpr::from(Expr::call("+", vec![var("x"), var("y")])));
    let input = vec![avar("x")];
    assert_eq!(ArithExpr::sum(input), avar("x"));
    let input = vec![avar("x"), ArithExpr::from(10), ArithExpr::from(20)];
    assert_eq!(
      ArithExpr::sum(input),
      ArithExpr::from(Expr::call("+", vec![var("x"), Expr::from(10), Expr::from(20)])),
    );
  }

  #[test]
  fn test_product_on_reals() {
    let input = Vec::<ArithExpr>::new();
    assert_eq!(ArithExpr::product(input), ArithExpr::one());
    let input = vec![ArithExpr::from(Expr::from(9))];
    assert_eq!(ArithExpr::product(input), ArithExpr::from(9));
    let input = vec![ArithExpr::from(Expr::from(9)), ArithExpr::from(Expr::from(2))];
    assert_eq!(ArithExpr::product(input), ArithExpr::from(18));
    let input = vec![ArithExpr::from(Expr::from(9)), ArithExpr::from(Expr::from(3)), ArithExpr::from(Expr::from(2))];
    assert_eq!(ArithExpr::product(input), ArithExpr::from(54));
  }

  #[test]
  fn test_product_on_vars() {
    let input = vec![avar("x"), avar("y")];
    assert_eq!(ArithExpr::product(input), ArithExpr::from(Expr::call("*", vec![var("x"), var("y")])));
    let input = vec![avar("x")];
    assert_eq!(ArithExpr::product(input), avar("x"));
    let input = vec![avar("x"), ArithExpr::from(0), ArithExpr::from(20)];
    assert_eq!(
      ArithExpr::product(input),
      ArithExpr::from(Expr::call("*", vec![var("x"), Expr::from(0), Expr::from(20)])),
    );
  }

  #[test]
  fn test_arithmetic_ops_on_vars() {
    assert_eq!(avar("x") + avar("y"), ArithExpr::from(Expr::call("+", vec![var("x"), var("y")])));
    assert_eq!(avar("x") * avar("y"), ArithExpr::from(Expr::call("*", vec![var("x"), var("y")])));
    assert_eq!(avar("x") - avar("y"), ArithExpr::from(Expr::call("-", vec![var("x"), var("y")])));
    assert_eq!(avar("x") / avar("y"), ArithExpr::from(Expr::call("/", vec![var("x"), var("y")])));
    assert_eq!(
      avar("x") - ArithExpr::from(10),
      ArithExpr::from(Expr::call("-", vec![var("x"), Expr::from(10)])),
    );
    assert_eq!(
      ArithExpr::from(10) - avar("x"),
      ArithExpr::from(Expr::call("-", vec![Expr::from(10), var("x")])),
    );
  }

  #[test]
  fn test_arithmetic_ops_on_reals() {
    assert_eq!(ArithExpr::from(10) + ArithExpr::from(20), ArithExpr::from(30));
    assert_eq!(ArithExpr::from(10) + ArithExpr::from(0), ArithExpr::from(10));
    assert_eq!(ArithExpr::from(10) * ArithExpr::from(20), ArithExpr::from(200));
    assert_eq!(ArithExpr::from(10) - ArithExpr::from(20), ArithExpr::from(-10));
    assert_eq!(ArithExpr::from(4) / ArithExpr::from(2), ArithExpr::from(2));
    assert_eq!(ArithExpr::from(2) / ArithExpr::from(4), ArithExpr::from(0.5));
  }

  #[test]
  fn test_division_by_zero_on_arith_expr() {
    assert_eq!(ArithExpr::from(0) / ArithExpr::from(1), ArithExpr::from(0));
    assert_eq!(
      ArithExpr::from(1) / ArithExpr::from(0),
      ArithExpr::from(Expr::call("/", vec![Expr::from(1), Expr::from(0)])),
    );
  }

  #[test]
  fn test_is_real() {
    // True cases
    assert!(ArithExpr::zero().is_real());
    assert!(ArithExpr::one().is_real());
    assert!(ArithExpr::from(10).is_real());
    assert!(ArithExpr::from(0.2).is_real());
    assert!(ArithExpr::from(Expr::from(Number::ratio(1, 2))).is_real());
    // False cases
    assert!(!ArithExpr::from(Expr::call("+", vec![])).is_real());
    assert!(!ArithExpr::from(Expr::call("complex", vec![Expr::from(1), Expr::from(0)])).is_real());
  }
}
