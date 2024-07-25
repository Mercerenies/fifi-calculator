
use super::interval_type::IntervalType;
use super::bound::Bounded;
use super::raw::RawInterval;
use crate::expr::Expr;
use crate::util::unwrap_infallible;

use try_traits::ops::{TryAdd, TrySub, TryMul};

use std::ops::Neg;

/// An interval form consisting of specifically real numbers on the
/// left and right hand sides.
///
/// Intervals are always kept in normal form, which is defined as
/// follows. An interval is in normal form if it is either (a)
/// nonempty or (b) equal to the interval `0..^0` (where 0 is the
/// value [`T::default()`]). Put another way, the normal form of the
/// empty interval is `0..^0`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Interval<T> {
  left: T,
  interval_type: IntervalType,
  right: T,
}

impl<T: Default + Ord> Interval<T> {
  /// Constructs a new interval.
  pub fn new(left: T, interval_type: IntervalType, right: T) -> Self {
    Self { left, interval_type, right }.normalize()
  }

  pub fn singleton(value: T) -> Self where T: Clone {
    Self { left: value.clone(), interval_type: IntervalType::Closed, right: value }
  }

  /// Constructs a new interval from bounds.
  pub fn from_bounds(left: Bounded<T>, right: Bounded<T>) -> Self {
    let interval_type = IntervalType::from_bounds(left.bound_type, right.bound_type);
    Self { left: left.scalar, interval_type, right: right.scalar }.normalize()
  }

