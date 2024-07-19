
use super::composite::CompositeUnit;
use crate::util::prism::ErrorWithPayload;

use thiserror::Error;
use num::One;

use std::fmt::{self, Formatter, Display, Debug};
use std::ops::{Mul, Div};

/// A scalar quantity, tagged with a unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tagged<S, U> {
  pub value: S,
  pub unit: CompositeUnit<U>,
}

#[derive(Clone, Debug, Error)]
#[error("Failed to convert units")]
pub struct TryConvertError<S, U> {
  pub tagged_value: Tagged<S, U>,
  pub attempted_target: CompositeUnit<U>,
}

impl<S, U> Tagged<S, U> {
  pub fn new(value: S, unit: CompositeUnit<U>) -> Self {
    Self { value, unit }
  }

  pub fn unitless(value: S) -> Self
  where U: One {
    Self::new(value, CompositeUnit::unitless())
  }

  pub fn into_base(self) -> S
  where S: for<'a> Mul<&'a U, Output = S>,
        S: for<'a> Div<&'a U, Output = S> {
    self.unit.to_base(self.value)
  }

  pub fn from_base(unit: CompositeUnit<U>, base_value: S) -> Self
  where S: for<'a> Mul<&'a U, Output = S>,
        S: for<'a> Div<&'a U, Output = S> {
    let value = unit.from_base(base_value);
    Self { value, unit }
  }

  pub fn try_convert(self, target_unit: CompositeUnit<U>) -> Result<Tagged<S, U>, TryConvertError<S, U>>
  where S: for<'a> Mul<&'a U, Output = S>,
        S: for<'a> Div<&'a U, Output = S> {
    if self.unit.dimension() == target_unit.dimension() {
      Ok(Tagged::from_base(target_unit, self.into_base()))
    } else {
      Err(TryConvertError { tagged_value: self, attempted_target: target_unit })
    }
  }

  pub fn convert_or_panic(self, target_unit: CompositeUnit<U>) -> Tagged<S, U>
  where S: for<'a> Mul<&'a U, Output = S>,
        S: for<'a> Div<&'a U, Output = S> {
    self.try_convert(target_unit).unwrap_or_else(|err| {
      panic!("Conversion from {} to {} failed", err.tagged_value.unit, err.attempted_target)
    })
  }
}

impl<S: Display, U> Display for Tagged<S, U> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    if self.unit.is_empty() {
      write!(f, "{}", self.value)
    } else {
      write!(f, "{} {}", self.value, self.unit)
    }
  }
}

impl<S: Debug, U: Debug> ErrorWithPayload<Tagged<S, U>> for TryConvertError<S, U> {
  fn recover_payload(self) -> Tagged<S, U> {
    self.tagged_value
  }
}
