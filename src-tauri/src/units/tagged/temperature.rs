
use super::base::Tagged;
use crate::units::dimension::{Dimension, BaseDimension};
use crate::units::unit::Unit;
use crate::units::composite::CompositeUnit;
use crate::units::convertible::TemperatureConvertible;

use thiserror::Error;

use std::convert::TryFrom;
use std::fmt::{self, Formatter, Display};

/// A [`Tagged`] value whose unit is a one-dimensional temperature
/// unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemperatureTagged<S, U> {
  value: S,
  unit: Unit<U>,
}

#[derive(Debug, Clone, Error)]
#[error("Expected temperature unit")]
pub struct DimensionMismatchError<S, U> {
  pub value: S,
  pub unit: Unit<U>,
  _priv: (),
}

#[derive(Debug, Clone, Error)]
#[non_exhaustive]
pub enum TryFromTaggedError<S, U> {
  #[error("{0}")]
  DimensionMismatch(#[from] DimensionMismatchError<S, U>),
  #[error("Expected single unit")]
  ExpectedSingleUnit(Tagged<S, U>),
}

impl<S, U> TemperatureTagged<S, U> {
  pub fn try_new(value: S, unit: Unit<U>) -> Result<Self, DimensionMismatchError<S, U>> {
    if unit.dimension() == &Dimension::from(BaseDimension::Temperature) {
      Ok(TemperatureTagged { value, unit })
    } else {
      Err(DimensionMismatchError {
        value,
        unit,
        _priv: (),
      })
    }
  }

  /// Constructs a [`TemperatureTagged`]. Panics if the unit is not a
  /// one-dimensional temperature unit.
  pub fn new(value: S, unit: Unit<U>) -> Self {
    Self::try_new(value, unit).unwrap_or_else(|_| {
      panic!("Expected temperature unit");
    })
  }
}

impl<S, U> TemperatureTagged<S, U>
where S: TemperatureConvertible<U> {
  pub fn into_base(self) -> <S as TemperatureConvertible<U>>::Output {
    let multiplicative_value = self.unit.to_base(self.value);
    multiplicative_value.offset(self.unit.temperature_offset())
  }

  pub fn try_from_base(
    unit: Unit<U>,
    base_value: <S as TemperatureConvertible<U>>::Output,
  ) -> Result<Self, DimensionMismatchError<S, U>> {
    let additive_value = S::unoffset(base_value, unit.temperature_offset());
    let multiplicative_value = unit.from_base(additive_value);
    Self::try_new(multiplicative_value, unit)
  }

  pub fn from_base(
    unit: Unit<U>,
    base_value: <S as TemperatureConvertible<U>>::Output,
  ) -> Self {
    Self::try_from_base(unit, base_value).unwrap_or_else(|_| {
      panic!("Expected temperature unit")
    })
  }

  pub fn try_convert(self, target_unit: Unit<U>) -> Result<Self, DimensionMismatchError<S, U>> {
    Self::try_from_base(target_unit, self.into_base())
  }

  pub fn convert(self, target_unit: Unit<U>) -> Self {
    Self::from_base(target_unit, self.into_base())
  }
}

impl<S, U> From<TemperatureTagged<S, U>> for Tagged<S, U> {
  fn from(t: TemperatureTagged<S, U>) -> Self {
    Tagged::new(t.value, t.unit.into())
  }
}

impl<S, U> TryFrom<Tagged<S, U>> for TemperatureTagged<S, U> {
  type Error = TryFromTaggedError<S, U>;

  fn try_from(tagged: Tagged<S, U>) -> Result<Self, TryFromTaggedError<S, U>> {
    let mut inner_units = tagged.unit.into_inner();
    if inner_units.len() != 1 || inner_units[0].exponent != 1 {
      return Err(TryFromTaggedError::ExpectedSingleUnit(
        Tagged::new(tagged.value, CompositeUnit::new(inner_units)),
      ));
    }
    let tagged = Self::try_new(tagged.value, inner_units.swap_remove(0).unit)?;
    Ok(tagged)
  }
}

impl<S: Display, U> Display for TemperatureTagged<S, U> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{} {}", self.value, self.unit)
  }
}
