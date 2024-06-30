
use super::unit::CompositeUnit;

use num::One;

use std::fmt::{self, Formatter, Display};

/// A scalar quantity, tagged with a unit.
pub struct Tagged<T> {
  pub value: T,
  pub unit: CompositeUnit<T>,
}

impl<T> Tagged<T> {
  pub fn new(value: T, unit: CompositeUnit<T>) -> Self {
    Self { value, unit }
  }

  pub fn unitless(value: T) -> Self
  where T: One {
    Self::new(value, CompositeUnit::unitless())
  }
}

impl<T> Display for Tagged<T> where T: Display {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{} {}", self.value, self.unit)
  }
}
