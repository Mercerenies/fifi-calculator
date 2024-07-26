
//! Defines the datatypes and prisms for working with intervals and
//! interval arithmetic.

mod base;
mod bound;
mod interval_type;
mod raw;

pub use base::Interval;
pub use bound::{Bounded, BoundType};
pub use raw::RawInterval;
pub use interval_type::IntervalType;

use crate::expr::algebra::infinity::{UnboundedNumber, IndeterminateFormError, SignedInfinity};
use crate::expr::number::Number;

use try_traits::ops::TryMul;
use num::Zero;

use std::cmp::Ordering;

/// The disjoint union of the types [`RawInterval<T>`] and `T`. This type
/// can be used as the target of any prism that wishes to treat
/// scalars `n` as singleton intervals `n .. n`.
#[derive(Clone, Debug)]
pub enum IntervalOrScalar<T> {
  Interval(RawInterval<T>),
  Scalar(T),
}

impl<T: Clone + Ord + Default> From<IntervalOrScalar<T>> for Interval<T> {
  fn from(interval_or_number: IntervalOrScalar<T>) -> Self {
    match interval_or_number {
      IntervalOrScalar::Interval(interval) => interval.into(),
      IntervalOrScalar::Scalar(scalar) => Interval::new(scalar.clone(), IntervalType::Closed, scalar),
    }
  }
}

pub fn is_unbounded(interval: &Interval<UnboundedNumber>) -> bool {
  interval.left().is_infinite() || interval.right().is_infinite()
}

pub fn includes_infinity(interval: &Interval<UnboundedNumber>) -> bool {
  interval.left().is_infinite() && interval.interval_type().includes_left() ||
    interval.right().is_infinite() && interval.interval_type().includes_right()
}

/// Returns a union of intervals.
pub fn interval_recip(interval: Interval<UnboundedNumber>) -> Interval<UnboundedNumber> {
  // Follows the algorithm at
  // https://en.wikipedia.org/wiki/Interval_arithmetic#Interval_operators,
  // with extensions for infinite bounds.
  use UnboundedNumber::{Finite, Infinite};
  use SignedInfinity::{PosInfinity, NegInfinity};
  let (left, right) = interval.into_bounds();
  match (left.scalar, right.scalar) {
    (Infinite(PosInfinity), Infinite(PosInfinity)) => Interval::singleton(UnboundedNumber::zero()),
    (Infinite(NegInfinity), Infinite(NegInfinity)) => Interval::singleton(UnboundedNumber::zero()),
    (Infinite(NegInfinity), Infinite(PosInfinity)) => all_real_numbers(IntervalType::Closed),
    (Infinite(PosInfinity), Infinite(NegInfinity)) => unreachable!(), // Not in normal form!
    (Infinite(PosInfinity), Finite(_)) => unreachable!(), // Not in normal form!
    (Finite(_), Infinite(NegInfinity)) => unreachable!(), // Not in normal form!
    (Infinite(NegInfinity), Finite(b)) =>
      match b.cmp(&Number::zero()) {
        Ordering::Greater => {
          all_real_numbers(IntervalType::Closed)
        }
        Ordering::Less => {
          Interval::from_bounds(
            Bounded::new(Finite(b.recip()), right.bound_type),
            Bounded::new(Finite(Number::zero()), left.bound_type),
          )
        }
        Ordering::Equal => {
          Interval::from_bounds(
            Bounded::new(Infinite(NegInfinity), right.bound_type),
            Bounded::new(Finite(Number::zero()), left.bound_type),
          )
        }
      },
    (Finite(a), Infinite(PosInfinity)) =>
      match a.cmp(&Number::zero()) {
        Ordering::Less => {
          all_real_numbers(IntervalType::Closed)
        }
        Ordering::Greater => {
          Interval::from_bounds(
            Bounded::new(Finite(Number::zero()), right.bound_type),
            Bounded::new(Finite(a.recip()), left.bound_type),
          )
        }
        Ordering::Equal => {
          Interval::from_bounds(
            Bounded::new(Finite(Number::zero()), right.bound_type),
            Bounded::new(Infinite(PosInfinity), left.bound_type),
          )
        }
      },
    (Finite(a), Finite(b)) => {
      if a.is_zero() && b.is_zero() {
        // TODO: Not totally correct; we really want uinf here but we
        // don't have a type that supports it available in this
        // function.
        Interval::singleton(Infinite(PosInfinity))
      } else if a.is_zero() {
        Interval::from_bounds(
          Bounded::new(Finite(b.recip()), right.bound_type),
          Bounded::new(Infinite(PosInfinity), left.bound_type),
        )
      } else if b.is_zero() {
        Interval::from_bounds(
          Bounded::new(Infinite(NegInfinity), right.bound_type),
          Bounded::new(Finite(a.recip()), left.bound_type),
        )
      } else if a < Number::zero() && Number::zero() < b {
        all_real_numbers(IntervalType::Closed)
      } else {
        Interval::from_bounds(
          Bounded::new(Finite(b.recip()), right.bound_type),
          Bounded::new(Finite(a.recip()), left.bound_type),
        )
      }
    }
  }
}

