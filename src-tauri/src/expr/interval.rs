
//! Defines the datatypes and prisms for working with intervals and
//! interval arithmetic.

use super::{Expr, TryFromExprError};
use super::number::Number;
use crate::util::prism::ErrorWithPayload;

use thiserror::Error;
use num::Zero;

use std::convert::TryFrom;
use std::cmp::Ordering;
use std::ops::{Add, Sub};

/// An interval form which allows arbitrary expressions on the left
/// and right hand sides.
///
/// For a variant that requires numbers as interval bounds, see
/// [`Interval`].
#[derive(Clone, Debug)]
pub struct IntervalAny {
  left: Expr,
  interval_type: IntervalType,
  right: Expr,
}

/// An interval form consisting of specifically real numbers on the
/// left and right hand sides.
#[derive(Clone, Debug, PartialEq)]
pub struct Interval {
  left: Number,
  interval_type: IntervalType,
  right: Number,
}

/// The disjoint union of the types [`Interval`] and [`Number`]. This
/// type can be used as the target of any prism that wishes to treat
/// numbers `n` as singleton intervals `n .. n`.
#[derive(Clone, Debug)]
pub enum IntervalOrNumber {
  Interval(Interval),
  Number(Number),
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

#[derive(Debug, Clone, Error)]
#[error("Error parsing interval type operator")]
pub struct ParseIntervalTypeError {
  _priv: (),
}

#[derive(Debug, Clone, Error)]
#[error("Expected Interval of real numbers")]
pub struct TryFromIntervalAnyError {
  original_value: IntervalAny,
  _priv: (),
}

impl IntervalAny {
  pub fn new(left: Expr, interval_type: IntervalType, right: Expr) -> Self {
    Self { left, interval_type, right }
  }
}

impl Interval {
  pub fn new(left: Number, interval_type: IntervalType, right: Number) -> Self {
    Self { left, interval_type, right }
  }

  pub fn empty_at(bound: Number) -> Self {
    Self { left: bound.clone(), interval_type: IntervalType::RightOpen, right: bound }
  }

  pub fn empty() -> Self {
    Self::empty_at(Number::zero())
  }

  pub fn normalize(self) -> Self {
    if self.right < self.left || (self.right == self.left && self.interval_type != IntervalType::Closed) {
      // The interval is empty, so represent it as the empty interval.
      Self::empty_at(self.left)
    } else {
      self
    }
  }
}

impl IntervalType {
  pub fn name(self) -> &'static str {
    match self {
      IntervalType::Closed => "..",
      IntervalType::RightOpen => "..^",
      IntervalType::LeftOpen => "^..",
      IntervalType::FullOpen => "^..^",
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

impl From<IntervalAny> for Expr {
  fn from(interval: IntervalAny) -> Expr {
    Expr::call(interval.interval_type.name(), vec![interval.left, interval.right])
  }
}

impl From<Interval> for IntervalAny {
  fn from(interval: Interval) -> Self {
    Self {
      left: interval.left.into(),
      interval_type: interval.interval_type,
      right: interval.right.into(),
    }
  }
}

impl From<Interval> for Expr {
  fn from(interval: Interval) -> Expr {
    Expr::from(
      IntervalAny::from(interval),
    )
  }
}

impl From<IntervalOrNumber> for Interval {
  fn from(interval_or_number: IntervalOrNumber) -> Self {
    match interval_or_number {
      IntervalOrNumber::Interval(interval) => interval,
      IntervalOrNumber::Number(number) => Interval::new(number.clone(), IntervalType::Closed, number),
    }
  }
}

impl TryFrom<Expr> for IntervalAny {
  type Error = TryFromExprError;

  fn try_from(expr: Expr) -> Result<Self, Self::Error> {
    const TYPE_NAME: &'static str = "IntervalAny";
    if let Expr::Call(name, args) = expr {
      if args.len() == 2 {
        if let Ok(op) = IntervalType::parse(&name) {
          let [left, right] = args.try_into().unwrap(); // unwrap: Just checked the vec length.
          return Ok(IntervalAny { left, interval_type: op, right });
        }
      }
      Err(TryFromExprError::new(TYPE_NAME, Expr::Call(name, args)))
    } else {
      Err(TryFromExprError::new(TYPE_NAME, expr))
    }
  }
}

impl TryFrom<IntervalAny> for Interval {
  type Error = TryFromIntervalAnyError;

  fn try_from(interval: IntervalAny) -> Result<Self, Self::Error> {
    match Number::try_from(interval.left) {
      Err(err) => Err(TryFromIntervalAnyError {
        original_value: IntervalAny::new(err.recover_payload(), interval.interval_type, interval.right),
        _priv: (),
      }),
      Ok(left) => {
        match Number::try_from(interval.right) {
          Err(err) => Err(TryFromIntervalAnyError {
            original_value: IntervalAny::new(left.into(), interval.interval_type, err.recover_payload()),
            _priv: (),
          }),
          Ok(right) => Ok(Interval { left, interval_type: interval.interval_type, right }),
        }
      }
    }
  }
}

impl TryFrom<Expr> for Interval {
  type Error = TryFromExprError;

  fn try_from(expr: Expr) -> Result<Self, Self::Error> {
    const TYPE_NAME: &'static str = "Interval";
    IntervalAny::try_from(expr)
      .map_err(|err| err.with_type_name(TYPE_NAME))
      .and_then(|interval| {
        Interval::try_from(interval)
          .map_err(|err| TryFromExprError::new(TYPE_NAME, err.recover_payload().into()))
      })
  }
}

impl ErrorWithPayload<IntervalAny> for TryFromIntervalAnyError {
  fn recover_payload(self) -> IntervalAny {
    self.original_value
  }
}

impl Add for Interval {
  type Output = Self;

  fn add(self, other: Self) -> Self {
    let interval_type = self.interval_type.min(other.interval_type);
    Interval::new(self.left + other.left, interval_type, self.right + other.right).normalize()
  }
}

impl Sub for Interval {
  type Output = Self;

  fn sub(self, other: Self) -> Self {
    let interval_type = self.interval_type.min(other.interval_type.flipped());
    Interval::new(self.left - other.right, interval_type, self.right - other.left).normalize()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

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
    assert_eq!(interval.normalize(), Interval::empty_at(Number::from(2)));
    let interval = Interval::new(Number::from(2), IntervalType::LeftOpen, Number::from(1));
    assert_eq!(interval.normalize(), Interval::empty_at(Number::from(2)));
    let interval = Interval::new(Number::from(2), IntervalType::RightOpen, Number::from(1));
    assert_eq!(interval.normalize(), Interval::empty_at(Number::from(2)));
    let interval = Interval::new(Number::from(2), IntervalType::FullOpen, Number::from(1));
    assert_eq!(interval.normalize(), Interval::empty_at(Number::from(2)));
    let interval = Interval::new(Number::from(2), IntervalType::LeftOpen, Number::from(2));
    assert_eq!(interval.normalize(), Interval::empty_at(Number::from(2)));
    let interval = Interval::new(Number::from(2), IntervalType::RightOpen, Number::from(2));
    assert_eq!(interval.normalize(), Interval::empty_at(Number::from(2)));
    let interval = Interval::new(Number::from(2), IntervalType::FullOpen, Number::from(2));
    assert_eq!(interval.normalize(), Interval::empty_at(Number::from(2)));
  }

  // TODO: More tests
}
