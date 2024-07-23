
use crate::units::{Unit, UnitWithPower, CompositeUnit};
use crate::units::dimension::{BaseDimension, Dimension};

use thiserror::Error;
use num::One;

/// A type capable of parsing units out of a string. A unit parser is
/// only responsible for parsing simple named units, not composite
/// units.
pub trait UnitParser<T> {
  /// Parses the string as a unit, or produces an error if the string
  /// cannot be parsed as a unit.
  fn parse_unit(&self, input: &str) -> Result<Unit<T>, UnitParserError>;

  /// Produces the base unit for this base dimension. This is the unit
  /// that all other units implicitly convert through.
  fn base_unit(&self, dimension: BaseDimension) -> Unit<T>;

  /// Produces the base unit for this dimension, built up using
  /// [`UnitParser::base_unit`].
  fn base_composite_unit(&self, dimension: &Dimension) -> CompositeUnit<T> {
    let component_units = dimension.components()
      .map(|(dimension, exponent)| UnitWithPower { unit: self.base_unit(dimension), exponent });
    CompositeUnit::new(component_units)
  }
}

/// Nullary units parser. Always fails.
#[derive(Debug, Clone)]
pub struct NullaryUnitParser;

#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("Failed to parse '{input}' as a unit")]
pub struct UnitParserError {
  pub input: String,
}

impl UnitParserError {
  pub fn new(input: impl Into<String>) -> Self {
    Self { input: input.into() }
  }
}

impl<T: One> UnitParser<T> for NullaryUnitParser {
  fn parse_unit(&self, input: &str) -> Result<Unit<T>, UnitParserError> {
    Err(UnitParserError::new(input))
  }

  fn base_unit(&self, dimension: BaseDimension) -> Unit<T> {
    // The exact name of the unit doesn't really matter here, since
    // we'll never successfully parse into it. But we just want it to
    // be a valid variable name (hence, the empty string is a poor
    // choice).
    Unit::new("unit", dimension, T::one())
  }
}

impl<'a, P, T> UnitParser<T> for &'a P
where P: UnitParser<T> + ?Sized {
  fn parse_unit(&self, input: &str) -> Result<Unit<T>, UnitParserError> {
    (**self).parse_unit(input)
  }

  fn base_unit(&self, dimension: BaseDimension) -> Unit<T> {
    (**self).base_unit(dimension)
  }
}