fn all_real_numbers(interval_type: IntervalType) -> Interval<UnboundedNumber> {
  Interval::new(UnboundedNumber::NEG_INFINITY, interval_type, UnboundedNumber::POS_INFINITY)
}

/// Returns a union of intervals.
pub fn interval_div(
  left: Interval<UnboundedNumber>,
  right: Interval<UnboundedNumber>,
) -> Result<Interval<UnboundedNumber>, IndeterminateFormError> {
  left.try_mul(interval_recip(right))
}

#[cfg(test)]
mod tests {
  use super::*;

  fn pos_infinity() -> UnboundedNumber {
    UnboundedNumber::POS_INFINITY
  }

  fn neg_infinity() -> UnboundedNumber {
    UnboundedNumber::NEG_INFINITY
  }

  fn finite(n: f64) -> UnboundedNumber {
    UnboundedNumber::Finite(n.into())
  }

  #[test]
  fn test_interval_recip_finite_without_zero() {
    let interval = Interval::new(finite(1.0), IntervalType::LeftOpen, finite(10.0));
    assert_eq!(
      interval_recip(interval),
      Interval::new(finite(0.1), IntervalType::RightOpen, finite(1.0))
    );

    let interval = Interval::new(finite(-10.0), IntervalType::Closed, finite(-0.5));
    assert_eq!(
      interval_recip(interval),
      Interval::new(finite(-2.0), IntervalType::Closed, finite(-0.1))
    );

    let interval = Interval::new(finite(4.0), IntervalType::Closed, finite(4.0));
    assert_eq!(
      interval_recip(interval),
      Interval::new(finite(0.25), IntervalType::Closed, finite(0.25))
    );
  }

  #[test]
  fn test_interval_recip_finite_right_zero() {
    let interval = Interval::new(finite(-4.0), IntervalType::LeftOpen, finite(0.0));
    assert_eq!(
      interval_recip(interval),
      Interval::new(neg_infinity(), IntervalType::RightOpen, finite(-0.25))
    );

    let interval = Interval::new(finite(-4.0), IntervalType::RightOpen, finite(0.0));
    assert_eq!(
      interval_recip(interval),
      Interval::new(neg_infinity(), IntervalType::LeftOpen, finite(-0.25))
    );
  }

  #[test]
  fn test_interval_recip_finite_left_zero() {
    let interval = Interval::new(finite(0.0), IntervalType::LeftOpen, finite(10.0));
    assert_eq!(
      interval_recip(interval),
      Interval::new(finite(0.1), IntervalType::RightOpen, pos_infinity())
    );

    let interval = Interval::new(finite(0.0), IntervalType::RightOpen, finite(20.0));
    assert_eq!(
      interval_recip(interval),
      Interval::new(finite(0.05), IntervalType::LeftOpen, pos_infinity())
    );
  }

  #[test]
  fn test_interval_recip_finite_includes_zero() {
    let interval = Interval::new(finite(-5.0), IntervalType::LeftOpen, finite(10.0));
    assert_eq!(
      interval_recip(interval),
      Interval::new(neg_infinity(), IntervalType::Closed, pos_infinity())
    );
  }

  #[test]
  fn test_interval_recip_all_reals() {
    let interval = Interval::new(neg_infinity(), IntervalType::Closed, pos_infinity());
    assert_eq!(
      interval_recip(interval),
      Interval::new(neg_infinity(), IntervalType::Closed, pos_infinity()),
    );
  }

  #[test]
  fn test_interval_recip_negative_unbounded() {
    let interval = Interval::new(neg_infinity(), IntervalType::RightOpen, finite(-2.0));
    assert_eq!(
      interval_recip(interval),
      Interval::new(finite(-0.5), IntervalType::LeftOpen, finite(0.0)),
    );

    let interval = Interval::new(neg_infinity(), IntervalType::RightOpen, finite(0.0));
    assert_eq!(
      interval_recip(interval),
      Interval::new(neg_infinity(), IntervalType::LeftOpen, finite(0.0)),
    );

    let interval = Interval::new(neg_infinity(), IntervalType::RightOpen, finite(2.0));
    assert_eq!(
      interval_recip(interval),
      Interval::new(neg_infinity(), IntervalType::Closed, pos_infinity()),
    );
  }

  #[test]
  fn test_interval_recip_positive_unbounded() {
    let interval = Interval::new(finite(2.0), IntervalType::FullOpen, pos_infinity());
    assert_eq!(
      interval_recip(interval),
      Interval::new(finite(0.0), IntervalType::FullOpen, finite(0.5)),
    );

    let interval = Interval::new(finite(0.0), IntervalType::RightOpen, pos_infinity());
    assert_eq!(
      interval_recip(interval),
      Interval::new(finite(0.0), IntervalType::LeftOpen, pos_infinity()),
    );

    let interval = Interval::new(finite(-2.0), IntervalType::RightOpen, pos_infinity());
    assert_eq!(
      interval_recip(interval),
      Interval::new(neg_infinity(), IntervalType::Closed, pos_infinity()),
    );
  }
}
