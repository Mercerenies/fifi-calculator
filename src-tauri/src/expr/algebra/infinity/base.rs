
use crate::expr::Expr;
use super::{INFINITY_NAME, UNDIRECTED_INFINITY_NAME, NAN_NAME};

use std::fmt::{self, Display, Formatter};
use std::ops::{Add, Sub, Mul, MulAssign, Div, Neg};

/// A limit value on the bounds of the usual real line (or complex
/// plane).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfiniteConstant {
  /// Positive infinity, greater than any finite real value.
  PosInfinity,
  /// Negative infinity, less than any finite real value.
  NegInfinity,
  /// An infinite quantity whose direction cannot be determined.
  UndirInfinity,
  /// An unknown quantity, usually resulting from an undeterminate
  /// form.
  ///
  /// Note: Unlike the IEEE 754 constant of the same name, this
  /// constant DOES compare equal to itself. The type
  /// [`InfiniteConstant`] correctly implements both `PartialEq` and
  /// `Eq`.
  NotANumber,
}

impl InfiniteConstant {
  pub const ALL: [InfiniteConstant; 4] = [
    InfiniteConstant::PosInfinity,
    InfiniteConstant::NegInfinity,
    InfiniteConstant::UndirInfinity,
    InfiniteConstant::NotANumber,
  ];

  pub fn abs(&self) -> InfiniteConstant {
    if self == &InfiniteConstant::NotANumber {
      InfiniteConstant::NotANumber
    } else {
      InfiniteConstant::PosInfinity
    }
  }
}

impl<'a> From<&'a InfiniteConstant> for Expr {
  fn from(c: &'a InfiniteConstant) -> Self {
    match c {
      InfiniteConstant::PosInfinity => Expr::var(INFINITY_NAME).unwrap(),
      InfiniteConstant::NegInfinity => Expr::call("negate", vec![Expr::var(INFINITY_NAME).unwrap()]),
      InfiniteConstant::UndirInfinity => Expr::var(UNDIRECTED_INFINITY_NAME).unwrap(),
      InfiniteConstant::NotANumber => Expr::var(NAN_NAME).unwrap(),
    }
  }
}

impl From<InfiniteConstant> for Expr {
  fn from(c: InfiniteConstant) -> Self {
    Expr::from(&c)
  }
}

impl Display for InfiniteConstant {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      InfiniteConstant::PosInfinity => write!(f, "inf"),
      InfiniteConstant::NegInfinity => write!(f, "-inf"),
      InfiniteConstant::UndirInfinity => write!(f, "uinf"),
      InfiniteConstant::NotANumber => write!(f, "nan"),
    }
  }
}

impl Add for InfiniteConstant {
  type Output = InfiniteConstant;

  fn add(self, other: InfiniteConstant) -> InfiniteConstant {
    use InfiniteConstant::*;
    match (self, other) {
      (NotANumber, _) | (_, NotANumber) => NotANumber,
      (UndirInfinity, _) | (_, UndirInfinity) => UndirInfinity,
      (PosInfinity, NegInfinity) | (NegInfinity, PosInfinity) => NotANumber,
      (PosInfinity, PosInfinity) => PosInfinity,
      (NegInfinity, NegInfinity) => PosInfinity,
    }
  }
}

impl Neg for InfiniteConstant {
  type Output = InfiniteConstant;

  fn neg(self) -> InfiniteConstant {
    use InfiniteConstant::*;
    match self {
      NotANumber => NotANumber,
      PosInfinity => NegInfinity,
      NegInfinity => PosInfinity,
      UndirInfinity => UndirInfinity,
    }
  }
}

impl Sub for InfiniteConstant {
  type Output = InfiniteConstant;

  fn sub(self, other: InfiniteConstant) -> InfiniteConstant {
    self + -other
  }
}

impl Mul for InfiniteConstant {
  type Output = InfiniteConstant;

  fn mul(self, other: InfiniteConstant) -> InfiniteConstant {
    use InfiniteConstant::*;
    match (self, other) {
      (NotANumber, _) | (_, NotANumber) => NotANumber,
      (UndirInfinity, _) | (_, UndirInfinity) => UndirInfinity,
      (PosInfinity, NegInfinity) | (NegInfinity, PosInfinity) => NegInfinity,
      (PosInfinity, PosInfinity) | (NegInfinity, NegInfinity) => PosInfinity,
    }
  }
}

impl MulAssign for InfiniteConstant {
  fn mul_assign(&mut self, other: InfiniteConstant) {
    *self = *self * other
  }
}

/// Trivial implementation of `Div`. Since we can't compare the
/// relative magnitudes of two infinities, we always get `NotANumber`.
impl Div for InfiniteConstant {
  type Output = InfiniteConstant;

  fn div(self, _: InfiniteConstant) -> InfiniteConstant {
    InfiniteConstant::NotANumber
  }
}

