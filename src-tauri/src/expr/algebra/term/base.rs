
use crate::expr::Expr;
use crate::expr::number::Number;
use crate::expr::algebra::factor::Factor;
use crate::expr::arithmetic::ArithExpr;
use crate::units::convertible::TemperatureConvertible;
use crate::util::PreOne;

use num::One;

use std::ops::{Mul, Div, MulAssign, DivAssign};
use std::fmt::{self, Display, Formatter};

/// A `Term` is an [`Expr`], represented as a numerator and a
/// denominator, both of which are products of [`Factor`]s. The
/// expressions in the products shall NOT themselves be applications
/// of the `*` operator and shall NOT be 2-arity applications of the
/// `/` operator. (For the purposes of completeness, applications of
/// `/` with argument count not equal to 2 are permitted, though such
/// applications make little sense in practice)
///
/// Every expression can be interpreted as a `Term`, even if such an
/// interpretation is trivial (i.e. the denominator is empty and the
/// numerator contains one factor with no exponential part). This
/// interpretation is realized via [`Term::parse`]. The opposite
/// mapping (from a `Term` back to an `Expr`) is realized by a
/// `From<Term> for Expr` impl. Note that the two are NOT inverses of
/// each other, as `Term::parse_expr` can lose information about
/// nested denominators as it attempts to automatically simplify
/// rational expressions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Term {
  numerator: Vec<Factor>,
  denominator: Vec<Factor>,
}

impl Term {
  pub fn numerator(&self) -> &[Factor] {
    &self.numerator
  }

  pub fn denominator(&self) -> &[Factor] {
    &self.denominator
  }

  pub fn into_parts(self) -> (Vec<Factor>, Vec<Factor>) {
    (self.numerator, self.denominator)
  }

  pub fn into_numerator(self) -> Vec<Factor> {
    self.numerator
  }

  pub fn into_denominator(self) -> Vec<Factor> {
    self.denominator
  }

  pub fn filter_factors<F>(mut self, mut f: F) -> Self
  where F: FnMut(&Factor) -> bool {
    self.numerator.retain(&mut f);
    self.denominator.retain(&mut f);
    self
  }

  pub fn partition_factors<F>(self, mut f: F) -> (Self, Self)
  where F: FnMut(&Factor) -> bool {
    let (num1, num2) = self.numerator.into_iter().partition(&mut f);
    let (den1, den2) = self.denominator.into_iter().partition(&mut f);
    (
      Self { numerator: num1, denominator: den1 },
      Self { numerator: num2, denominator: den2 },
    )
  }

  pub fn recip(self) -> Self {
    Term {
      numerator: self.denominator,
      denominator: self.numerator,
    }
  }

  pub fn is_empty(&self) -> bool {
    self.numerator.is_empty() && self.denominator.is_empty()
  }

  /// Removes any literal 1 values from the term. Specifically,
  /// removes any value for which [`Expr::is_one`] is true.
  pub fn remove_ones(self) -> Self {
    self.filter_factors(|f| !f.is_pre_one())
  }

  pub fn from_parts<I1, I2>(numerator: I1, denominator: I2) -> Term
  where I1: IntoIterator,
        I2: IntoIterator,
        I1::Item: Into<Expr>,
        I2::Item: Into<Expr> {
    let numerator = numerator.into_iter()
      .map(|expr| Self::parse(expr.into()))
      .fold(Term::one(), |acc, x| acc * x);
    let denominator = denominator.into_iter()
      .map(|expr| Self::parse(expr.into()))
      .fold(Term::one(), |acc, x| acc * x);
    numerator / denominator
  }

  pub fn from_numerator<I1>(numerator: I1) -> Term
  where I1: IntoIterator,
        I1::Item: Into<Expr> {
    Self::from_parts(numerator, Vec::<Expr>::new())
  }

  pub fn from_denominator<I2>(denominator: I2) -> Term
  where I2: IntoIterator,
        I2::Item: Into<Expr> {
    Self::from_parts(Vec::<Expr>::new(), denominator)
  }

