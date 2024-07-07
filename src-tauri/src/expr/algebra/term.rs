
use crate::expr::Expr;
use crate::expr::number::Number;

use num::One;

use std::ops::{Mul, Div, MulAssign, DivAssign};

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
/// the [`Term::parse_expr`] function. The opposite mapping (from
/// `Term` back to `Expr`) is realized by a `From<Term> for Expr`
/// impl. Note that the two are NOT inverses of each other, as
/// `Term::parse_expr` can lose information about nested denominators
/// as it attempts to automatically simplify rational expressions.
///
/// Note also that, for the purposes of the `Term` type,
/// multiplication is assumed to be commutative. That is, this
/// structure does NOT make sense if matrices, quaternions, or other
/// non-commutative rings may be involved.
#[derive(Debug, Clone, PartialEq)]
pub struct Term {
  numerator: Vec<Expr>,
  denominator: Vec<Expr>,
}

impl Term {
  pub fn new(numerator: Vec<Expr>, denominator: Vec<Expr>) -> Term {
    let numerator = numerator.into_iter()
      .map(Term::parse_expr)
      .fold(Term::one(), |acc, x| acc * x);
    let denominator = denominator.into_iter()
      .map(Term::parse_expr)
      .fold(Term::one(), |acc, x| acc * x);
    numerator / denominator
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

  pub fn recip(self) -> Self {
    Term {
      numerator: self.denominator,
      denominator: self.numerator,
    }
  }

  pub fn parse_expr(expr: Expr) -> Term {
    match expr {
      Expr::Call(function_name, args) => {
        match function_name.as_ref() {
          "*" => {
            args.into_iter()
              .map(Term::parse_expr)
              .fold(Term::one(), |acc, x| acc * x)
          }
          "/" if args.len() == 2 => {
            let [numerator, denominator] = args.try_into().unwrap();
            let numerator = Term::parse_expr(numerator);
            let denominator = Term::parse_expr(denominator);
            numerator / denominator
          }
          _ => {
            // Unknown function application, return a trivial Term.
            Term {
              numerator: vec![Expr::Call(function_name, args)],
              denominator: Vec::new(),
            }
          }
        }
      }
      expr => {
        // Atomic expression, return a trivial Term.
        Term {
          numerator: vec![expr],
          denominator: Vec::new(),
        }
      }
    }
  }
}

impl From<Term> for Expr {
  fn from(t: Term) -> Expr {
    let numerator =
      if t.numerator.is_empty() {
        Expr::one()
      } else {
        Expr::call("*", t.numerator)
      };
    if t.denominator.is_empty() {
      numerator
    } else {
      Expr::call("/", vec![numerator, Expr::call("*", t.denominator)])
    }
  }
}

impl From<Number> for Term {
  fn from(n: Number) -> Self {
    Term::parse_expr(Expr::from(n))
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

  #[test]
  fn test_parse_simple_expr() {
    let expr = Expr::call("+", vec![Expr::from(0), Expr::from(10)]);
    let term = Term::parse_expr(expr);
    assert_eq!(term, Term {
      numerator: vec![Expr::call("+", vec![Expr::from(0), Expr::from(10)])],
      denominator: Vec::new(),
    });

    let expr = Expr::call("*", vec![Expr::from(0), Expr::from(10)]);
    let term = Term::parse_expr(expr);
    assert_eq!(term, Term {
      numerator: vec![Expr::from(0), Expr::from(10)],
      denominator: Vec::new(),
    });

    let expr = Expr::call("/", vec![Expr::from(0), Expr::from(10)]);
    let term = Term::parse_expr(expr);
    assert_eq!(term, Term {
      numerator: vec![Expr::from(0)],
      denominator: vec![Expr::from(10)],
    });
  }

  #[test]
  fn test_parse_expr_with_bad_division_arity() {
    let expr = Expr::call("/", vec![Expr::from(0), Expr::from(10), Expr::from(15)]);
    let term = Term::parse_expr(expr);
    assert_eq!(term, Term {
      numerator: vec![Expr::call("/", vec![Expr::from(0), Expr::from(10), Expr::from(15)])],
      denominator: Vec::new(),
    });
  }

  #[test]
  fn test_parse_nested_division() {
    let expr = Expr::call("/", vec![
      Expr::call("/", vec![Expr::from(0), Expr::from(10)]),
      Expr::call("/", vec![Expr::from(100), Expr::from(110)]),
    ]);
    let term = Term::parse_expr(expr);
    assert_eq!(term, Term {
      numerator: vec![Expr::from(0), Expr::from(110)],
      denominator: vec![Expr::from(10), Expr::from(100)],
    });
  }

  #[test]
  fn test_parse_nested_multiplication() {
    let expr = Expr::call("*", vec![
      Expr::call("*", vec![Expr::from(0), Expr::from(10)]),
      Expr::call("*", vec![Expr::from(100), Expr::from(110)]),
    ]);
    let term = Term::parse_expr(expr);
    assert_eq!(term, Term {
      numerator: vec![Expr::from(0), Expr::from(10), Expr::from(100), Expr::from(110)],
      denominator: Vec::new(),
    });
  }

  #[test]
  fn test_parse_nested_mixed_ops_1() {
    let expr = Expr::call("/", vec![
      Expr::call("*", vec![Expr::from(0), Expr::from(10)]),
      Expr::call("*", vec![Expr::from(100), Expr::from(110)]),
    ]);
    let term = Term::parse_expr(expr);
    assert_eq!(term, Term {
      numerator: vec![Expr::from(0), Expr::from(10)],
      denominator: vec![Expr::from(100), Expr::from(110)],
    });
  }

  #[test]
  fn test_parse_nested_mixed_ops_2() {
    let expr = Expr::call("*", vec![
      Expr::call("/", vec![Expr::from(0), Expr::from(10)]),
      Expr::call("/", vec![Expr::from(100), Expr::from(110)]),
    ]);
    let term = Term::parse_expr(expr);
    assert_eq!(term, Term {
      numerator: vec![Expr::from(0), Expr::from(100)],
      denominator: vec![Expr::from(10), Expr::from(110)],
    });
  }
}
