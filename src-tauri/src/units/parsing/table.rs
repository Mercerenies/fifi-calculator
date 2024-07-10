
use super::base::{UnitParser, UnitParserError};
use crate::units::unit::Unit;
use crate::units::dimension::BaseDimension;

use std::collections::HashMap;

/// A [`UnitParser`] which looks up the given name in a pre-determined
/// hash table.
pub struct TableBasedParser<T> {
  pub table: HashMap<String, Unit<T>>,
  pub base_units: Box<dyn Fn(BaseDimension) -> Unit<T> + Send + Sync>,
}

impl<T> TableBasedParser<T> {
  pub fn new<F>(table: HashMap<String, Unit<T>>, base_units: F) -> Self
  where F: Fn(BaseDimension) -> Unit<T> + Send + Sync + 'static {
    Self {
      table,
      base_units: Box::new(base_units),
    }
  }
}

impl<T: Clone> UnitParser<T> for TableBasedParser<T> {
  fn parse_unit(&self, input: &str) -> Result<Unit<T>, UnitParserError> {
    self.table.get(input)
      .cloned()
      .ok_or_else(|| UnitParserError::new(input.to_owned()))
  }

  fn base_unit(&self, dimension: BaseDimension) -> Unit<T> {
    (self.base_units)(dimension)
  }
}

#[cfg(test)]
pub(crate) mod test_utils {
  use super::*;
  use crate::units::dimension::BaseDimension;

  pub fn sample_table() -> TableBasedParser<f64> {
    let mut table = HashMap::new();
    table.insert("m".to_owned(), Unit::new("m", BaseDimension::Length, 1.0));
    table.insert("s".to_owned(), Unit::new("s", BaseDimension::Time, 1.0));
    table.insert("min".to_owned(), Unit::new("min", BaseDimension::Time, 60.0));
    TableBasedParser::new(table, |_| panic!("Should not be called"))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::units::dimension::BaseDimension;
  use super::test_utils::sample_table;

  #[test]
  fn test_lookup() {
    let table = sample_table();
    assert_eq!(
      table.parse_unit("m"),
      Ok(Unit::new("m", BaseDimension::Length, 1.0)),
    );
    assert_eq!(table.parse_unit("xyz"), Err(UnitParserError::new("xyz")));
    assert_eq!(table.parse_unit("M"), Err(UnitParserError::new("M"))); // Note: Case sensitive
  }
}
