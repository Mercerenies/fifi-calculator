
use super::base::Tagged;
use crate::units::dimension::{Dimension, BaseDimension};
use crate::units::unit::Unit;
use crate::units::composite::CompositeUnit;
use crate::units::convertible::TemperatureConvertible;
use crate::util::prism::ErrorWithPayload;

use thiserror::Error;

use std::convert::TryFrom;
use std::fmt::{self, Formatter, Debug, Display};

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

#[derive(Debug, Clone, Error)]
#[error("Expected temperature unit")]
pub struct TryIntoTemperatureUnitError<U> {
  pub unit: CompositeUnit<U>,
  _priv: (),
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

  pub fn into_value(self) -> S {
    self.value
  }

  pub fn into_unit(self) -> Unit<U> {
    self.unit
  }
}

/// Returns the single temperature unit contained within `unit`. If
/// `unit` consists of multiple units or does not contain a
/// temperature unit, returns an error.
pub fn try_into_basic_temperature_unit<U>(unit: CompositeUnit<U>) -> Result<Unit<U>, TryIntoTemperatureUnitError<U>> {
  let mut components = unit.into_inner();
  let desired_dimension = Dimension::from(BaseDimension::Temperature);
  if components.len() == 1 && components[0].exponent == 1 && components[0].dimension() == desired_dimension {
    Ok(components.swap_remove(0).unit)
  } else {
    Err(TryIntoTemperatureUnitError { unit: CompositeUnit::new(components), _priv: () })
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

impl<S: Debug + 'static, U: Debug + 'static> ErrorWithPayload<Tagged<S, U>> for TryFromTaggedError<S, U> {
  fn recover_payload(self) -> Tagged<S, U> {
    match self {
      TryFromTaggedError::DimensionMismatch(err) => Tagged::new(err.value, err.unit.into()),
      TryFromTaggedError::ExpectedSingleUnit(tagged) => tagged,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use num::pow::Pow;
  use approx::assert_abs_diff_eq;

  fn kelvins() -> Unit<f64> {
    Unit::new("K", BaseDimension::Temperature, 1.0)
      .with_temperature_offset(0.0)
  }

  fn celsius() -> Unit<f64> {
    Unit::new("degC", BaseDimension::Temperature, 1.0)
      .with_temperature_offset(273.15)
  }

  fn fahrenheit() -> Unit<f64> {
    Unit::new("degF", BaseDimension::Temperature, 5.0 / 9.0)
      .with_temperature_offset(255.3722)
  }

  fn meters() -> Unit<f64> {
    // Note: Not a temperature unit.
    Unit::new("m", BaseDimension::Length, 1.0)
  }

  #[test]
  fn test_try_new() {
    TemperatureTagged::try_new(0.0, kelvins()).unwrap();
    TemperatureTagged::try_new(0.0, fahrenheit()).unwrap();
    TemperatureTagged::try_new(0.0, meters()).unwrap_err();
  }

  #[test]
  fn test_new_or_panic() {
    TemperatureTagged::new(0.0, kelvins());
    TemperatureTagged::new(0.0, fahrenheit());
  }

  #[test]
  #[should_panic]
  fn test_new_or_panic_on_invalid_unit() {
    TemperatureTagged::new(0.0, meters());
  }

  #[test]
  fn test_convert_kelvins_celsius() {
    let kelvins = TemperatureTagged::new(0.0, kelvins());
    let celsius = kelvins.convert(celsius());
    assert_eq!(celsius.into_value(), -273.15);
  }

  #[test]
  fn test_convert_celsius_kelvins() {
    let celsius = TemperatureTagged::new(0.0, celsius());
    let kelvins = celsius.convert(kelvins());
    assert_eq!(kelvins.into_value(), 273.15);
  }

  #[test]
  fn test_convert_celsius_fahrenheit() {
    let celsius = TemperatureTagged::new(0.0, celsius());
    let fahrenheit = celsius.convert(fahrenheit());
    assert_abs_diff_eq!(fahrenheit.into_value(), 32.0, epsilon = 0.001);
  }

  #[test]
  fn test_convert_fahrenheit_celsius() {
    let fahrenheit = TemperatureTagged::new(212.0, fahrenheit());
    let celsius = fahrenheit.convert(celsius());
    assert_abs_diff_eq!(celsius.into_value(), 100.0, epsilon = 0.001);
  }

  #[test]
  #[should_panic]
  fn test_convert_invalid() {
    let fahrenheit = TemperatureTagged::new(212.0, fahrenheit());
    fahrenheit.convert(meters());
  }

  #[test]
  fn test_try_into_basic_temperature_unit() {
    let unit = CompositeUnit::from(kelvins());
    assert_eq!(try_into_basic_temperature_unit(unit).unwrap(), kelvins());
    let unit = CompositeUnit::from(celsius());
    assert_eq!(try_into_basic_temperature_unit(unit).unwrap(), celsius());
    let unit = CompositeUnit::from(celsius().pow(3));
    try_into_basic_temperature_unit(unit).unwrap_err();
    let unit = celsius() * kelvins();
    try_into_basic_temperature_unit(unit).unwrap_err();
    let unit = CompositeUnit::from(meters());
    try_into_basic_temperature_unit(unit).unwrap_err();
  }
}
