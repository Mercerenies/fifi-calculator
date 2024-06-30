
use super::unit::CompositeUnit;
use crate::util::prism::ErrorWithPayload;

use thiserror::Error;
use num::One;

use std::fmt::{self, Formatter, Display, Debug};
use std::ops::{Mul, Div};

/// A scalar quantity, tagged with a unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tagged<T> {
  pub value: T,
  pub unit: CompositeUnit<T>,
}

#[derive(Clone, Debug, Error)]
#[error("Failed to convert units")]
pub struct TryConvertError<T> {
  pub tagged_value: Tagged<T>,
  pub attempted_target: CompositeUnit<T>,
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

  pub fn try_convert(self, target_unit: CompositeUnit<T>) -> Result<Tagged<T>, TryConvertError<T>>
  where T: for<'a> Mul<&'a T, Output = T>,
        T: for<'a> Div<&'a T, Output = T> {
    if self.unit.dimension() == target_unit.dimension() {
      Ok(Tagged::from_base(target_unit, self.into_base()))
    } else {
      Err(TryConvertError { tagged_value: self, attempted_target: target_unit })
    }
  }

  pub fn convert_or_panic(self, target_unit: CompositeUnit<T>) -> Tagged<T>
  where T: for<'a> Mul<&'a T, Output = T>,
        T: for<'a> Div<&'a T, Output = T> {
    self.try_convert(target_unit).unwrap_or_else(|err| {
      panic!("Conversion from {} to {} failed", err.tagged_value.unit, err.attempted_target)
    })
  }
}

impl<T> Display for Tagged<T> where T: Display {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    if self.unit.is_empty() {
      write!(f, "{}", self.value)
    } else {
      write!(f, "{} {}", self.value, self.unit)
    }
  }
}

impl<T: Debug> ErrorWithPayload<Tagged<T>> for TryConvertError<T> {
  fn recover_payload(self) -> Tagged<T> {
    self.tagged_value
  }
}
