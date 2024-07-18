
//! Simplification engine for units with matching dimensions which can
//! be safely canceled off.

use super::unit::CompositeUnit;

/// Returns a composite unit with the same dimension as the input but
/// with any units of compatible dimension canceled off.
pub fn simplify_compatible_units<T>(unit: CompositeUnit<T>) -> CompositeUnit<T> {
  let unit_terms = unit.into_inner();
  todo!()
}
