
use crate::expr::Expr;
use crate::expr::number::Number;
use super::parser::TermParser;

use num::One;

use std::ops::{Mul, Div, MulAssign, DivAssign};
use std::fmt::{self, Display, Formatter};

/// A `Term` is an [`Expr`], represented as a numerator and a
/// denominator, both of which are products of expressions. The
/// expressions in the products shall NOT be applications of the `*`
/// operator and shall NOT be 2-arity applications of the `/`
/// operator. (For the purposes of completeness, applications of `/`
/// with argument count not equal to 2 are permitted, though such
/// applications make little sense in practice)
///
/// Every expression can be interpreted as a `Term`, even if such an
/// interpretation is trivial (i.e. the denominator is empty and the
/// numerator contains one term). This interpretation is realized with
/// the [`TermParser`][super::parser::TermParser] struct. The opposite
/// mapping (from `Term` back to `Expr`) is realized by a `From<Term>
/// for Expr` impl. Note that the two are NOT inverses of each other,
/// as `Term::parse_expr` can lose information about nested
/// denominators as it attempts to automatically simplify rational
/// expressions.
#[derive(Debug, Clone, PartialEq)]
pub struct Term {
  pub(super) numerator: Vec<Expr>,
  pub(super) denominator: Vec<Expr>,
}

impl Term {
  pub fn singleton(expr: Expr) -> Self {
    Self {
      numerator: vec![expr],
      denominator: Vec::new(),
    }
  }

  pub fn numerator(&self) -> &[Expr] {
    &self.numerator
  }

  pub fn denominator(&self) -> &[Expr] {
    &self.denominator
  }

  pub fn into_parts(self) -> (Vec<Expr>, Vec<Expr>) {
    (self.numerator, self.denominator)
  }

  pub fn into_numerator(self) -> Vec<Expr> {
    self.numerator
  }

  pub fn into_denominator(self) -> Vec<Expr> {
    self.denominator
  }

  pub fn filter_factors<F>(mut self, mut f: F) -> Self
  where F: FnMut(&Expr) -> bool {
    self.numerator.retain(&mut f);
    self.denominator.retain(&mut f);
    self
  }

  pub fn partition_factors<F>(self, mut f: F) -> (Self, Self)
  where F: FnMut(&Expr) -> bool {
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
}

impl From<Term> for Expr {
  fn from(mut t: Term) -> Expr {
    let numerator =
      if t.numerator.is_empty() {
        Expr::one()
      } else if t.numerator.len() == 1 {
        t.numerator.swap_remove(0)
      } else {
        Expr::call("*", t.numerator)
      };
    if t.denominator.is_empty() {
      numerator
    } else if t.denominator.len() == 1 {
      Expr::call("/", vec![numerator, t.denominator.swap_remove(0)])
    } else {
      Expr::call("/", vec![numerator, Expr::call("*", t.denominator)])
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

fn fmt_product(f: &mut Formatter, exprs: &[Expr]) -> fmt::Result {
  if exprs.is_empty() {
    write!(f, "1")?;
    return Ok(());
  }
  let mut first = true;
  for expr in exprs {
    if !first {
      write!(f, " ")?;
    } else {
      first = false;
    }
    write!(f, "{}", expr)?;
  }
  Ok(())
}

impl From<Number> for Term {
  fn from(n: Number) -> Self {
    // We use the default parser here. Numbers are clearly atomic, so
    // the parser type doesn't matter.
    let parser = TermParser::new();
    parser.parse(Expr::from(n))
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
    self.numerator().is_empty() && self.denominator().is_empty()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_recip() {
    let term = Term {
      numerator: vec![Expr::from(0), Expr::from(1)],
      denominator: vec![Expr::from(2), Expr::from(3)],
    };
    assert_eq!(term.recip(), Term {
      numerator: vec![Expr::from(2), Expr::from(3)],
      denominator: vec![Expr::from(0), Expr::from(1)],
    });
  }
}
