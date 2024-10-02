
use std::fmt::{self, Debug, Formatter};

/// Trait for defining a stricter notion of equality than the usual
/// [`PartialEq`].
///
/// Like `PartialEq`, the equality relation defined here should be a
/// partial equivalence relation. That is, it should be symmetric and
/// transitive. Further, `a.strict_eq(b)` should imply `a == b`.
pub trait StrictEq: PartialEq {
  fn strict_eq(&self, other: &Self) -> bool;
}

/// Lifts a [`StrictEq`] relation into `PartialEq` for use with macros
/// like `assert_eq!`.
///
/// The `Debug` impl for `Strictly<'a, T>` prints equivalently to a
/// simple `T`, to make debug output prettier.
pub struct Strictly<'a, T>(pub &'a T);

impl<'a, T: StrictEq> PartialEq for Strictly<'a, T> {
  fn eq(&self, other: &Self) -> bool {
    self.0.strict_eq(other.0)
  }
}

impl<'a, T: Debug> Debug for Strictly<'a, T> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "{:?}", self.0)
  }
}

impl<T: StrictEq> StrictEq for Vec<T> {
  fn strict_eq(&self, other: &Self) -> bool {
    if self.len() != other.len() {
      return false;
    }
    self.iter().zip(other).all(|(a, b)| a.strict_eq(b))
  }
}

#[macro_export]
macro_rules! assert_strict_eq {
  ($left:expr, $right:expr $(,)?) => {
    match (&$left, &$right) {
      (left_val, right_val) => {
        assert_eq!(
          $crate::util::stricteq::Strictly(left_val),
          $crate::util::stricteq::Strictly(right_val),
        )
      }
    }
  }
}

#[macro_export]
macro_rules! assert_strict_ne {
  ($left:expr, $right:expr $(,)?) => {
    match (&$left, &$right) {
      (left_val, right_val) => {
        assert_ne!(
          $crate::util::stricteq::Strictly(left_val),
          $crate::util::stricteq::Strictly(right_val),
        )
      }
    }
  }
}
