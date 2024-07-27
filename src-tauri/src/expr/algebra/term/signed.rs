
use super::base::Term;
use crate::expr::Expr;
use crate::util::Sign;

use std::fmt::{self, Formatter, Display};
use std::ops::{Mul, Div, Neg};

/// A `SignedTerm` is a [`Term`] together with a [`Sign`].
#[derive(Debug, Clone, PartialEq)]
pub struct SignedTerm {
  pub sign: Sign,
  pub term: Term,
}

impl SignedTerm {
  pub fn new(sign: Sign, term: Term) -> Self {
    Self { sign, term }
  }

  pub fn recip(self) -> Self {
    Self {
      sign: self.sign,
      term: self.term.recip(),
    }
  }
}

impl Display for SignedTerm {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "{} {}", self.sign, self.term)
  }
}

impl Mul for SignedTerm {
  type Output = Self;

  fn mul(self, other: Self) -> Self::Output {
    Self {
      sign: self.sign * other.sign,
      term: self.term * other.term,
    }
  }
}

impl Div for SignedTerm {
  type Output = Self;

  fn div(self, other: Self) -> Self::Output {
    Self {
      sign: self.sign * other.sign,
      term: self.term / other.term,
    }
  }
}

impl From<SignedTerm> for Expr {
  fn from(signed_term: SignedTerm) -> Self {
    let expr = Expr::from(signed_term.term);
    match signed_term.sign {
      Sign::Negative => Expr::call("negate", vec![expr]),
      Sign::Positive => expr,
    }
  }
}

impl Neg for SignedTerm {
  type Output = Self;

  fn neg(self) -> Self::Output {
    Self {
      sign: self.sign.other(),
      term: self.term,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::expr::algebra::term::parser::TermParser;
  use crate::expr::Expr;

  #[test]
  fn test_recip() {
    let term_parser = TermParser::new();
    let signed_term = SignedTerm::new(Sign::Positive, term_parser.from_parts([Expr::from(10)], [Expr::from(20)]));
    assert_eq!(
      signed_term.recip(),
      SignedTerm::new(Sign::Positive, term_parser.from_parts([Expr::from(20)], [Expr::from(10)])),
    );
    let signed_term = SignedTerm::new(Sign::Negative, term_parser.from_parts([Expr::from(10)], [Expr::from(20)]));
    assert_eq!(
      signed_term.recip(),
      SignedTerm::new(Sign::Negative, term_parser.from_parts([Expr::from(20)], [Expr::from(10)])),
    );
  }

  #[test]
  fn test_mul() {
    let term_parser = TermParser::new();

    let term1 = SignedTerm::new(Sign::Positive, term_parser.from_parts([Expr::from(10)], [Expr::from(20)]));
    let term2 = SignedTerm::new(Sign::Negative, term_parser.from_parts([Expr::from(30)], [Expr::from(40)]));
    assert_eq!(
      term1 * term2,
      SignedTerm::new(Sign::Negative, term_parser.from_parts(
        [Expr::from(10), Expr::from(30)],
        [Expr::from(20), Expr::from(40)],
      )),
    );

    let term1 = SignedTerm::new(Sign::Negative, term_parser.from_parts([Expr::from(10)], [Expr::from(20)]));
    let term2 = SignedTerm::new(Sign::Negative, term_parser.from_parts([Expr::from(30)], [Expr::from(40)]));
    assert_eq!(
      term1 * term2,
      SignedTerm::new(Sign::Positive, term_parser.from_parts(
        [Expr::from(10), Expr::from(30)],
        [Expr::from(20), Expr::from(40)],
      )),
    );
  }

  #[test]
  fn test_div() {
    let term_parser = TermParser::new();

    let term1 = SignedTerm::new(Sign::Positive, term_parser.from_parts([Expr::from(10)], [Expr::from(20)]));
    let term2 = SignedTerm::new(Sign::Negative, term_parser.from_parts([Expr::from(30)], [Expr::from(40)]));
    assert_eq!(
      term1 / term2,
      SignedTerm::new(Sign::Negative, term_parser.from_parts(
        [Expr::from(10), Expr::from(40)],
        [Expr::from(20), Expr::from(30)],
      )),
    );

    let term1 = SignedTerm::new(Sign::Negative, term_parser.from_parts([Expr::from(10)], [Expr::from(20)]));
    let term2 = SignedTerm::new(Sign::Negative, term_parser.from_parts([Expr::from(30)], [Expr::from(40)]));
    assert_eq!(
      term1 / term2,
      SignedTerm::new(Sign::Positive, term_parser.from_parts(
        [Expr::from(10), Expr::from(40)],
        [Expr::from(20), Expr::from(30)],
      )),
    );
  }
}
