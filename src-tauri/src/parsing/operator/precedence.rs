
/// The precedence of an operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Precedence(u64);

impl Precedence {
  pub const MIN: Precedence = Precedence(0);
  pub const MAX: Precedence = Precedence(u64::MAX);

  /// Internally, we store an operator's precedence as ten times the
  /// input value, so that we can increment or decrement to represent
  /// associativity.
  ///
  /// For example, if `#` is a left-associative operator with
  /// (internal) precedence value `p`, then its left-hand side is also
  /// at precedence value `p`, while its right-hand side is at
  /// precedence value `p + 1`, indicating parentheses will be
  /// required if `#` is encountered again.
  ///
  /// Use [`from_raw`](Precedence::from_raw) to bypass the
  /// multiplication and construct a `Precedence` value directly.
  pub fn new(n: u64) -> Precedence {
    Precedence(n * 10)
  }

  pub fn from_raw(n: u64) -> Precedence {
    Precedence(n)
  }

  pub fn incremented(self) -> Precedence {
    Precedence(self.0 + 1)
  }
}

impl From<u64> for Precedence {
  fn from(n: u64) -> Precedence {
    Precedence::new(n)
  }
}
