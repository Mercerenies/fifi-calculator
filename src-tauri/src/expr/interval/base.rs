
use super::interval_type::IntervalType;
use super::bound::Bounded;
use super::raw::RawInterval;
use crate::expr::Expr;

use num::Zero;

use std::ops::{Add, Sub, Mul, Neg};

/// An interval form consisting of specifically real numbers on the
/// left and right hand sides.
///
/// Intervals are always kept in normal form, which is defined as
/// follows. An interval is in normal form if it is either (a)
/// nonempty or (b) equal to the interval `0..^0`. Put another way,
/// the normal form of the empty interval is `0..^0`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Interval<T> {
  left: T,
  interval_type: IntervalType,
  right: T,
}

impl<T: Zero + Ord> Interval<T> {
  /// Constructs a new interval.
  pub fn new(left: T, interval_type: IntervalType, right: T) -> Self {
    Self { left, interval_type, right }.normalize()
  }

  /// Constructs a new interval from bounds.
  pub fn from_bounds(left: Bounded<T>, right: Bounded<T>) -> Self {
    let interval_type = IntervalType::from_bounds(left.bound_type, right.bound_type);
    Self { left: left.scalar, interval_type, right: right.scalar }.normalize()
  }

  pub fn empty() -> Self {
    Self { left: T::zero(), interval_type: IntervalType::RightOpen, right: T::zero() }
  }

  pub fn is_empty(&self) -> bool {
    self.right < self.left || (self.right == self.left && self.interval_type != IntervalType::Closed)
  }

  pub fn into_bounds(self) -> (Bounded<T>, Bounded<T>) {
    let (left_bound, right_bound) = self.interval_type.into_bounds();
    (
      Bounded { scalar: self.left, bound_type: left_bound },
      Bounded { scalar: self.right, bound_type: right_bound },
    )
  }

  fn normalize(self) -> Self {
    if self.is_empty() {
      // The interval is empty, so represent it as the canonical empty interval.
      Self::empty()
    } else {
      self
    }
  }

  /// Applies a unary, monotone function to this interval to produce a
  /// new interval. It is the caller's responsibility to ensure that
  /// the provided function is monotonic.
  pub fn map_monotone<F, U>(self, f: F) -> Interval<U>
  where F: Fn(T) -> U,
        U: Ord + Zero {
    if self.is_empty() {
      return Interval::empty();
    }
    let (lower, upper) = self.into_bounds();
    Interval::from_bounds(
      lower.map(&f),
      upper.map(f),
    ).normalize()
  }

  /// Applies a binary, monotone function to the two intervals to
  /// produce a new interval. It is the caller's responsibility to
  /// ensure that the provided function is monotonic.
  pub fn apply_monotone<F, S, U>(self, other: Interval<S>, f: F) -> Interval<U>
  where F: Fn(T, S) -> U,
        T: Clone,
        S: Clone + Ord + Zero,
        U: Clone + Ord + Zero {
    if self.is_empty() || other.is_empty() {
      return Interval::empty();
    }

    let (left_lower, left_upper) = self.into_bounds();
    let (right_lower, right_upper) = other.into_bounds();
    let all_combinations = [
      left_lower.clone().apply(right_lower.clone(), &f),
      left_lower.clone().apply(right_upper.clone(), &f),
      left_upper.clone().apply(right_lower, &f),
      left_upper.apply(right_upper, &f),
    ];
    let lower = all_combinations.iter().cloned().reduce(Bounded::min).unwrap(); // unwrap: Non-empty array
    let upper = all_combinations.into_iter().reduce(Bounded::max).unwrap(); // unwrap: Non-empty array
    Interval::from_bounds(lower, upper).normalize()
  }
}

impl<T> Interval<T> {
  pub fn left(&self) -> &T {
    &self.left
  }

  pub fn right(&self) -> &T {
    &self.right
  }

  pub fn into_raw(self) -> RawInterval<T> {
    RawInterval { left: self.left, interval_type: self.interval_type, right: self.right }
  }
}

impl<T> From<Interval<T>> for Expr
where T: Into<Expr> {
  fn from(interval: Interval<T>) -> Expr {
    Expr::from(interval.into_raw())
  }
}

impl<T: Add + Zero + Ord> Add for Interval<T>
where <T as Add>::Output: Zero + Ord {
  type Output = Interval<<T as Add>::Output>;

  fn add(self, other: Self) -> Self::Output {
    if self.is_empty() || other.is_empty() {
      return Interval::empty();
    }
    let interval_type = self.interval_type.min(other.interval_type);
    Interval::new(self.left + other.left, interval_type, self.right + other.right).normalize()
  }
}

impl<T: Sub + Zero + Ord> Sub for Interval<T>
where <T as Sub>::Output: Zero + Ord {
  type Output = Interval<<T as Sub>::Output>;

  fn sub(self, other: Self) -> Self::Output {
    if self.is_empty() || other.is_empty() {
      return Interval::empty();
    }
    let interval_type = self.interval_type.min(other.interval_type.flipped());
    Interval::new(self.left - other.right, interval_type, self.right - other.left).normalize()
  }
}

