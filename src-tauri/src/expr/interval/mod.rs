
//! Defines the datatypes and prisms for working with intervals and
//! interval arithmetic.

use super::{Expr, TryFromExprError};
use crate::util::prism::ErrorWithPayload;

use thiserror::Error;
use num::Zero;

use std::convert::TryFrom;
use std::cmp::{Ordering, min};
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

/// Equivalent to the [`Interval`] type but does not force its
/// structure into normal form. This is useful as the target of
/// prisms, since there is no data loss when storing information in
/// this structure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawInterval<T> {
  pub left: T,
  pub interval_type: IntervalType,
  pub right: T,
}

/// The disjoint union of the types [`RawInterval<T>`] and `T`. This type
/// can be used as the target of any prism that wishes to treat
/// scalars `n` as singleton intervals `n .. n`.
#[derive(Clone, Debug)]
pub enum IntervalOrScalar<T> {
  Interval(RawInterval<T>),
  Scalar(T),
}

/// The type of interval. Corresponds to the four infix operators
/// representing intervals.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntervalType {
  Closed,
  RightOpen,
  LeftOpen,
  FullOpen,
}

/// An interval bound together with its bound type.
///
/// Binary arithmetic operations on bounded numbers always take the
/// stricter bound of the two arguments.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bounded<T> {
  scalar: T,
  bound_type: BoundType,
}

/// Whether or not a bound is inclusive.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BoundType {
  Exclusive,
  Inclusive,
}

#[derive(Debug, Clone, Error)]
#[error("Error parsing interval type operator")]
pub struct ParseIntervalTypeError {
  _priv: (),
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

impl<T> RawInterval<T> {
  /// Constructs a new interval. This constructor does NOT normalize
  /// the interval.
  pub fn new(left: T, interval_type: IntervalType, right: T) -> Self {
    Self { left, interval_type, right }
  }

  /// Constructs a new interval from bounds. This constructor does NOT
  /// normalize the interval.
  pub fn from_bounds(left: Bounded<T>, right: Bounded<T>) -> Self {
    let interval_type = IntervalType::from_bounds(left.bound_type, right.bound_type);
    Self { left: left.scalar, interval_type, right: right.scalar }
  }

  pub fn normalize(self) -> Self where T: Ord + Zero {
    Interval::from(self).into_raw()
  }

  pub fn into_bounds(self) -> (Bounded<T>, Bounded<T>) {
    let (left_bound, right_bound) = self.interval_type.into_bounds();
    (
      Bounded { scalar: self.left, bound_type: left_bound },
      Bounded { scalar: self.right, bound_type: right_bound },
    )
  }
}

impl<T> Bounded<T> {
  pub fn new(scalar: T, bound_type: BoundType) -> Self {
    Self { scalar, bound_type }
  }

  pub fn bound_type(&self) -> BoundType {
    self.bound_type
  }

  pub fn scalar(&self) -> &T {
    &self.scalar
  }

  pub fn into_scalar(self) -> T {
    self.scalar
  }

  pub fn map<F, U>(self, f: F) -> Bounded<U>
  where F: FnOnce(T) -> U {
    Bounded {
      scalar: f(self.scalar),
      bound_type: self.bound_type,
    }
  }

  pub fn apply<F, S, U>(self, other: Bounded<S>, f: F) -> Bounded<U>
  where F: FnOnce(T, S) -> U {
    Bounded {
      scalar: f(self.scalar, other.scalar),
      bound_type: min(self.bound_type, other.bound_type), // Take the *stricter* bound
    }
  }

  pub fn min(self, other: Bounded<T>) -> Bounded<T> where T: Ord {
    match self.scalar.cmp(&other.scalar) {
      Ordering::Greater => other,
      Ordering::Less => self,
      Ordering::Equal => Bounded::new(self.scalar, self.bound_type.max(other.bound_type)),
    }
  }

  pub fn max(self, other: Bounded<T>) -> Bounded<T> where T: Ord {
    match self.scalar.cmp(&other.scalar) {
      Ordering::Greater => self,
      Ordering::Less => other,
      Ordering::Equal => Bounded::new(self.scalar, self.bound_type.max(other.bound_type)),
    }
  }
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

impl<T> From<RawInterval<T>> for Expr
where T: Into<Expr> {
  fn from(interval: RawInterval<T>) -> Expr {
    Expr::call(interval.interval_type.name(), vec![interval.left.into(), interval.right.into()])
  }
}

impl<T: Ord + Zero> From<RawInterval<T>> for Interval<T> {
  fn from(interval: RawInterval<T>) -> Self {
    Self::new(
      interval.left,
      interval.interval_type,
      interval.right,
    )
  }
}

impl<T> From<Interval<T>> for Expr
where T: Into<Expr> {
  fn from(interval: Interval<T>) -> Expr {
    Expr::from(interval.into_raw())
  }
}

impl<T: Clone + Ord + Zero> From<IntervalOrScalar<T>> for Interval<T> {
  fn from(interval_or_number: IntervalOrScalar<T>) -> Self {
    match interval_or_number {
      IntervalOrScalar::Interval(interval) => interval.into(),
      IntervalOrScalar::Scalar(scalar) => Interval::new(scalar.clone(), IntervalType::Closed, scalar),
    }
  }
}

fn try_from_expr_to_interval(expr: Expr) -> Result<RawInterval<Expr>, TryFromExprError> {
  const TYPE_NAME: &str = "RawInterval";
  if let Expr::Call(name, args) = expr {
    if args.len() == 2 {
      if let Ok(op) = IntervalType::parse(&name) {
        let [left, right] = args.try_into().unwrap(); // unwrap: Just checked the vec length.
        return Ok(RawInterval { left, interval_type: op, right });
      }
    }
    Err(TryFromExprError::new(TYPE_NAME, Expr::Call(name, args)))
  } else {
    Err(TryFromExprError::new(TYPE_NAME, expr))
  }
}

fn narrow_interval_type<T>(interval: RawInterval<Expr>) -> Result<RawInterval<T>, TryFromExprError>
where T: TryFrom<Expr>,
      Expr: From<T>,
      T::Error: ErrorWithPayload<Expr> {
  const TYPE_NAME: &str = "RawInterval";
  match T::try_from(interval.left) {
    Err(err) => Err(TryFromExprError::new(
      TYPE_NAME,
      RawInterval::new(err.recover_payload(), interval.interval_type, interval.right).into(),
    )),
    Ok(left) => {
      match T::try_from(interval.right) {
        Err(err) => Err(TryFromExprError::new(
          TYPE_NAME,
          RawInterval::new(left.into(), interval.interval_type, err.recover_payload()).into(),
        )),
        Ok(right) => Ok(RawInterval { left, interval_type: interval.interval_type, right }),
      }
    }
  }
}

impl<T> TryFrom<Expr> for RawInterval<T>
where T: TryFrom<Expr>,
      Expr: From<T>,
      T::Error: ErrorWithPayload<Expr> {
  type Error = TryFromExprError;

  fn try_from(expr: Expr) -> Result<Self, Self::Error> {
    let raw_interval = try_from_expr_to_interval(expr)?;
    narrow_interval_type(raw_interval)
  }
}

impl<T: Add> Add for Bounded<T> {
  type Output = Bounded<T::Output>;

