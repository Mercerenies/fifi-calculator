
pub use super::base::InfiniteConstant;
use crate::util::prism::ErrorWithPayload;
use crate::expr::number::Number;
use crate::expr::Expr;

use thiserror::Error;

use std::ops::{Neg, Mul};
use std::cmp::Ordering;

/// An infinity value with a known sign.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SignedInfinity {
  NegInfinity,
  PosInfinity,
}

#[derive(Debug, Clone, Error)]
#[error("Expected signed infinity")]
pub struct ExpectedSignedInfinityError {
  original_constant: InfiniteConstant,
}

impl From<SignedInfinity> for InfiniteConstant {
  fn from(s: SignedInfinity) -> InfiniteConstant {
    match s {
      SignedInfinity::NegInfinity => InfiniteConstant::NegInfinity,
      SignedInfinity::PosInfinity => InfiniteConstant::PosInfinity,
    }
  }
}

impl TryFrom<InfiniteConstant> for SignedInfinity {
  type Error = ExpectedSignedInfinityError;

  fn try_from(c: InfiniteConstant) -> Result<SignedInfinity, ExpectedSignedInfinityError> {
    match c {
      InfiniteConstant::NegInfinity => Ok(SignedInfinity::NegInfinity),
      InfiniteConstant::PosInfinity => Ok(SignedInfinity::PosInfinity),
      _ => Err(ExpectedSignedInfinityError { original_constant: c }),
    }
  }
}

impl ErrorWithPayload<InfiniteConstant> for ExpectedSignedInfinityError {
  fn recover_payload(self) -> InfiniteConstant {
    self.original_constant
  }
}

impl PartialEq<Number> for SignedInfinity {
  fn eq(&self, _: &Number) -> bool {
    false
  }
}

impl PartialOrd<Number> for SignedInfinity {
  fn partial_cmp(&self, _other: &Number) -> Option<Ordering> {
    match self {
      SignedInfinity::NegInfinity => Some(Ordering::Less),
      SignedInfinity::PosInfinity => Some(Ordering::Greater),
    }
  }
}

impl PartialEq<SignedInfinity> for Number {
  fn eq(&self, _: &SignedInfinity) -> bool {
    false
  }
}

impl PartialOrd<SignedInfinity> for Number {
  fn partial_cmp(&self, other: &SignedInfinity) -> Option<Ordering> {
    other.partial_cmp(self).map(Ordering::reverse)
  }
}

impl From<SignedInfinity> for Expr {
  fn from(c: SignedInfinity) -> Self {
    Expr::from(InfiniteConstant::from(c))
  }
}

impl Neg for SignedInfinity {
  type Output = Self;

  fn neg(self) -> Self::Output {
    match self {
      SignedInfinity::NegInfinity => SignedInfinity::PosInfinity,
      SignedInfinity::PosInfinity => SignedInfinity::NegInfinity,
    }
  }
}

impl Mul for SignedInfinity {
  type Output = Self;

  fn mul(self, other: Self) -> Self::Output {
    if self == other {
      SignedInfinity::PosInfinity
    } else {
      SignedInfinity::NegInfinity
    }
  }
}
