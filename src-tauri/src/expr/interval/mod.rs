
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
