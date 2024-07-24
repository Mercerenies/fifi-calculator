
mod prisms;

use crate::expr::Expr;

use std::fmt::{self, Display, Formatter};

pub const INFINITY_NAME: &str = "inf";
pub const UNDIRECTED_INFINITY_NAME: &str = "uinf";
pub const NAN_NAME: &str = "nan";

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
