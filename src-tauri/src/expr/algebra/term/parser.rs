
use super::base::Term;
use crate::expr::Expr;

use num::One;

/// A parser which takes an [`Expr`] and produces a [`Term`]. Such a
/// parser must always succeed.
///
/// Note also that, for the purposes of this parser, multiplication is
/// assumed to be commutative. That is, this structure does NOT make
/// sense if matrices, quaternions, or other non-commutative rings may
/// be involved. We may add a parameter to this structure to permit
/// such customization in the future.
#[derive(Debug, Clone)]
pub struct TermParser {
  _priv: (),
}

impl TermParser {
  pub const fn new() -> Self {
    Self { _priv: () }
  }

  pub fn parse(&self, expr: Expr) -> Term {
    match expr {
      Expr::Call(function_name, args) => {
        match function_name.as_ref() {
          "*" => {
            args.into_iter()
              .map(|arg| self.parse(arg))
              .fold(Term::one(), |acc, x| acc * x)
          }
          "/" if args.len() == 2 => {
            let [numerator, denominator] = args.try_into().unwrap();
            let numerator = self.parse(numerator);
            let denominator = self.parse(denominator);
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

  pub fn from_parts<I1, I2>(&self, numerator: I1, denominator: I2) -> Term
  where I1: IntoIterator<Item=Expr>,
        I2: IntoIterator<Item=Expr> {
    let numerator = numerator.into_iter()
      .map(|expr| self.parse(expr))
      .fold(Term::one(), |acc, x| acc * x);
    let denominator = denominator.into_iter()
      .map(|expr| self.parse(expr))
      .fold(Term::one(), |acc, x| acc * x);
    numerator / denominator
  }
}

#[cfg(test)]
mod tests {
  use super::*;


  #[test]
  fn test_parse_simple_expr() {
    let expr = Expr::call("+", vec![Expr::from(0), Expr::from(10)]);
    let term = TermParser::new().parse(expr);
    assert_eq!(term, Term {
      numerator: vec![Expr::call("+", vec![Expr::from(0), Expr::from(10)])],
      denominator: Vec::new(),
    });

    let expr = Expr::call("*", vec![Expr::from(0), Expr::from(10)]);
    let term = TermParser::new().parse(expr);
    assert_eq!(term, Term {
      numerator: vec![Expr::from(0), Expr::from(10)],
      denominator: Vec::new(),
    });

    let expr = Expr::call("/", vec![Expr::from(0), Expr::from(10)]);
    let term = TermParser::new().parse(expr);
    assert_eq!(term, Term {
      numerator: vec![Expr::from(0)],
      denominator: vec![Expr::from(10)],
    });
  }

  #[test]
  fn test_parse_expr_with_bad_division_arity() {
    let expr = Expr::call("/", vec![Expr::from(0), Expr::from(10), Expr::from(15)]);
    let term = TermParser::new().parse(expr);
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
    let term = TermParser::new().parse(expr);
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
    let term = TermParser::new().parse(expr);
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
    let term = TermParser::new().parse(expr);
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
    let term = TermParser::new().parse(expr);
    assert_eq!(term, Term {
      numerator: vec![Expr::from(0), Expr::from(100)],
      denominator: vec![Expr::from(10), Expr::from(110)],
    });
  }
}