  pub fn empty() -> Self {
    Self { left: T::default(), interval_type: IntervalType::RightOpen, right: T::default() }
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
        U: Ord + Default {
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
  ///
  /// If the underlying operation fails, then failures are propagated
  /// to the caller.
  pub fn apply_monotone_err<F, S, U, E>(self, other: Interval<S>, f: F) -> Result<Interval<U>, E>
  where F: Fn(T, S) -> Result<U, E>,
        T: Clone,
        S: Clone + Ord + Default,
        U: Clone + Ord + Default {
    if self.is_empty() || other.is_empty() {
      return Ok(Interval::empty());
    }

    let (left_lower, left_upper) = self.into_bounds();
    let (right_lower, right_upper) = other.into_bounds();
    let all_combinations = [
      left_lower.clone().apply_err(right_lower.clone(), &f)?,
      left_lower.clone().apply_err(right_upper.clone(), &f)?,
      left_upper.clone().apply_err(right_lower, &f)?,
      left_upper.apply_err(right_upper, &f)?,
    ];
    let lower = all_combinations.iter().cloned().reduce(Bounded::min).unwrap(); // unwrap: Non-empty array
    let upper = all_combinations.into_iter().reduce(Bounded::max).unwrap(); // unwrap: Non-empty array
    Ok(Interval::from_bounds(lower, upper).normalize())
  }

  /// Applies a binary, monotone function to the two intervals to
  /// produce a new interval. It is the caller's responsibility to
  /// ensure that the provided function is monotonic.
  pub fn apply_monotone<F, S, U>(self, other: Interval<S>, f: F) -> Interval<U>
  where F: Fn(T, S) -> U,
        T: Clone,
        S: Clone + Ord + Default,
        U: Clone + Ord + Default {
    unwrap_infallible(
      self.apply_monotone_err(other, |x, y| Ok(f(x, y))),
    )
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

impl<T: Default + Ord> Default for Interval<T> {
  fn default() -> Self {
    Self::empty()
  }
}

impl<T> From<Interval<T>> for Expr
where T: Into<Expr> {
  fn from(interval: Interval<T>) -> Expr {
    Expr::from(interval.into_raw())
  }
}

// Note: In principle, we would also supply impl Add, Mul, Sub, etc.
// for Interval when the underlying type supports it. Unfortunately,
// due to blanket impls of TryAdd, etc. in try_traits, that creates a
// trait conflict. So we just impl the Try* versions, and if Error =
// Infallible then so be it.

impl<T: TryAdd + Default + Ord> TryAdd for Interval<T>
where <T as TryAdd>::Output: Default + Ord {
  type Output = Interval<<T as TryAdd>::Output>;
  type Error = <T as TryAdd>::Error;

  fn try_add(self, other: Self) -> Result<Self::Output, Self::Error> {
    if self.is_empty() || other.is_empty() {
      return Ok(Interval::empty());
    }
    let left = self.left.try_add(other.left)?;
    let right = self.right.try_add(other.right)?;
    let interval_type = self.interval_type.min(other.interval_type);
    Ok(Interval::new(left, interval_type, right).normalize())
  }
}

impl<T: TrySub + Default + Ord> TrySub for Interval<T>
where <T as TrySub>::Output: Default + Ord {
  type Output = Interval<<T as TrySub>::Output>;
  type Error = <T as TrySub>::Error;

  fn try_sub(self, other: Self) -> Result<Self::Output, Self::Error> {
    if self.is_empty() || other.is_empty() {
      return Ok(Interval::empty());
    }
    let left = self.left.try_sub(other.right)?;
    let right = self.right.try_sub(other.left)?;
    let interval_type = self.interval_type.min(other.interval_type.flipped());
    Ok(Interval::new(left, interval_type, right).normalize())
  }
}

/// Note: This instance assumes that the multiplication on `T` is a
/// monotone operation with respect to its `Ord` instance.
impl<T> TryMul for Interval<T>
where T: TryMul + Default + Ord + Clone,
      <T as TryMul>::Output: Default + Ord + Clone {
  type Output = Interval<<T as TryMul>::Output>;
  type Error = <T as TryMul>::Error;

  fn try_mul(self, other: Self) -> Result<Self::Output, Self::Error> {
    self.apply_monotone_err(other, |x, y| x.try_mul(y))
  }
}

/// Note: This instance assumes that the `T: Neg` instance is
/// order-reversing with respect to the `T: Ord` instance.
impl<T: Neg> Neg for Interval<T>
where <T as Neg>::Output: Default + Ord {
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
      interval1.try_add(interval2).unwrap(),
      Interval::new(Number::from(11), IntervalType::Closed, Number::from(22)),
    );
    let interval1 = Interval::new(Number::from(1), IntervalType::LeftOpen, Number::from(2));
    let interval2 = Interval::new(Number::from(10), IntervalType::FullOpen, Number::from(20));
    assert_eq!(
      interval1.try_add(interval2).unwrap(),
      Interval::new(Number::from(11), IntervalType::FullOpen, Number::from(22)),
    );
    let interval1 = Interval::new(Number::from(1), IntervalType::LeftOpen, Number::from(2));
    let interval2 = Interval::new(Number::from(10), IntervalType::RightOpen, Number::from(20));
    assert_eq!(
      interval1.try_add(interval2).unwrap(),
      Interval::new(Number::from(11), IntervalType::FullOpen, Number::from(22)),
    );
    let interval1 = Interval::new(Number::from(1), IntervalType::RightOpen, Number::from(2));
    let interval2 = Interval::new(Number::from(10), IntervalType::RightOpen, Number::from(20));
    assert_eq!(
      interval1.try_add(interval2).unwrap(),
      Interval::new(Number::from(11), IntervalType::RightOpen, Number::from(22)),
    );
  }

  #[test]
  fn test_sub_interval() {
    let interval1 = Interval::new(Number::from(1), IntervalType::Closed, Number::from(2));
    let interval2 = Interval::new(Number::from(10), IntervalType::Closed, Number::from(20));
    assert_eq!(
      interval1.try_sub(interval2).unwrap(),
      Interval::new(Number::from(-19), IntervalType::Closed, Number::from(-8)),
    );
    let interval1 = Interval::new(Number::from(1), IntervalType::LeftOpen, Number::from(2));
    let interval2 = Interval::new(Number::from(10), IntervalType::FullOpen, Number::from(20));
    assert_eq!(
      interval1.try_sub(interval2).unwrap(),
      Interval::new(Number::from(-19), IntervalType::FullOpen, Number::from(-8)),
    );
    let interval1 = Interval::new(Number::from(1), IntervalType::LeftOpen, Number::from(2));
    let interval2 = Interval::new(Number::from(10), IntervalType::RightOpen, Number::from(20));
    assert_eq!(
      interval1.try_sub(interval2).unwrap(),
      Interval::new(Number::from(-19), IntervalType::LeftOpen, Number::from(-8)),
    );
    let interval1 = Interval::new(Number::from(1), IntervalType::RightOpen, Number::from(2));
    let interval2 = Interval::new(Number::from(10), IntervalType::RightOpen, Number::from(20));
    assert_eq!(
      interval1.try_sub(interval2).unwrap(),
      Interval::new(Number::from(-19), IntervalType::FullOpen, Number::from(-8)),
    );
  }

  #[test]
  fn test_add_empty_intervals() {
    let interval1 = Interval::empty();
    let interval2 = Interval::new(Number::from(1), IntervalType::Closed, Number::from(2));
    assert_eq!(interval1.try_add(interval2).unwrap(), Interval::empty());
  }

  #[test]
  fn test_sub_empty_intervals() {
    let interval1 = Interval::empty();
    let interval2 = Interval::new(Number::from(1), IntervalType::Closed, Number::from(2));
    assert_eq!(interval1.try_sub(interval2).unwrap(), Interval::empty());
  }

  #[test]
  fn test_mul_empty_intervals() {
    let interval1 = Interval::empty();
    let interval2 = Interval::new(Number::from(1), IntervalType::Closed, Number::from(2));
    assert_eq!(interval1.clone().try_mul(interval2.clone()).unwrap(), Interval::empty());
    assert_eq!(interval2.try_mul(interval1).unwrap(), Interval::empty());
  }

  #[test]
  fn test_mul() {
    let interval1 = Interval::new(Number::from(1), IntervalType::RightOpen, Number::from(3));
    let interval2 = Interval::new(Number::from(4), IntervalType::Closed, Number::from(6));
    assert_eq!(
      interval1.try_mul(interval2).unwrap(),
      Interval::new(Number::from(4), IntervalType::RightOpen, Number::from(18)),
    );
    let interval1 = Interval::new(Number::from(-1), IntervalType::LeftOpen, Number::from(4));
    let interval2 = Interval::new(Number::from(0), IntervalType::FullOpen, Number::from(12));
    assert_eq!(
      interval1.try_mul(interval2).unwrap(),
      Interval::new(Number::from(-12), IntervalType::FullOpen, Number::from(48)),
    );
    let interval1 = Interval::new(Number::from(3), IntervalType::Closed, Number::from(3));
    let interval2 = Interval::new(Number::from(0), IntervalType::FullOpen, Number::from(2));
    assert_eq!(
      interval1.try_mul(interval2).unwrap(),
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
