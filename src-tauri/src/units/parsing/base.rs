
use crate::units::unit::Unit;

use thiserror::Error;

/// A type capable of parsing units out of a string. A unit parser is
/// only responsible for parsing simple named units, not composite
/// units.
pub trait UnitParser<T> {
  fn parse_unit(&self, input: &str) -> Result<Unit<T>, UnitParserError>;
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

impl<T> UnitParser<T> for NullaryUnitParser {
  fn parse_unit(&self, input: &str) -> Result<Unit<T>, UnitParserError> {
    Err(UnitParserError::new(input))
  }
}

impl<'a, P, T> UnitParser<T> for &'a P
where P: UnitParser<T> + ?Sized {
  fn parse_unit(&self, input: &str) -> Result<Unit<T>, UnitParserError> {
    (**self).parse_unit(input)
  }
}
