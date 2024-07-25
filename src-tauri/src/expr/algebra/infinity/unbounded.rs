
use crate::expr::{Expr, TryFromExprError};
use crate::expr::number::Number;
use crate::util::prism::Prism;
use super::prisms::expr_to_unbounded_number;
use super::signed::SignedInfinity;

use num::Zero;

use std::convert::TryFrom;
use std::cmp::Ordering;
use std::ops::{Add, Sub, Mul, Neg};

/// Either a finite real value or a signed infinity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnboundedNumber {
  Finite(Number),
  Infinite(SignedInfinity),
}

impl UnboundedNumber {
  pub const POS_INFINITY: Self = UnboundedNumber::Infinite(SignedInfinity::PosInfinity);
  pub const NEG_INFINITY: Self = UnboundedNumber::Infinite(SignedInfinity::NegInfinity);

  /// Returns true if this number is finite.
  pub fn is_finite(&self) -> bool {
    match self {
      UnboundedNumber::Finite(_) => true,
      _ => false,
    }
  }

  /// Returns true if this number is an infinity.
  pub fn is_infinite(&self) -> bool {
    match self {
      UnboundedNumber::Infinite(_) => true,
      _ => false,
    }
  }
}

impl TryFrom<Expr> for UnboundedNumber {
  type Error = TryFromExprError;

  fn try_from(expr: Expr) -> Result<UnboundedNumber, TryFromExprError> {
    expr_to_unbounded_number().narrow_type(expr)
      .map_err(|expr| TryFromExprError::new("UnboundedNumber", expr))
  }
}

impl PartialOrd for UnboundedNumber {
  fn partial_cmp(&self, other: &UnboundedNumber) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for UnboundedNumber {
  fn cmp(&self, other: &Self) -> Ordering {
    match (self, other) {
      (UnboundedNumber::Finite(a), UnboundedNumber::Finite(b)) => a.cmp(b),
      (UnboundedNumber::Finite(a), UnboundedNumber::Infinite(b)) => a.partial_cmp(b).unwrap(),
      (UnboundedNumber::Infinite(a), UnboundedNumber::Finite(b)) => a.partial_cmp(b).unwrap(),
      (UnboundedNumber::Infinite(a), UnboundedNumber::Infinite(b)) => a.partial_cmp(b).unwrap(),
    }
  }
}

impl From<UnboundedNumber> for Expr {
  fn from(c: UnboundedNumber) -> Self {
    match c {
      UnboundedNumber::Finite(c) => Expr::from(c),
      UnboundedNumber::Infinite(c) => Expr::from(c),
    }
  }
}

impl Add for UnboundedNumber {
  type Output = Self;

  /// Adds two unbounded numbers. Panics if the inputs are infinities
  /// of different signs.
  fn add(self, other: Self) -> Self {
    use UnboundedNumber::*;
    match (self, other) {
      (Infinite(a), Infinite(b)) =>
        if a == b { Infinite(a) } else { panic!("Cannot add infinities of different signs"); },
      (Infinite(a), _) => Infinite(a),
      (_, Infinite(b)) => Infinite(b),
      (Finite(a), Finite(b)) => Finite(a + b),
    }
  }
}

impl Zero for UnboundedNumber {
  fn zero() -> Self {
    UnboundedNumber::Finite(Number::zero())
  }

  fn is_zero(&self) -> bool {
    match self {
      UnboundedNumber::Finite(a) => a.is_zero(),
      _ => false,
    }
  }
}

impl Neg for UnboundedNumber {
  type Output = Self;

  fn neg(self) -> Self {
    match self {
      UnboundedNumber::Finite(a) => UnboundedNumber::Finite(-a),
      UnboundedNumber::Infinite(a) => UnboundedNumber::Infinite(-a),
    }
  }
}

impl Sub for UnboundedNumber {
  type Output = Self;

  /// Subtracts two unbounded numbers. Panics if the inputs are
  /// infinities of the same sign.
  fn sub(self, other: Self) -> Self {
    self + -other
  }
}

impl Mul for UnboundedNumber {
  type Output = Self;

  /// Multiplies the two unbounded numbers. Panics on infinity times
  /// zero.
  fn mul(self, other: Self) -> Self {
    use UnboundedNumber::*;
    match (self, other) {
      (Infinite(a), Infinite(b)) => Infinite(a * b),
      (Infinite(a), Finite(b)) => match b.cmp(&Number::zero()) {
        Ordering::Greater => Infinite(a),
        Ordering::Less => Infinite(-a),
        Ordering::Equal => panic!("Cannot multiply infinity by zero"),
      }
      (Finite(a), Infinite(b)) => Infinite(b) * Finite(a), // See above case
      (Finite(a), Finite(b)) => Finite(a * b),
    }
  }
}
