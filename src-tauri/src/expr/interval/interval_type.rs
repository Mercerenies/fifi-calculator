
use super::bound::BoundType;
use crate::util::brackets::HtmlBrackets;

use thiserror::Error;

use std::cmp::Ordering;

/// The type of interval. Corresponds to the four infix operators
/// representing intervals.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntervalType {
  Closed,
  RightOpen,
  LeftOpen,
  FullOpen,
}

#[derive(Debug, Clone, Error)]
#[error("Error parsing interval type operator")]
pub struct ParseIntervalTypeError {
  _priv: (),
}

impl IntervalType {
  pub fn from_bounds(left: BoundType, right: BoundType) -> Self {
    match (left, right) {
      (BoundType::Inclusive, BoundType::Inclusive) => IntervalType::Closed,
      (BoundType::Inclusive, BoundType::Exclusive) => IntervalType::RightOpen,
      (BoundType::Exclusive, BoundType::Inclusive) => IntervalType::LeftOpen,
      (BoundType::Exclusive, BoundType::Exclusive) => IntervalType::FullOpen,
    }
  }

  pub fn name(self) -> &'static str {
    match self {
      IntervalType::Closed => "..",
      IntervalType::RightOpen => "..^",
      IntervalType::LeftOpen => "^..",
      IntervalType::FullOpen => "^..^",
    }
  }

  pub fn into_bounds(self) -> (BoundType, BoundType) {
    match self {
      IntervalType::Closed => (BoundType::Inclusive, BoundType::Inclusive),
      IntervalType::RightOpen => (BoundType::Inclusive, BoundType::Exclusive),
      IntervalType::LeftOpen => (BoundType::Exclusive, BoundType::Inclusive),
      IntervalType::FullOpen => (BoundType::Exclusive, BoundType::Exclusive),
    }
  }

  pub fn parse(s: &str) -> Result<Self, ParseIntervalTypeError> {
    match s {
      ".." => Ok(IntervalType::Closed),
      "..^" => Ok(IntervalType::RightOpen),
      "^.." => Ok(IntervalType::LeftOpen),
      "^..^" => Ok(IntervalType::FullOpen),
      _ => Err(ParseIntervalTypeError { _priv: () }),
    }
  }

  pub fn is_interval_type(s: &str) -> bool {
    Self::parse(s).is_ok()
  }

  /// Returns the greatest-lower-bound of `self` and `other`,
  /// according to the `PartialOrd` instance for `IntervalType`.
  pub fn min(self, other: IntervalType) -> IntervalType {
    match self.partial_cmp(&other) {
      Some(Ordering::Greater) => other,
      Some(Ordering::Less | Ordering::Equal) => self,
      None => IntervalType::FullOpen,
    }
  }

  /// Returns the least-upper-bound of `self` and `other`,
  /// according to the `PartialOrd` instance for `IntervalType`.
  pub fn max(self, other: IntervalType) -> IntervalType {
    match self.partial_cmp(&other) {
      Some(Ordering::Greater | Ordering::Equal) => self,
      Some(Ordering::Less) => other,
      None => IntervalType::Closed,
    }
  }

  pub fn flipped(self) -> IntervalType {
    match self {
      IntervalType::Closed => IntervalType::Closed,
      IntervalType::RightOpen => IntervalType::LeftOpen,
      IntervalType::LeftOpen => IntervalType::RightOpen,
      IntervalType::FullOpen => IntervalType::FullOpen,
    }
  }

  pub fn includes_left(self) -> bool {
    self == IntervalType::Closed || self == IntervalType::RightOpen
  }

  pub fn includes_right(self) -> bool {
    self == IntervalType::Closed || self == IntervalType::LeftOpen
  }

  pub fn html_brackets(self) -> HtmlBrackets {
    let (left_bound, right_bound) = self.into_bounds();
    HtmlBrackets::non_matching(left_bound.html_bracket_type(), right_bound.html_bracket_type())
  }
}

/// The `PartialOrd` implementation for `IntervalType` is ordered by
/// strictness. The most strict type of ordering is a fully open one,
/// and the least strict is a closed interval. The two half-open
/// interval types are not comparable. That is,
/// `IntervalType::FullOpen` is the least value of this ordering and
/// `IntervalType::Closed` is the greatest value.
impl PartialOrd for IntervalType {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    if self == other {
      Some(Ordering::Equal)
    } else if self == &IntervalType::Closed || other == &IntervalType::FullOpen {
      Some(Ordering::Greater)
    } else if self == &IntervalType::FullOpen || other == &IntervalType::Closed {
      Some(Ordering::Less)
    } else {
      None
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_interval_type_min() {
    use IntervalType::*;
    assert_eq!(Closed.min(Closed), Closed);
    assert_eq!(LeftOpen.min(LeftOpen), LeftOpen);
    assert_eq!(LeftOpen.min(RightOpen), FullOpen);
    assert_eq!(LeftOpen.min(Closed), LeftOpen);
    assert_eq!(FullOpen.min(RightOpen), FullOpen);
    assert_eq!(RightOpen.min(FullOpen), FullOpen);
    assert_eq!(FullOpen.min(FullOpen), FullOpen);
  }

  #[test]
  fn test_interval_type_max() {
    use IntervalType::*;
    assert_eq!(Closed.max(Closed), Closed);
    assert_eq!(LeftOpen.max(LeftOpen), LeftOpen);
    assert_eq!(LeftOpen.max(RightOpen), Closed);
    assert_eq!(LeftOpen.max(Closed), Closed);
    assert_eq!(FullOpen.max(RightOpen), RightOpen);
    assert_eq!(RightOpen.max(FullOpen), RightOpen);
    assert_eq!(FullOpen.max(FullOpen), FullOpen);
  }

  #[test]
  fn test_interval_type_partial_ordering() {
    use IntervalType::*;
    assert!(FullOpen <= Closed);
    assert!(FullOpen <= LeftOpen);
    assert!(FullOpen <= FullOpen);
    assert!(Closed <= Closed);
    assert!(RightOpen <= Closed);
    assert!(LeftOpen <= Closed);
    assert_eq!(LeftOpen.partial_cmp(&RightOpen), None);
  }
}
