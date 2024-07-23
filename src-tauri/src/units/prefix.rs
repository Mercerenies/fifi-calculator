
use super::unit::Unit;

use num::pow::Pow;

use std::ops::Mul;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct MetricPrefix {
  pub prefix_name: String,
  pub exponent: i32,
}

impl MetricPrefix {
  pub fn new(prefix_name: impl Into<String>, exponent: i32) -> MetricPrefix {
    MetricPrefix {
      prefix_name: prefix_name.into(),
      exponent,
    }
  }

  #[allow(clippy::redundant_closure)] // reason: consistency with the other closures
  pub fn apply<T>(&self, unit: Unit<T>) -> Unit<T>
  where T: Pow<i32, Output = T> + From<i32> + Mul<Output = T> {
    unit.augment(
      |name| format!("{}{}", self.prefix_name, name),
      |amount| amount * T::from(10).pow(self.exponent),
      |composed| Some(composed),
      |temperature_offset| temperature_offset,
    )
  }

  pub fn si_prefixes() -> Vec<MetricPrefix> {
    vec![
      MetricPrefix::new("Q", 30),
      MetricPrefix::new("R", 27),
      MetricPrefix::new("Y", 24),
      MetricPrefix::new("Z", 21),
      MetricPrefix::new("E", 18),
      MetricPrefix::new("P", 15),
      MetricPrefix::new("T", 12),
      MetricPrefix::new("G", 9),
      MetricPrefix::new("M", 6),
      MetricPrefix::new("k", 3),
      MetricPrefix::new("h", 2),
      MetricPrefix::new("D", 1),
      MetricPrefix::new("d", -1),
      MetricPrefix::new("c", -2),
      MetricPrefix::new("m", -3),
      // Note: We accept both "u" and "μ" for micro.
      MetricPrefix::new("u", -6),
      MetricPrefix::new("μ", -6),
      MetricPrefix::new("n", -9),
      MetricPrefix::new("p", -12),
      MetricPrefix::new("f", -15),
      MetricPrefix::new("a", -18),
      MetricPrefix::new("z", -21),
      MetricPrefix::new("y", -24),
      MetricPrefix::new("r", -27),
      MetricPrefix::new("q", -30),
    ]
  }

  pub fn si_prefixes_map() -> HashMap<String, MetricPrefix> {
    Self::si_prefixes()
      .into_iter()
      .map(|prefix| (prefix.prefix_name.clone(), prefix))
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::units::dimension::{Dimension, BaseDimension};

  #[test]
  fn apply_test() {
    let kilo_prefix = MetricPrefix::si_prefixes_map().get("k").unwrap().clone();
    let meters = Unit::<f64>::new("m", BaseDimension::Length, 1.0);
    let kilometers = kilo_prefix.apply(meters);
    assert_eq!(kilometers.name(), "km");
    assert_eq!(kilometers.dimension(), &Dimension::singleton(BaseDimension::Length));
    assert_eq!(kilometers.amount_of_base(), &1000.0);
  }
}
