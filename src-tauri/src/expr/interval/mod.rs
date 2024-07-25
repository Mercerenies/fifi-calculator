
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

use num::Zero;

/// The disjoint union of the types [`RawInterval<T>`] and `T`. This type
/// can be used as the target of any prism that wishes to treat
/// scalars `n` as singleton intervals `n .. n`.
#[derive(Clone, Debug)]
pub enum IntervalOrScalar<T> {
  Interval(RawInterval<T>),
  Scalar(T),
}

impl<T: Clone + Ord + Zero> From<IntervalOrScalar<T>> for Interval<T> {
  fn from(interval_or_number: IntervalOrScalar<T>) -> Self {
    match interval_or_number {
      IntervalOrScalar::Interval(interval) => interval.into(),
      IntervalOrScalar::Scalar(scalar) => Interval::new(scalar.clone(), IntervalType::Closed, scalar),
    }
  }
}
