
use super::base::{UnitParser, UnitParserError};
use crate::units::prefix::MetricPrefix;
use crate::units::unit::Unit;

use num::pow::Pow;

use std::ops::Mul;
use std::collections::HashMap;

pub struct PrefixParser<P> {
  inner: P,
  prefixes: HashMap<String, MetricPrefix>,
  longest_prefix_len: usize,
}

impl<P> PrefixParser<P> {
  pub fn new(inner: P, prefixes: impl IntoIterator<Item = MetricPrefix>) -> Self {
    let prefixes: HashMap<_, _> = prefixes.into_iter().map(|p| (p.prefix_name.clone(), p)).collect();
    let longest_prefix_len = prefixes.keys().map(|s| s.len()).max().unwrap_or(0);
    Self { inner, prefixes, longest_prefix_len }
  }

  /// A `PrefixParser` based on the given inner parser, which accepts
  /// standard SI prefixes, as per [`MetricPrefix::si_prefixes`].
  pub fn new_si(inner: P) -> Self {
    Self::new(inner, MetricPrefix::si_prefixes())
  }
}

impl<T, P> UnitParser<T> for PrefixParser<P>
where P: UnitParser<T>,
      T: Pow<i32, Output = T> + From<i32> + Mul<Output = T> {
  fn parse_unit(&self, input: &str) -> Result<Unit<T>, UnitParserError> {
    self.inner.parse_unit(input).or_else(|err| {
      for i in 1..=self.longest_prefix_len {
        if !input.is_char_boundary(i) {
          continue;
        }
        let (prefix, input) = input.split_at(i);
        if let Some(prefix) = self.prefixes.get(prefix) {
          if let Ok(unit) = self.inner.parse_unit(input) {
            return Ok(prefix.apply(unit));
          }
        }
      }
      Err(err)
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::units::parsing::table::test_utils::sample_table;
  use crate::units::dimension::BaseDimension;

  #[test]
  fn test_parse_base_values() {
    let parser = PrefixParser::new_si(sample_table());
    assert_eq!(parser.parse_unit("m"), Ok(Unit::new("m", BaseDimension::Length, 1.0)));
    assert_eq!(parser.parse_unit("s"), Ok(Unit::new("s", BaseDimension::Time, 1.0)));
    assert_eq!(parser.parse_unit("min"), Ok(Unit::new("min", BaseDimension::Time, 60.0)));
  }

  #[test]
  fn test_parse_with_prefix() {
    let parser = PrefixParser::new_si(sample_table());
    assert_eq!(parser.parse_unit("km"), Ok(Unit::new("km", BaseDimension::Length, 1000.0)));
    assert_eq!(parser.parse_unit("ms"), Ok(Unit::new("ms", BaseDimension::Time, 0.001)));
    // Bet you've never seen someone work in units of "centi-minutes" before ;)
    assert_eq!(parser.parse_unit("cmin"), Ok(Unit::new("cmin", BaseDimension::Time, 0.6)));
  }

  #[test]
  fn test_parse_invalid() {
    let parser = PrefixParser::new_si(sample_table());
    parser.parse_unit("").unwrap_err();
    parser.parse_unit("😇😇😇").unwrap_err(); // Test multi-byte char in front position.
    parser.parse_unit("ABCDEFG").unwrap_err();
    parser.parse_unit("Km").unwrap_err();
    parser.parse_unit("kM").unwrap_err();
    parser.parse_unit("kkm").unwrap_err(); // Do not allow multiple prefixes.
    parser.parse_unit("mse").unwrap_err();
  }

  #[test]
  fn test_parse_ambiguous() {
    // In case of a unit that can be parsed either as a named unit or
    // a prefixed unit, prefer the named parse. In this example, "min"
    // can technically be parsed as "minutes" or "milli-inches". The
    // former parse should be preferred.
    let table = {
      let mut table = sample_table();
      table.table.insert("in".to_owned(), Unit::new("in", BaseDimension::Length, 0.0254));
      table
    };
    let parser = PrefixParser::new_si(table);
    assert_eq!(
      parser.parse_unit("min"),
      Ok(Unit::new("min", BaseDimension::Time, 60.0)),
    );
  }
}
