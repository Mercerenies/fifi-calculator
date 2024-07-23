
use super::dimension::Dimension;
use super::unit::Unit;
use super::unit_with_power::UnitWithPower;

use itertools::Itertools;
use num::One;
use num::pow::Pow;

use std::fmt::{self, Formatter, Display};
use std::ops::{Mul, Div};
use std::hash::{Hash, Hasher};

/// A composite unit is a formal product and quotient of named units.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompositeUnit<T> {
  // Internally, we store a composite unit as a vector, sorted
  // alphabetically by unit name. A given unit name shall only appear
  // at most once in this vector, and any unit which appears in this
  // vector shall have a nonzero exponent.
  elements: Vec<UnitWithPower<T>>,
}

/// Helper newtype struct which implements `Eq` and `Ord` to compare
/// unit names alone.
#[derive(Debug)]
struct UnitByName<T>(Unit<T>);

impl<T> CompositeUnit<T> {
  /// Constructs a new composite unit as the product of all of the
  /// inputs.
  pub fn new(inputs: impl IntoIterator<Item = UnitWithPower<T>>) -> Self {
    let mut elements: Vec<_> = inputs.into_iter()
      .map(|u| (UnitByName(u.unit), u.exponent))
      .into_grouping_map()
      .sum()
      .into_iter()
      .map(|(unit_by_name, exponent)| UnitWithPower { unit: unit_by_name.0, exponent })
      .filter(|u| u.exponent != 0)
      .collect();
    elements.sort_by(|a, b| a.unit.name().cmp(b.unit.name()));
    Self { elements }
  }

  /// The unitless composite unit. This serves as the "one" value for
  /// multiplication and division of composite units.
  pub fn unitless() -> Self {
    Self::new([])
  }

  pub fn is_empty(&self) -> bool {
    self.elements.is_empty()
  }

  /// A vector of the distinct units in this composite unit, sorted by
  /// name and tagged with their exponent. All returned exponents
  /// shall be non-zero.
  pub fn into_inner(self) -> Vec<UnitWithPower<T>> {
    self.elements
  }

  pub fn units(&self) -> &[UnitWithPower<T>] {
    &self.elements
  }

  /// An iterator over the distinct units in this composite unit,
  /// sorted by name and tagged with their exponents. All returned
  /// exponents shall be non-zero.
  pub fn iter(&self) -> impl Iterator<Item = &UnitWithPower<T>> {
    self.elements.iter()
  }

  /// The reciprocal of `self`.
  pub fn recip(mut self) -> Self {
    for elem in &mut self.elements {
      elem.exponent = - elem.exponent;
    }
    self
  }

  pub fn to_base<'a, U>(&'a self, mut amount: U) -> U
  where U: Mul<&'a T, Output = U>,
        U: Div<&'a T, Output = U> {
    for elem in &self.elements {
      amount = elem.to_base(amount);
    }
    amount
  }

  pub fn from_base<'a, U>(&'a self, mut amount: U) -> U
  where U: Mul<&'a T, Output = U>,
        U: Div<&'a T, Output = U> {
    for elem in &self.elements {
      amount = elem.from_base(amount);
    }
    amount
  }

  /// The dimension of the composite unit.
  pub fn dimension(&self) -> Dimension {
    self.elements.iter()
      .map(UnitWithPower::dimension)
      .fold(Dimension::one(), |acc, dim| acc * dim)
  }

  /// The dimension of the composite unit, considering only positive
  /// powers. All negative powers are replaced with dimensionless
  /// quantities.
  pub fn pos_dimension(&self) -> Dimension {
    self.elements
      .iter()
      .map(UnitWithPower::dimension)
      .map(|dim| dim.max(&Dimension::one()))
      .fold(Dimension::one(), |acc, dim| acc * dim)
  }

  /// The dimension of the composite unit, considering only negative
  /// powers. All positive powers are replaced with dimensionless
  /// quantities.
  pub fn neg_dimension(&self) -> Dimension {
    self.elements
      .iter()
      .map(UnitWithPower::dimension)
      .map(|dim| dim.min(&Dimension::one()))
      .fold(Dimension::one(), |acc, dim| acc * dim)
  }

  /// Expands all units which are themselves compositions, forming a
  /// new unit made up of the compositions. Any units which were not
  /// compositions are unmodified.
  pub fn expand_compositions(self) -> Self {
    let elements = self.elements
      .into_iter()
      .flat_map(|unit_with_power| unit_with_power.expand_compositions().into_inner());
    Self::new(elements)
  }
}