  fn add(self, other: Self) -> Bounded<T::Output> {
    self.apply(other, |x, y| x + y)
  }
}

impl<T: Sub> Sub for Bounded<T> {
  type Output = Bounded<T::Output>;

  fn sub(self, other: Self) -> Bounded<T::Output> {
    self.apply(other, |x, y| x - y)
  }
}

impl<T: Mul> Mul for Bounded<T> {
  type Output = Bounded<T::Output>;

  fn mul(self, other: Self) -> Bounded<T::Output> {
    self.apply(other, |x, y| x * y)
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

  #[test]
  fn test_try_from_expr_for_raw_interval() {
    let expr = Expr::call("..", vec![Expr::from(0), Expr::from(1)]);
    assert_eq!(
      RawInterval::<Expr>::try_from(expr),
      Ok(RawInterval::<Expr>::new(Expr::from(0), IntervalType::Closed, Expr::from(1))),
    );
    let expr = Expr::call("..^", vec![Expr::call("foo", vec![]), Expr::from(2)]);
    assert_eq!(
      RawInterval::<Expr>::try_from(expr),
      Ok(RawInterval::<Expr>::new(Expr::call("foo", vec![]), IntervalType::RightOpen, Expr::from(2))),
    );
  }

  #[test]
  fn test_try_from_expr_for_raw_interval_failed() {
    let expr = Expr::call("foo", vec![Expr::from(0), Expr::from(1)]);
    assert_eq!(
      RawInterval::<Expr>::try_from(expr),
      Err(TryFromExprError::new(
        "RawInterval",
        Expr::call("foo", vec![Expr::from(0), Expr::from(1)])
      )),
    );
    let expr = Expr::call("..", vec![Expr::from(0), Expr::from(1), Expr::from(2)]);
    assert_eq!(
      RawInterval::<Expr>::try_from(expr),
      Err(TryFromExprError::new(
        "RawInterval",
        Expr::call("..", vec![Expr::from(0), Expr::from(1), Expr::from(2)])
      )),
    );
    let expr = Expr::from(0);
    assert_eq!(
      RawInterval::<Expr>::try_from(expr),
      Err(TryFromExprError::new(
        "RawInterval",
        Expr::from(0),
      )),
    );
  }

  #[test]
  fn test_try_from_expr_for_raw_interval_number() {
    let expr = Expr::call("..", vec![Expr::from(0), Expr::from(1)]);
    assert_eq!(
      RawInterval::try_from(expr),
      Ok(RawInterval::new(Number::from(0), IntervalType::Closed, Number::from(1))),
    );
  }

  #[test]
  fn test_try_from_expr_for_interval_with_non_literal_arg() {
    let expr = Expr::call("..^", vec![Expr::call("foo", vec![]), Expr::from(2)]);
    assert_eq!(
      RawInterval::<Number>::try_from(expr),
      Err(TryFromExprError::new(
        "RawInterval",
        Expr::call("..^", vec![Expr::call("foo", vec![]), Expr::from(2)])
      )),
    );
  }

  #[test]
  fn test_try_from_expr_for_interval_failed() {
    let expr = Expr::call("foo", vec![Expr::from(0), Expr::from(1)]);
    assert_eq!(
      RawInterval::<Number>::try_from(expr),
      Err(TryFromExprError::new(
        "RawInterval",
        Expr::call("foo", vec![Expr::from(0), Expr::from(1)])
      )),
    );
    let expr = Expr::call("..", vec![Expr::from(0), Expr::from(1), Expr::from(2)]);
    assert_eq!(
      RawInterval::<Number>::try_from(expr),
      Err(TryFromExprError::new(
        "RawInterval",
        Expr::call("..", vec![Expr::from(0), Expr::from(1), Expr::from(2)])
      )),
    );
    let expr = Expr::from(0);
    assert_eq!(
      RawInterval::<Number>::try_from(expr),
      Err(TryFromExprError::new(
        "RawInterval",
        Expr::from(0),
      )),
    );
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