  pub fn parse(expr: Expr) -> Term {
    match expr {
      Expr::Call(function_name, args) => {
        match function_name.as_ref() {
          "*" => {
            args.into_iter()
              .map(Self::parse)
              .fold(Term::one(), |acc, x| acc * x)
          }
          "/" if args.len() == 2 => {
            let [numerator, denominator] = args.try_into().unwrap();
            let numerator = Self::parse(numerator);
            let denominator = Self::parse(denominator);
            numerator / denominator
          }
          _ => {
            // Unknown function application, return a trivial Term.
            Term {
              numerator: vec![Factor::parse(Expr::Call(function_name, args))],
              denominator: Vec::new(),
            }
          }
        }
      }
      expr => {
        // Atomic expression, return a trivial Term.
        Term {
          numerator: vec![Factor::parse(expr)],
          denominator: Vec::new(),
        }
      }
    }
  }
}

impl From<Term> for Expr {
  fn from(t: Term) -> Expr {
    let numerator = ArithExpr::product(t.numerator.into_iter().map(ArithExpr::from).collect());
    if t.denominator.is_empty() {
      numerator.into()
    } else {
      let denominator = ArithExpr::product(t.denominator.into_iter().map(ArithExpr::from).collect());
      (numerator / denominator).into()
    }
  }
}

impl Display for Term {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    fmt_product(f, &self.numerator)?;
    if !self.denominator.is_empty() {
      write!(f, " / ")?;
      fmt_product(f, &self.denominator)?;
    }
    Ok(())
  }
}

fn fmt_product(f: &mut Formatter, factors: &[Factor]) -> fmt::Result {
  if factors.is_empty() {
    write!(f, "1")?;
    return Ok(());
  }
  let mut first = true;
  for factor in factors {
    if !first {
      write!(f, " ")?;
    } else {
      first = false;
    }
    write!(f, "{}", factor)?;
  }
  Ok(())
}

impl From<Number> for Term {
  fn from(n: Number) -> Self {
    Term::parse(Expr::from(n))
  }
}

impl MulAssign for Term {
  fn mul_assign(&mut self, other: Self) {
    self.numerator.extend(other.numerator);
    self.denominator.extend(other.denominator);
  }
}

impl Mul for Term {
  type Output = Term;

  fn mul(mut self, other: Self) -> Self {
    self *= other;
    self
  }
}

impl Mul<Number> for Term {
  type Output = Term;

  fn mul(self, other: Number) -> Self {
    self * Term::from(other)
  }
}

impl Mul<&Number> for Term {
  type Output = Term;

  fn mul(self, other: &Number) -> Self {
    self * other.clone()
  }
}

impl DivAssign for Term {
  fn div_assign(&mut self, other: Self) {
    self.numerator.extend(other.denominator);
    self.denominator.extend(other.numerator);
  }
}

impl Div for Term {
  type Output = Term;

  fn div(mut self, other: Self) -> Self {
    self /= other;
    self
  }
}

impl Div<Number> for Term {
  type Output = Term;

  fn div(self, other: Number) -> Self {
    self / Term::from(other)
  }
}

impl Div<&Number> for Term {
  type Output = Term;

  fn div(self, other: &Number) -> Self {
    self / other.clone()
  }
}

impl One for Term {
  fn one() -> Self {
    Term {
      numerator: Vec::new(),
      denominator: Vec::new(),
    }
  }

  fn is_one(&self) -> bool {
    self.numerator().iter().all(Factor::is_pre_one) &&
      self.denominator().iter().all(Factor::is_pre_one)
  }
}

impl TemperatureConvertible<Number> for Term {
  type Output = Expr;

  fn offset(self, offset: Option<&Number>) -> Expr {
    match offset {
      None => Expr::from(self),
      Some(number) => Expr::call("+", vec![self.into(), Expr::from(number.clone())]),
    }
  }