impl<T> From<Unit<T>> for CompositeUnit<T> {
  fn from(unit: Unit<T>) -> Self {
    CompositeUnit::new([UnitWithPower { unit, exponent: 1 }])
  }
}

impl<T> From<UnitWithPower<T>> for CompositeUnit<T> {
  fn from(unit: UnitWithPower<T>) -> Self {
    CompositeUnit::new([unit])
  }
}

impl<T> PartialEq for UnitByName<T> {
  fn eq(&self, other: &Self) -> bool {
    self.0.name() == other.0.name()
  }
}

impl<T> Eq for UnitByName<T> {}

impl<T> PartialOrd for UnitByName<T> {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl<T> Ord for UnitByName<T> {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.0.name().cmp(other.0.name())
  }
}

impl<T> Hash for UnitByName<T> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.0.name().hash(state);
  }
}

impl<T> Display for CompositeUnit<T> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    if self.elements.is_empty() {
      write!(f, "1")
    } else {
      let product = self.elements.iter()
        .map(|u| u.unit.to_string())
        .join(" ");
      write!(f, "{}", product)
    }
  }
}

impl<T, S> Mul<S> for CompositeUnit<T>
where S: Into<CompositeUnit<T>> {
  type Output = CompositeUnit<T>;

  fn mul(self, rhs: S) -> Self::Output {
    let mut elements = self.elements;
    elements.extend(rhs.into().elements);
    Self::new(elements)
  }
}

impl<T, S> Div<S> for CompositeUnit<T>
where S: Into<CompositeUnit<T>> {
  type Output = CompositeUnit<T>;

  #[allow(clippy::suspicious_arithmetic_impl)] // Multiply by reciprocal is correct
  fn div(self, rhs: S) -> Self::Output {
    self * rhs.into().recip()
  }
}

impl<T> Pow<i64> for CompositeUnit<T> {
  type Output = CompositeUnit<T>;

  fn pow(self, rhs: i64) -> Self::Output {
    Self::new(self.elements.into_iter().map(|u| u.pow(rhs)))
  }
}

impl<T> One for CompositeUnit<T> {
  fn one() -> Self {
    CompositeUnit::unitless()
  }