/// Note: This instance assumes that the multiplication on `T` is a
/// monotone operation with respect to its `Ord` instance.
impl<T> Mul for Interval<T>
where T: Mul + Zero + Ord + Clone,
      <T as Mul>::Output: Zero + Ord + Clone {
  type Output = Interval<<T as Mul>::Output>;

  fn mul(self, other: Self) -> Self::Output {
    self.apply_monotone(other, |x, y| x * y)
  }
}

/// Note: This instance assumes that the `T: Neg` instance is
/// order-reversing with respect to the `T: Ord` instance.
impl<T: Neg> Neg for Interval<T>
where <T as Neg>::Output: Zero + Ord {
  type Output = Interval<<T as Neg>::Output>;

  fn neg(self) -> Self::Output {
    let interval_type = self.interval_type.flipped();
    Interval::new(-self.right, interval_type, -self.left).normalize()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::expr::number::Number;

  #[test]
  fn test_normalize_nonempty_interval() {
    let nonempty_interval = Interval::new(Number::from(0), IntervalType::Closed, Number::from(10));
    assert_eq!(nonempty_interval.clone().normalize(), nonempty_interval);
    let nonempty_interval = Interval::new(Number::from(2), IntervalType::Closed, Number::from(2));
    assert_eq!(nonempty_interval.clone().normalize(), nonempty_interval);
  }

  #[test]
  fn test_normalize_empty_intervals() {
    let interval = Interval::new(Number::from(2), IntervalType::Closed, Number::from(1));
    assert_eq!(interval.normalize(), Interval::empty());
    let interval = Interval::new(Number::from(2), IntervalType::LeftOpen, Number::from(1));
    assert_eq!(interval.normalize(), Interval::empty());
    let interval = Interval::new(Number::from(2), IntervalType::RightOpen, Number::from(1));
    assert_eq!(interval.normalize(), Interval::empty());
    let interval = Interval::new(Number::from(2), IntervalType::FullOpen, Number::from(1));
    assert_eq!(interval.normalize(), Interval::empty());
    let interval = Interval::new(Number::from(2), IntervalType::LeftOpen, Number::from(2));
    assert_eq!(interval.normalize(), Interval::empty());
    let interval = Interval::new(Number::from(2), IntervalType::RightOpen, Number::from(2));
    assert_eq!(interval.normalize(), Interval::empty());
    let interval = Interval::new(Number::from(2), IntervalType::FullOpen, Number::from(2));
    assert_eq!(interval.normalize(), Interval::empty());
  }

  #[test]
  fn test_is_empty_on_nonempty_interval() {
    let nonempty_interval = Interval::new(Number::from(0), IntervalType::Closed, Number::from(10));
    assert!(!nonempty_interval.is_empty());
    let nonempty_interval = Interval::new(Number::from(2), IntervalType::Closed, Number::from(2));
    assert!(!nonempty_interval.is_empty());
  }

  #[test]
  fn test_is_empty_on_empty_intervals() {
    let interval = Interval::new(Number::from(2), IntervalType::Closed, Number::from(1));
    assert!(interval.is_empty());
    let interval = Interval::new(Number::from(2), IntervalType::LeftOpen, Number::from(1));
    assert!(interval.is_empty());
    let interval = Interval::new(Number::from(2), IntervalType::RightOpen, Number::from(1));
    assert!(interval.is_empty());
    let interval = Interval::new(Number::from(2), IntervalType::FullOpen, Number::from(1));
    assert!(interval.is_empty());
    let interval = Interval::new(Number::from(2), IntervalType::LeftOpen, Number::from(2));
    assert!(interval.is_empty());
    let interval = Interval::new(Number::from(2), IntervalType::RightOpen, Number::from(2));
    assert!(interval.is_empty());
    let interval = Interval::new(Number::from(2), IntervalType::FullOpen, Number::from(2));
    assert!(interval.is_empty());
  }

  #[test]
  fn test_add_interval() {
    let interval1 = Interval::new(Number::from(1), IntervalType::Closed, Number::from(2));
    let interval2 = Interval::new(Number::from(10), IntervalType::Closed, Number::from(20));
    assert_eq!(
      interval1 + interval2,
      Interval::new(Number::from(11), IntervalType::Closed, Number::from(22)),
    );
    let interval1 = Interval::new(Number::from(1), IntervalType::LeftOpen, Number::from(2));
    let interval2 = Interval::new(Number::from(10), IntervalType::FullOpen, Number::from(20));
    assert_eq!(
      interval1 + interval2,
      Interval::new(Number::from(11), IntervalType::FullOpen, Number::from(22)),
    );
    let interval1 = Interval::new(Number::from(1), IntervalType::LeftOpen, Number::from(2));
    let interval2 = Interval::new(Number::from(10), IntervalType::RightOpen, Number::from(20));
    assert_eq!(
      interval1 + interval2,
      Interval::new(Number::from(11), IntervalType::FullOpen, Number::from(22)),
    );
    let interval1 = Interval::new(Number::from(1), IntervalType::RightOpen, Number::from(2));
    let interval2 = Interval::new(Number::from(10), IntervalType::RightOpen, Number::from(20));
    assert_eq!(
      interval1 + interval2,
      Interval::new(Number::from(11), IntervalType::RightOpen, Number::from(22)),
    );
  }

  #[test]
  fn test_sub_interval() {
    let interval1 = Interval::new(Number::from(1), IntervalType::Closed, Number::from(2));
    let interval2 = Interval::new(Number::from(10), IntervalType::Closed, Number::from(20));
    assert_eq!(
      interval1 - interval2,
      Interval::new(Number::from(-19), IntervalType::Closed, Number::from(-8)),
    );
    let interval1 = Interval::new(Number::from(1), IntervalType::LeftOpen, Number::from(2));
    let interval2 = Interval::new(Number::from(10), IntervalType::FullOpen, Number::from(20));
    assert_eq!(
      interval1 - interval2,
      Interval::new(Number::from(-19), IntervalType::FullOpen, Number::from(-8)),
    );
    let interval1 = Interval::new(Number::from(1), IntervalType::LeftOpen, Number::from(2));
    let interval2 = Interval::new(Number::from(10), IntervalType::RightOpen, Number::from(20));
    assert_eq!(
      interval1 - interval2,
      Interval::new(Number::from(-19), IntervalType::LeftOpen, Number::from(-8)),
    );
    let interval1 = Interval::new(Number::from(1), IntervalType::RightOpen, Number::from(2));
    let interval2 = Interval::new(Number::from(10), IntervalType::RightOpen, Number::from(20));
    assert_eq!(
      interval1 - interval2,
      Interval::new(Number::from(-19), IntervalType::FullOpen, Number::from(-8)),
    );
  }

  #[test]
  fn test_add_empty_intervals() {
    let interval1 = Interval::empty();
    let interval2 = Interval::new(Number::from(1), IntervalType::Closed, Number::from(2));
    assert_eq!(interval1 + interval2, Interval::empty());
  }

  #[test]
  fn test_sub_empty_intervals() {
    let interval1 = Interval::empty();
    let interval2 = Interval::new(Number::from(1), IntervalType::Closed, Number::from(2));
    assert_eq!(interval1 - interval2, Interval::empty());
  }

  #[test]
  fn test_mul_empty_intervals() {
    let interval1 = Interval::empty();
    let interval2 = Interval::new(Number::from(1), IntervalType::Closed, Number::from(2));
    assert_eq!(interval1.clone() * interval2.clone(), Interval::empty());
    assert_eq!(interval2 * interval1, Interval::empty());
  }

  #[test]
  fn test_mul() {
    let interval1 = Interval::new(Number::from(1), IntervalType::RightOpen, Number::from(3));
    let interval2 = Interval::new(Number::from(4), IntervalType::Closed, Number::from(6));
    assert_eq!(
      interval1 * interval2,
      Interval::new(Number::from(4), IntervalType::RightOpen, Number::from(18)),
    );
    let interval1 = Interval::new(Number::from(-1), IntervalType::LeftOpen, Number::from(4));
    let interval2 = Interval::new(Number::from(0), IntervalType::FullOpen, Number::from(12));
    assert_eq!(
      interval1 * interval2,
      Interval::new(Number::from(-12), IntervalType::FullOpen, Number::from(48)),
    );
    let interval1 = Interval::new(Number::from(3), IntervalType::Closed, Number::from(3));
    let interval2 = Interval::new(Number::from(0), IntervalType::FullOpen, Number::from(2));
    assert_eq!(
      interval1 * interval2,
      Interval::new(Number::from(0), IntervalType::FullOpen, Number::from(6)),
    );
  }

  #[test]
  fn test_roundtrip_through_bounds() {
    let interval = Interval::new(Number::from(0), IntervalType::Closed, Number::from(10));
    let (left_bound, right_bound) = interval.clone().into_bounds();
    assert_eq!(Interval::from_bounds(left_bound, right_bound), interval);
    let interval = Interval::new(Number::from(0), IntervalType::LeftOpen, Number::from(10));
    let (left_bound, right_bound) = interval.clone().into_bounds();
    assert_eq!(Interval::from_bounds(left_bound, right_bound), interval);
    let interval = Interval::new(Number::from(0), IntervalType::RightOpen, Number::from(10));
    let (left_bound, right_bound) = interval.clone().into_bounds();
    assert_eq!(Interval::from_bounds(left_bound, right_bound), interval);
    let interval = Interval::new(Number::from(0), IntervalType::FullOpen, Number::from(10));
    let (left_bound, right_bound) = interval.clone().into_bounds();
    assert_eq!(Interval::from_bounds(left_bound, right_bound), interval);
  }
}