  fn unoffset(input: Expr, offset: Option<&Number>) -> Term {
    match offset {
      None => Term::parse(input),
      Some(number) => Term::parse(Expr::call("-", vec![input, Expr::from(number.clone())])),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_recip() {
    let term = Term {
      numerator: vec![Factor::from(0), Factor::from(1)],
      denominator: vec![Factor::from(2), Factor::from(3)],
    };
    assert_eq!(term.recip(), Term {
      numerator: vec![Factor::from(2), Factor::from(3)],
      denominator: vec![Factor::from(0), Factor::from(1)],
    });
  }

  #[test]
  fn test_parse_simple_expr() {
    let expr = Expr::call("+", vec![Expr::from(0), Expr::from(10)]);
    let term = Term::parse(expr);
    assert_eq!(term, Term {
      numerator: vec![Factor::parse(Expr::call("+", vec![Expr::from(0), Expr::from(10)]))],
      denominator: Vec::new(),
    });

    let expr = Expr::call("*", vec![Expr::from(0), Expr::from(10)]);
    let term = Term::parse(expr);
    assert_eq!(term, Term {
      numerator: vec![Factor::from(0), Factor::from(10)],
      denominator: Vec::new(),
    });

    let expr = Expr::call("/", vec![Expr::from(0), Expr::from(10)]);
    let term = Term::parse(expr);
    assert_eq!(term, Term {
      numerator: vec![Factor::from(0)],
      denominator: vec![Factor::from(10)],
    });
  }

  #[test]
  fn test_parse_expr_with_bad_division_arity() {
    let expr = Expr::call("/", vec![Expr::from(0), Expr::from(10), Expr::from(15)]);
    let term = Term::parse(expr);
    assert_eq!(term, Term {
      numerator: vec![Factor::parse(Expr::call("/", vec![Expr::from(0), Expr::from(10), Expr::from(15)]))],
      denominator: Vec::new(),
    });
  }

  #[test]
  fn test_parse_nested_division() {
    let expr = Expr::call("/", vec![
      Expr::call("/", vec![Expr::from(0), Expr::from(10)]),
      Expr::call("/", vec![Expr::from(100), Expr::from(110)]),
    ]);
    let term = Term::parse(expr);
    assert_eq!(term, Term {
      numerator: vec![Factor::from(0), Factor::from(110)],
      denominator: vec![Factor::from(10), Factor::from(100)],
    });
  }

  #[test]
  fn test_parse_nested_multiplication() {
    let expr = Expr::call("*", vec![
      Expr::call("*", vec![Expr::from(0), Expr::from(10)]),
      Expr::call("*", vec![Expr::from(100), Expr::from(110)]),
    ]);
    let term = Term::parse(expr);
    assert_eq!(term, Term {
      numerator: vec![Factor::from(0), Factor::from(10), Factor::from(100), Factor::from(110)],
      denominator: Vec::new(),
    });
  }

  #[test]
  fn test_parse_nested_mixed_ops_1() {
    let expr = Expr::call("/", vec![
      Expr::call("*", vec![Expr::from(0), Expr::from(10)]),
      Expr::call("*", vec![Expr::from(100), Expr::from(110)]),
    ]);
    let term = Term::parse(expr);
    assert_eq!(term, Term {
      numerator: vec![Factor::from(0), Factor::from(10)],
      denominator: vec![Factor::from(100), Factor::from(110)],
    });
  }

  #[test]
  fn test_parse_nested_mixed_ops_2() {
    let expr = Expr::call("*", vec![
      Expr::call("/", vec![Expr::from(0), Expr::from(10)]),
      Expr::call("/", vec![Expr::from(100), Expr::from(110)]),
    ]);
    let term = Term::parse(expr);
    assert_eq!(term, Term {
      numerator: vec![Factor::from(0), Factor::from(100)],
      denominator: vec![Factor::from(10), Factor::from(110)],
    });
  }
}
