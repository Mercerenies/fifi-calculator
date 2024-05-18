
/// The different ways a number can be represented. These are ordered
/// in terms of priority, so if `a <= b`, that implies that the
/// arithmetic system here will try to use representation `a` before
/// resorting to representation `b`. For instance, `Integer <= Float`
/// implies that we will try to use integer arithmetic and only resort
/// to floating-point values when necessary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NumberRepr {
  /// An integer, exact.
  Integer,
  /// A rational number, exact.
  Ratio,
  /// An inexact IEEE 754 floating-point value.
  Float,
}

impl NumberRepr {
  /// Returns true if the numerical representation represents exact
  /// known quantities, as opposed to approximations.
  pub fn is_exact(&self) -> bool {
    match self {
      NumberRepr::Integer => true,
      NumberRepr::Ratio => true,
      NumberRepr::Float => false,
    }
  }
}

