
//! Subsystem for converting between units and simplifying expressions
//! which contain units.

pub mod convertible;
pub mod dimension;
pub mod parsing;
pub mod prefix;
pub mod simplifier;
pub mod tagged;

mod composite;
mod unit;
mod unit_with_power;

pub use unit::{Unit, UnitCompositionError, UnitCompositionErrorReason};
pub use unit_with_power::UnitWithPower;
pub use composite::CompositeUnit;

/// Helper functions for creating test units.
#[cfg(test)]
pub(crate) mod test_utils {
  use super::*;
  use super::dimension::BaseDimension;

  pub fn meters() -> Unit<f64> {
    Unit::new("m", BaseDimension::Length, 1.0)
  }

  pub fn kilometers() -> Unit<f64> {
    Unit::new("km", BaseDimension::Length, 1000.0)
  }

  pub fn seconds() -> Unit<f64> {
    Unit::new("s", BaseDimension::Time, 1.0)
  }

  pub fn minutes() -> Unit<f64> {
    Unit::new("min", BaseDimension::Time, 60.0)
  }
}
