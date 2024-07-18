
//! Simplification engine for units with matching dimensions which can
//! be safely canceled off.

use super::unit::{CompositeUnit, UnitWithPower};
use crate::util::double_borrow_mut;

/// Returns a composite unit with the same dimension as the input but
/// with any units of compatible dimension canceled off. Specifically,
/// if two units `a` and `b` appear in `unit` such that
///
/// 1. `a.unit` and `b.unit` have the _same_ dimension, AND
///
/// 2. `a.exponent` and `b.exponent` have opposite signs,
///
/// then this algorithm will cancel off as many of the units as
/// possible until one of them has power zero.
///
/// This algorithm is left-biased, so if there are several units that
/// could be canceled, the ones further to the left will be canceled
/// first. Note that [`CompositeUnit`] stores its units in
/// alphabetical order, so this effectively means this algorithm will
/// prefer to cancel units whose names are earlier in the alphabet
/// before those whose names appear later in the alphabet.
pub fn simplify_compatible_units<T>(unit: CompositeUnit<T>) -> CompositeUnit<T> {
  let mut unit_terms = unit.into_inner();
  for i in 0..unit_terms.len() {
    for j in i+1..unit_terms.len() {
      if is_compatible(&unit_terms[i], &unit_terms[j]) {
        let (left, right) = double_borrow_mut(&mut unit_terms, i, j);
        reduce_compatible(left, right);
      }
    }
  }
  CompositeUnit::new(unit_terms)
}

fn is_compatible<T>(left: &UnitWithPower<T>, right: &UnitWithPower<T>) -> bool {
  left.unit.dimension() == right.unit.dimension() &&
    left.exponent.signum() != right.exponent.signum()
}

fn reduce_compatible<T>(left: &mut UnitWithPower<T>, right: &mut UnitWithPower<T>) {
  assert!(is_compatible(left, right));
  let d = i64::min(left.exponent.abs(), right.exponent.abs());
  left.exponent -= d * left.exponent.signum();
  right.exponent -= d * right.exponent.signum();
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::units::unit::Unit;
  use crate::units::dimension::BaseDimension;

  use num::pow::Pow;

  fn meters() -> Unit<f64> {
    Unit::new("m", BaseDimension::Length, 1.0)
  }

  fn liters() -> Unit<f64> {
    Unit::new("L", BaseDimension::Length.pow(3), 0.001)
  }

  fn kilometers() -> Unit<f64> {
    Unit::new("km", BaseDimension::Length, 1000.0)
  }

  fn seconds() -> Unit<f64> {
    Unit::new("s", BaseDimension::Time, 1.0)
  }

  fn hertz() -> Unit<f64> {
    Unit::new("Hz", BaseDimension::Time.pow(-1), 1.0)
  }

  fn minutes() -> Unit<f64> {
    Unit::new("min", BaseDimension::Time, 60.0)
  }

  #[test]
  fn test_is_compatible() {
    assert!(!is_compatible(
      &UnitWithPower { unit: seconds(), exponent: 1 },
      &UnitWithPower { unit: hertz(), exponent: 1 },
    ));
    assert!(is_compatible(
      &UnitWithPower { unit: seconds(), exponent: 1 },
      &UnitWithPower { unit: minutes(), exponent: -1 },
    ));
    assert!(is_compatible(
      &UnitWithPower { unit: hertz(), exponent: 1 },
      &UnitWithPower { unit: hertz(), exponent: -1 },
    ));
    assert!(is_compatible(
      &UnitWithPower { unit: seconds(), exponent: 3 },
      &UnitWithPower { unit: minutes(), exponent: -2 },
    ));
    assert!(!is_compatible(
      &UnitWithPower { unit: seconds(), exponent: 1 },
      &UnitWithPower { unit: hertz(), exponent: -1 },
    ));
    assert!(!is_compatible(
      &UnitWithPower { unit: seconds(), exponent: 1 },
      &UnitWithPower { unit: minutes(), exponent: 1 },
    ));
    assert!(!is_compatible(
      &UnitWithPower { unit: seconds(), exponent: 1 },
      &UnitWithPower { unit: kilometers(), exponent: 1 },
    ));
    assert!(is_compatible(
      &UnitWithPower { unit: liters(), exponent: 1 },
      &UnitWithPower { unit: liters(), exponent: -2 },
    ));
    // Note: Even though the dimensions (after applying powers) would
    // cancel off perfectly, they're not compatible under this basic
    // simplifier since the base dimensions differ.
    assert!(!is_compatible(
      &UnitWithPower { unit: liters(), exponent: 1 },
      &UnitWithPower { unit: meters(), exponent: -3 },
    ));
  }

  #[test]
  fn test_reduce_compatible() {
    let mut a = UnitWithPower { unit: seconds(), exponent: 2 };
    let mut b = UnitWithPower { unit: minutes(), exponent: -1 };
    reduce_compatible(&mut a, &mut b);
    assert_eq!(a.exponent, 1);
    assert_eq!(b.exponent, 0);

    let mut a = UnitWithPower { unit: hertz(), exponent: -3 };
    let mut b = UnitWithPower { unit: hertz(), exponent: 3 };
    reduce_compatible(&mut a, &mut b);
    assert_eq!(a.exponent, 0);
    assert_eq!(b.exponent, 0);

    let mut a = UnitWithPower { unit: liters(), exponent: -5 };
    let mut b = UnitWithPower { unit: liters(), exponent: 4 };
    reduce_compatible(&mut a, &mut b);
    assert_eq!(a.exponent, -1);
    assert_eq!(b.exponent, 0);
  }

  #[test]
  fn test_simplify_compatible_units() {
    let unit = CompositeUnit::new([
      UnitWithPower { unit: kilometers(), exponent: -1 },
      UnitWithPower { unit: meters(), exponent: 1 },
      UnitWithPower { unit: minutes(), exponent: -2 },
      UnitWithPower { unit: seconds(), exponent: 3 },
    ]);
    let unit = simplify_compatible_units(unit);
    assert_eq!(unit, CompositeUnit::new([
      UnitWithPower { unit: seconds(), exponent: 1 },
    ]));
  }
}
