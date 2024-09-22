
use crate::expr::Expr;
use crate::expr::arithmetic::ArithExpr;

use num::pow::Pow;

/// A factor consists of a base and an exponent, both of which are
/// expressions.
///
/// Invariant: The base of a factor is not a binary application of the
/// `^` operator.
///
/// Every expression can be interpreted as a factor. Expressions which
/// are binary applications of the `^` operator will be treated as
/// nontrivial factors with an explicit exponent, while any other
/// expressions will be treated as having the trivial factor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Factor {
  pub base: Expr,
  pub exponent: Option<Expr>,
}

impl Factor {
  pub const EXPONENT_OPERATOR: &'static str = "^";

  pub fn base(&self) -> &Expr {
    &self.base
  }

  pub fn exponent(&self) -> Option<&Expr> {
    self.exponent.as_ref()
  }

  pub fn exponent_mut(&mut self) -> &mut Option<Expr> {
    &mut self.exponent
  }

  pub fn exponent_or_one(&self) -> Expr {
    self.exponent.clone().unwrap_or_else(Expr::one)
  }

  pub fn has_exponent(&self) -> bool {
    self.exponent.is_some()
  }

  pub fn into_parts(self) -> (Expr, Option<Expr>) {
    (self.base, self.exponent)
  }

  /// Simplifies trivial exponents. An exponent which is exactly equal
  /// to the numerical constant 1 is eliminated, and an exponent which
  /// is exactly equal to the numerical constant 0 causes the whole
  /// term to be replaced with a 1.
  pub fn simplify_trivial_powers(mut self) -> Self {
    if self.exponent.as_ref().map_or(false, Expr::is_zero) {
      self.exponent = None;
      self.base = Expr::one();
    } else if self.exponent.as_ref().map_or(false, Expr::is_one) {
      self.exponent = None;
    }
    self
  }

  /// Alias for `Factor::from`.
  pub fn parse(expr: Expr) -> Self {
    Factor::from(expr)
  }

  /// Constructs a [`Factor`] from its constituent parts. Note that if
  /// `base` is itself a binary application of the `^` operator, then
  /// this method will parse that expression as part of the factor.
  pub fn from_parts(base: Expr, exponent: Option<impl Into<Expr>>) -> Self {
    let mut result = Self::parse(base);
    if let Some(e) = exponent {
      result = result.pow(e.into());
    }
    result
  }
}

impl From<Factor> for Expr {
  fn from(factor: Factor) -> Expr {
    match factor.exponent {
      None => factor.base,
      Some(e) => Expr::call(Factor::EXPONENT_OPERATOR, vec![factor.base, e]),
    }
  }
}

impl From<Expr> for Factor {
  fn from(expr: Expr) -> Factor {
    match expr {
      Expr::Call(f, args) if f == Factor::EXPONENT_OPERATOR && args.len() == 2 => {
        let [base, exponent] = args.try_into().unwrap();
        Factor::from(base).pow(exponent)
      }
      _ => {
        Factor { base: expr, exponent: None }
      }
    }
  }
}

impl Pow<ArithExpr> for Factor {
  type Output = Factor;

  fn pow(mut self, rhs: ArithExpr) -> Self {
    self.exponent = match self.exponent {
      None => Some(rhs.into()),
      Some(e) => Some((e * rhs).into()),
    };
    self
  }
}

impl Pow<Expr> for Factor {
  type Output = Factor;

  fn pow(self, rhs: Expr) -> Self {
    self.pow(ArithExpr::from(rhs))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn assert_roundtrip_on_expr(expr: Expr) {
    let factor = Factor::from(expr.clone());
    assert_eq!(Expr::from(factor), expr);
  }

  #[test]
  fn test_roundtrip_on_expr() {
    assert_roundtrip_on_expr(Expr::from(1));
    assert_roundtrip_on_expr(Expr::from(99));
    assert_roundtrip_on_expr(Expr::call("+", vec![Expr::from(10), Expr::from(20)]));
    assert_roundtrip_on_expr(Expr::call("^", vec![Expr::from(10), Expr::from(20)]));
    assert_roundtrip_on_expr(Expr::call("^", vec![
      Expr::from(10),
      Expr::call("^", vec![Expr::from(20), Expr::from(25)]),
    ]));
  }

  #[test]
  fn test_factor_from_expr() {
    let factor = Factor::from(Expr::call("+", vec![Expr::from(10), Expr::from(20)]));
    assert_eq!(factor.base, Expr::call("+", vec![Expr::from(10), Expr::from(20)]));
    assert_eq!(factor.exponent, None);

    let factor = Factor::from(Expr::call("^", vec![Expr::from(10), Expr::from(20)]));
    assert_eq!(factor.base, Expr::from(10));
    assert_eq!(factor.exponent, Some(Expr::from(20)));

    // Note: Wrong arity for ^ operator.
    let factor = Factor::from(Expr::call("^", vec![Expr::from(10), Expr::from(20), Expr::from(30)]));
    assert_eq!(factor.base, Expr::call("^", vec![Expr::from(10), Expr::from(20), Expr::from(30)]));
    assert_eq!(factor.exponent, None);
  }

  #[test]
  fn test_factor_from_expr_with_nested_power() {
    let factor = Factor::from(Expr::call("^", vec![
      Expr::call("^", vec![Expr::from(10), Expr::from(20)]),
      Expr::var("x").unwrap(),
    ]));
    assert_eq!(factor.base, Expr::from(10));
    assert_eq!(factor.exponent, Some(Expr::call("*", vec![Expr::from(20), Expr::var("x").unwrap()])));
  }

  #[test]
  fn test_factor_from_expr_with_nested_power_and_all_powers_numerical() {
    let factor = Factor::from(Expr::call("^", vec![
      Expr::call("^", vec![Expr::from(10), Expr::from(20)]),
      Expr::from(30),
    ]));
    assert_eq!(factor.base, Expr::from(10));
    assert_eq!(factor.exponent, Some(Expr::from(600)));
  }

  #[test]
  fn test_simplify_trivial_powers() {
    let factor = Factor { base: Expr::from(10), exponent: Some(Expr::from(20)) };
    assert_eq!(factor.clone().simplify_trivial_powers(), factor);

    let factor = Factor { base: Expr::from(10), exponent: None };
    assert_eq!(factor.clone().simplify_trivial_powers(), factor);

    let factor = Factor { base: Expr::from(10), exponent: Some(Expr::from(1)) };
    assert_eq!(factor.simplify_trivial_powers(), Factor { base: Expr::from(10), exponent: None });

    let factor = Factor { base: Expr::var("x").unwrap(), exponent: Some(Expr::from(0)) };
    assert_eq!(factor.simplify_trivial_powers(), Factor { base: Expr::from(1), exponent: None });
  }
}
