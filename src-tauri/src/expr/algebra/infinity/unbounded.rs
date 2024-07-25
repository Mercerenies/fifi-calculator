
use crate::expr::{Expr, TryFromExprError};
use crate::expr::number::Number;
use crate::util::prism::Prism;
use super::prisms::expr_to_unbounded_number;
use super::signed::SignedInfinity;

use std::convert::TryFrom;
use std::cmp::Ordering;

/// Either a finite real value or a signed infinity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnboundedNumber {
  Finite(Number),
  Infinite(SignedInfinity),
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