  fn is_one(&self) -> bool {
    self.elements.is_empty()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::units::test_utils::{kilometers, seconds, minutes};
  use crate::units::dimension::BaseDimension;

  #[test]
  fn test_new_composite_unit_simple_units() {
    let unit = CompositeUnit::new([
      UnitWithPower { unit: kilometers(), exponent: 3 },
      UnitWithPower { unit: seconds(), exponent: -1 },
    ]);
    assert_eq!(unit.elements, vec![
      UnitWithPower { unit: kilometers(), exponent: 3 },
      UnitWithPower { unit: seconds(), exponent: -1 },
    ]);
    let unit = CompositeUnit::new([
      UnitWithPower { unit: seconds(), exponent: -1 },
      UnitWithPower { unit: kilometers(), exponent: 3 },
    ]);
    assert_eq!(unit.elements, vec![
      UnitWithPower { unit: kilometers(), exponent: 3 },
      UnitWithPower { unit: seconds(), exponent: -1 },
    ]);
  }

  #[test]
  fn test_new_composite_unit_with_repeated_values() {
    let unit = CompositeUnit::new([
      UnitWithPower { unit: kilometers(), exponent: 2 },
      UnitWithPower { unit: seconds(), exponent: -1 },
      UnitWithPower { unit: kilometers(), exponent: 1 },
      UnitWithPower { unit: seconds(), exponent: -1 },
    ]);
    assert_eq!(unit.elements, vec![
      UnitWithPower { unit: kilometers(), exponent: 3 },
      UnitWithPower { unit: seconds(), exponent: -2 },
    ]);
  }

  #[test]
  fn test_dimension_of_composite_unit() {
    let unit = CompositeUnit::new([
      UnitWithPower { unit: kilometers(), exponent: 3 },
      UnitWithPower { unit: seconds(), exponent: -1 },
    ]);
    assert_eq!(
      unit.dimension(),
      BaseDimension::Length.pow(3) / BaseDimension::Time,
    );
  }

  #[test]
  fn test_dimension_of_empty_composite_unit() {
    let unit = CompositeUnit::<f64>::unitless();
    assert_eq!(unit.dimension(), Dimension::one());
  }

  #[test]
  fn test_pos_dimension_of_composite_unit() {
    let unit = CompositeUnit::new([
      UnitWithPower { unit: kilometers(), exponent: 3 },
      UnitWithPower { unit: seconds(), exponent: -1 },
    ]);
    assert_eq!(
      unit.pos_dimension(),
      BaseDimension::Length.pow(3),
    );
  }

  #[test]
  fn test_neg_dimension_of_composite_unit() {
    let unit = CompositeUnit::new([
      UnitWithPower { unit: kilometers(), exponent: 3 },
      UnitWithPower { unit: seconds(), exponent: -1 },
    ]);
    assert_eq!(
      unit.neg_dimension(),
      BaseDimension::Time.pow(-1),
    );
  }

  #[test]
  fn test_pos_dimension_of_composite_unit_with_nontrivial_dim_within() {
    let base_unit = Unit::new("mph", BaseDimension::Length / BaseDimension::Time, 1.0);
    let unit = CompositeUnit::new([
      UnitWithPower { unit: base_unit, exponent: 2 },
    ]);
    assert_eq!(
      unit.pos_dimension(),
      BaseDimension::Length.pow(2),
    );
  }

  #[test]
  fn test_neg_dimension_of_composite_unit_with_nontrivial_dim_within() {
    let base_unit = Unit::new("mph", BaseDimension::Length / BaseDimension::Time, 1.0);
    let unit = CompositeUnit::new([
      UnitWithPower { unit: base_unit, exponent: 2 },
    ]);
    assert_eq!(
      unit.neg_dimension(),
      BaseDimension::Time.pow(-2),
    );
  }

  #[test]
  fn test_composite_unit_recip() {
    let unit = CompositeUnit::new([
      UnitWithPower { unit: kilometers(), exponent: 3 },
      UnitWithPower { unit: seconds(), exponent: -2 },
    ]);
    assert_eq!(unit.recip(), CompositeUnit::new([
      UnitWithPower { unit: kilometers(), exponent: -3 },
      UnitWithPower { unit: seconds(), exponent: 2 },
    ]));
  }

  #[test]
  fn test_composite_unit_to_base() {
    let unit = CompositeUnit::new([
      UnitWithPower { unit: kilometers(), exponent: 1 },
      UnitWithPower { unit: minutes(), exponent: -1 },
    ]);
    assert_eq!(unit.to_base(18_000.0), 300_000.0);
  }

  #[test]
  fn test_composite_unit_from_base() {
    let unit = CompositeUnit::new([
      UnitWithPower { unit: kilometers(), exponent: 1 },
      UnitWithPower { unit: minutes(), exponent: -1 },
    ]);
    assert_eq!(unit.from_base(300_000.0), 18_000.0);
  }
}
