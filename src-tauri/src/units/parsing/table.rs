
use super::base::{UnitParser, UnitParserError};
use crate::units::unit::Unit;

use std::collections::HashMap;

/// A [`UnitParser`] which looks up the given name in a pre-determined
/// hash table.
pub struct TableBasedParser<T> {
  pub table: HashMap<String, Unit<T>>,
}

impl<T: Clone> UnitParser<T> for TableBasedParser<T> {
  fn parse_unit(&self, input: &str) -> Result<Unit<T>, UnitParserError> {
    self.table.get(input)
      .map(|u| u.clone())
      .ok_or_else(|| UnitParserError::new(input.to_owned()))
  }
}
