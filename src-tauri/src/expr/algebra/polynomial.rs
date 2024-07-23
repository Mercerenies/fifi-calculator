
use super::term::{TermParser, Sign, SignedTerm};
use crate::expr::Expr;

use num::Zero;

use std::fmt::{self, Formatter, Display};
use std::ops::{Add, Sub, AddAssign, SubAssign, Neg, Mul};

/// A polynomial is a sum of several signed terms.
#[derive(Debug, Clone, PartialEq)]
pub struct Polynomial {
  terms: Vec<SignedTerm>,
}

impl Polynomial {
  pub fn new(terms: impl IntoIterator<Item = SignedTerm>) -> Self {
    Self { terms: terms.into_iter().collect() }
  }

  pub fn terms(&self) -> &[SignedTerm] {
    &self.terms
  }

  pub fn into_terms(self) -> Vec<SignedTerm> {
    self.terms
  }

  pub fn len(&self) -> usize {
    self.terms.len()
  }

  pub fn is_empty(&self) -> bool {
    self.terms.is_empty()
  }
}

pub fn parse_polynomial(term_parser: &TermParser, expr: Expr) -> Polynomial {
  match expr {
    Expr::Call(function_name, args) => {
      match function_name.as_ref() {
        "+" => {
          args.into_iter()
            .map(|arg| parse_polynomial(term_parser, arg))
            .fold(Polynomial::zero(), |a, b| a + b)
        }
        "-" if args.len() == 2 => {
          let [left, right] = args.try_into().unwrap();
          let left = parse_polynomial(term_parser, left);
          let right = parse_polynomial(term_parser, right);
          left - right
        }
        "negate" if args.len() == 1 => {
          let [arg] = args.try_into().unwrap();
          - parse_polynomial(term_parser, arg)
        }
        _ => {
          // Unknown function application, parse as Term
          let term = term_parser.parse(Expr::Call(function_name, args));
          Polynomial { terms: vec![SignedTerm::new(Sign::Positive, term)] }
        }
      }
    }
    expr => {
      // Atomic expression, parse as Term.
      let term = term_parser.parse(expr);
      Polynomial { terms: vec![SignedTerm::new(Sign::Positive, term)] }
    }
  }
}

impl Display for Polynomial {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    for (i, term) in self.terms.iter().enumerate() {
      if i > 0 {
        write!(f, " ")?;
      }
      term.fmt(f)?;
    }
    Ok(())
  }
}

impl From<Polynomial> for Expr {
  fn from(p: Polynomial) -> Self {
    fn plus(a: Expr, b: Expr) -> Expr {
      match a {
        Expr::Call(name, mut args) if name == "+" => {
          args.push(b);
          Expr::call("+", args)
        }
        a => {
          Expr::call("+", vec![a, b])
        }
      }
    }
    fn minus(a: Expr, b: Expr) -> Expr {
      Expr::call("-", vec![a, b])
    }

    if p.terms.is_empty() {
      Expr::zero()
    } else {
      let mut iter = p.terms.into_iter();
      let first = Expr::from(iter.next().unwrap()); // unwrap: We just checked that it was non-empty.
      iter.fold(first, |a, b| {
        match b.sign {
          Sign::Positive => plus(a, Expr::from(b.term)),
          Sign::Negative => minus(a, Expr::from(b.term)),
        }
      })
    }
  }
}

impl Mul<Sign> for Polynomial {
  type Output = Polynomial;

  fn mul(self, rhs: Sign) -> Self::Output {
    if rhs == Sign::Positive {
      self
    } else {
      Polynomial { terms: self.terms.into_iter().map(|t| -t).collect() }
    }
  }
}

impl Neg for Polynomial {
  type Output = Polynomial;

  fn neg(mut self) -> Self::Output {
    self.terms = self.terms.into_iter().map(|t| -t).collect();
    self
  }
}

impl AddAssign for Polynomial {
  fn add_assign(&mut self, rhs: Self) {
    self.terms.extend(rhs.terms);
  }
}

impl SubAssign for Polynomial {
  fn sub_assign(&mut self, rhs: Self) {
    self.terms.extend(rhs.terms.into_iter().map(|t| -t));
  }
}

impl Add for Polynomial {
  type Output = Polynomial;

  fn add(mut self, rhs: Self) -> Self::Output {
    self += rhs;
    self
  }
}

impl Sub for Polynomial {
  type Output = Polynomial;

