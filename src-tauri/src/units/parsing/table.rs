
use super::base::{UnitParser, UnitParserError};
use crate::units::unit::Unit;

use std::collections::HashMap;

/// A [`UnitParser`] which looks up the given name in a pre-determined
/// hash table.
#[derive(Debug, Clone)]
pub struct TableBasedParser<T> {
  pub table: HashMap<String, Unit<T>>,
}

impl<T> TableBasedParser<T> {
  pub fn new(table: HashMap<String, Unit<T>>) -> Self {
    Self { table }
  }
}

impl<T: Clone> UnitParser<T> for TableBasedParser<T> {
  fn parse_unit(&self, input: &str) -> Result<Unit<T>, UnitParserError> {
    self.table.get(input)
      .map(|u| u.clone())
      .ok_or_else(|| UnitParserError::new(input.to_owned()))
  }
}

#[cfg(test)]
pub(crate) mod test_utils {
  use super::*;
  use crate::units::dimension::{Dimension, BaseDimension};

  pub fn sample_table() -> TableBasedParser<f64> {
    let mut table = HashMap::new();
    table.insert("m".to_owned(), Unit::new("m", Dimension::singleton(BaseDimension::Length), 1.0));
    table.insert("s".to_owned(), Unit::new("s", Dimension::singleton(BaseDimension::Time), 1.0));
    table.insert("min".to_owned(), Unit::new("min", Dimension::singleton(BaseDimension::Time), 60.0));
    TableBasedParser { table }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::units::dimension::{Dimension, BaseDimension};
  use super::test_utils::sample_table;

  #[test]
  fn test_lookup() {
    let table = sample_table();
    assert_eq!(
      table.parse_unit("m"),
      Ok(Unit::new("m", Dimension::singleton(BaseDimension::Length), 1.0)),
    );
    assert_eq!(table.parse_unit("xyz"), Err(UnitParserError::new("xyz")));
    assert_eq!(table.parse_unit("M"), Err(UnitParserError::new("M"))); // Note: Case sensitive
  }
}
