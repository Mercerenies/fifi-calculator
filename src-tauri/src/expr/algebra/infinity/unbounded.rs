
use crate::expr::{Expr, TryFromExprError};
use crate::expr::number::Number;
use crate::util::PreOne;
use crate::util::prism::Prism;
use super::prisms::expr_to_unbounded_number;
use super::signed::SignedInfinity;

use num::{Zero, One};
use try_traits::ops::{TryAdd, TrySub, TryMul};
use thiserror::Error;

use std::convert::TryFrom;
use std::cmp::Ordering;
use std::ops::{Neg, Div};

/// Either a finite real value or a signed infinity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnboundedNumber {
  Finite(Number),
  Infinite(SignedInfinity),
}

#[derive(Debug, Clone, Error)]
#[error("Indeterminate form: {message}")]
pub struct IndeterminateFormError {
  message: &'static str,
}

impl UnboundedNumber {
  pub const POS_INFINITY: Self = UnboundedNumber::Infinite(SignedInfinity::PosInfinity);
  pub const NEG_INFINITY: Self = UnboundedNumber::Infinite(SignedInfinity::NegInfinity);

  /// Returns true if this number is finite.
  pub fn is_finite(&self) -> bool {
    matches!(self, UnboundedNumber::Finite(_))
  }

  /// Returns true if this number is an infinity.
  pub fn is_infinite(&self) -> bool {
    matches!(self, UnboundedNumber::Infinite(_))
  }

  pub fn zero() -> Self {
    UnboundedNumber::Finite(Number::zero())
  }

  pub fn one() -> Self {
    UnboundedNumber::Finite(Number::one())
  }

  pub fn finite(n: impl Into<Number>) -> Self {
    UnboundedNumber::Finite(n.into())
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

impl PreOne for UnboundedNumber {
  fn pre_one() -> Self {
    Self::Finite(Number::one())
  }

  fn is_pre_one(&self) -> bool {
    if let Self::Finite(n) = self {
      n.is_one()
    } else {
      false
    }
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

impl TryAdd for UnboundedNumber {
  type Output = Self;
  type Error = IndeterminateFormError;

  fn try_add(self, other: Self) -> Result<Self, IndeterminateFormError> {
    use UnboundedNumber::*;
    match (self, other) {
      (Infinite(a), Infinite(b)) =>
        if a == b { Ok(Infinite(a)) } else { Err(IndeterminateFormError { message: "inf - inf" }) },
      (Infinite(a), _) => Ok(Infinite(a)),
      (_, Infinite(b)) => Ok(Infinite(b)),
      (Finite(a), Finite(b)) => Ok(Finite(a + b)),
    }
  }
}

impl Default for UnboundedNumber {
  fn default() -> Self {
    UnboundedNumber::Finite(Number::default())
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

impl TrySub for UnboundedNumber {
  type Output = Self;
  type Error = IndeterminateFormError;

  fn try_sub(self, other: Self) -> Result<Self, IndeterminateFormError> {
    self.try_add(-other)
  }
}

impl TryMul for UnboundedNumber {
  type Output = Self;
  type Error = IndeterminateFormError;

  /// Multiplies the two unbounded numbers. Panics on infinity times
  /// zero.
  fn try_mul(self, other: Self) -> Result<Self, IndeterminateFormError> {
    use UnboundedNumber::*;
    match (self, other) {
      (Infinite(a), Infinite(b)) => Ok(Infinite(a * b)),
      (Infinite(a), Finite(b)) => match b.cmp(&Number::zero()) {
        Ordering::Greater => Ok(Infinite(a)),
        Ordering::Less => Ok(Infinite(-a)),
        Ordering::Equal => Err(IndeterminateFormError { message: "inf * 0" }),
      }
      (Finite(a), Infinite(b)) => Infinite(b).try_mul(Finite(a)), // See above case
      (Finite(a), Finite(b)) => Ok(Finite(a * b)),
    }
  }
}

/// Scalar division. Panics on division by zero.
impl Div<Number> for UnboundedNumber {
  type Output = Self;

  fn div(self, other: Number) -> Self {
    assert!(other != Number::zero(), "Division by zero");
    match self {
      UnboundedNumber::Finite(a) => {
        UnboundedNumber::Finite(a / other)
      }
      UnboundedNumber::Infinite(a) => {
        UnboundedNumber::Infinite(
          if other < Number::zero() { - a } else { a },
        )
      }
    }
  }
}