  fn sub(mut self, rhs: Self) -> Self::Output {
    self -= rhs;
    self
  }
}

impl Zero for Polynomial {
  fn zero() -> Self {
    Self { terms: Vec::new() }
  }

  fn is_zero(&self) -> bool {
    self.terms.is_empty()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_atom_parse() {
    let term_parser = TermParser::new();
    let expr = Expr::from(99);
    assert_eq!(
      parse_polynomial(&term_parser, expr),
      Polynomial { terms: vec![
        SignedTerm::new(Sign::Positive, term_parser.from_parts([Expr::from(99)], [])),
      ] },
    );
  }

  #[test]
  fn test_negation_parse() {
    let term_parser = TermParser::new();
    let expr = Expr::call("negate", vec![Expr::from(99)]);
    assert_eq!(
      parse_polynomial(&term_parser, expr),
      Polynomial { terms: vec![
        SignedTerm::new(Sign::Negative, term_parser.from_parts([Expr::from(99)], [])),
      ] },
    );
  }

  #[test]
  fn test_sum_parse() {
    let term_parser = TermParser::new();
    let expr = Expr::call("+", vec![Expr::from(10), Expr::from(20), Expr::from(30)]);
    assert_eq!(
      parse_polynomial(&term_parser, expr),
      Polynomial { terms: vec![
        SignedTerm::new(Sign::Positive, term_parser.from_parts([Expr::from(10)], [])),
        SignedTerm::new(Sign::Positive, term_parser.from_parts([Expr::from(20)], [])),
        SignedTerm::new(Sign::Positive, term_parser.from_parts([Expr::from(30)], [])),
      ] },
    );
  }

  #[test]
  fn test_difference_parse() {
    let term_parser = TermParser::new();
    let expr = Expr::call("-", vec![Expr::from(10), Expr::from(20)]);
    assert_eq!(
      parse_polynomial(&term_parser, expr),
      Polynomial { terms: vec![
        SignedTerm::new(Sign::Positive, term_parser.from_parts([Expr::from(10)], [])),
        SignedTerm::new(Sign::Negative, term_parser.from_parts([Expr::from(20)], [])),
      ] },
    );
  }

  #[test]
  fn test_mixed_parse() {
    let term_parser = TermParser::new();
    let expr = Expr::call("+", vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("-", vec![Expr::from(30), Expr::from(40)]),
      Expr::from(50),
      Expr::call("-", vec![
        Expr::from(60),
        Expr::call("+", vec![Expr::from(70), Expr::from(80)]),
      ]),
    ]);
    assert_eq!(
      parse_polynomial(&term_parser, expr),
      Polynomial { terms: vec![
        SignedTerm::new(Sign::Positive, term_parser.from_parts([Expr::from(10)], [])),
        SignedTerm::new(Sign::Positive, term_parser.from_parts([Expr::from(20)], [])),
        SignedTerm::new(Sign::Positive, term_parser.from_parts([Expr::from(30)], [])),
        SignedTerm::new(Sign::Negative, term_parser.from_parts([Expr::from(40)], [])),
        SignedTerm::new(Sign::Positive, term_parser.from_parts([Expr::from(50)], [])),
        SignedTerm::new(Sign::Positive, term_parser.from_parts([Expr::from(60)], [])),
        SignedTerm::new(Sign::Negative, term_parser.from_parts([Expr::from(70)], [])),
        SignedTerm::new(Sign::Negative, term_parser.from_parts([Expr::from(80)], [])),
      ] },
    );
  }

  #[test]
  fn test_mixed_parse_back_into_expr() {
    let term_parser = TermParser::new();
    let expr = Expr::call("+", vec![
      Expr::from(10),
      Expr::from(20),
      Expr::call("-", vec![Expr::from(30), Expr::from(40)]),
      Expr::from(50),
      Expr::call("-", vec![
        Expr::from(60),
        Expr::call("+", vec![Expr::from(70), Expr::from(80)]),
      ]),
    ]);
    assert_eq!(
      Expr::from(parse_polynomial(&term_parser, expr)),
      Expr::call("-", vec![
        Expr::call("-", vec![
          Expr::call("+", vec![
            Expr::call("-", vec![
              Expr::call("+", vec![
                Expr::from(10),
                Expr::from(20),
                Expr::from(30),
              ]),
              Expr::from(40),
            ]),
            Expr::from(50),
            Expr::from(60),
          ]),
          Expr::from(70),
        ]),
        Expr::from(80),
      ]),
    );
  }
}
