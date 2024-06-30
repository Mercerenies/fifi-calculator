
use super::unit::CompositeUnit;

use num::One;

use std::fmt::{self, Formatter, Display};
use std::ops::{Mul, Div};

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

  pub fn into_base(self) -> T
  where T: for<'a> Mul<&'a T, Output = T>,
        T: for<'a> Div<&'a T, Output = T> {
    self.unit.to_base(self.value)
  }

  pub fn from_base(unit: CompositeUnit<T>, base_value: T) -> Self
  where T: for<'a> Mul<&'a T, Output = T>,
        T: for<'a> Div<&'a T, Output = T> {
    let value = unit.from_base(base_value);
    Self { value, unit }
  }
}

impl<T> Display for Tagged<T> where T: Display {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{} {}", self.value, self.unit)
  }
}
