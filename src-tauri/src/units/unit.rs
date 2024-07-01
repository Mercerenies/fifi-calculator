
use super::dimension::Dimension;

use itertools::Itertools;
use num::One;
use num::pow::Pow;

use std::fmt::{self, Formatter, Display};
use std::ops::{Mul, Div};
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

/// A unit is a named quantity in some [`Dimension`] which can be
/// converted to the "base" unit of that dimension.
///
/// Units are always stored with reference to an underlying scalar
/// type, such as `f64`. Custom numerical types can also be used.
///
/// Our definition of "base unit" matches that of Emacs Calc.
/// Specifically, our definition of "base unit" is equal to the SI
/// base unit, except that we use grams for mass rather than
/// kilograms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unit<T> {
  name: String,
  dimension: Dimension,
  /// The amount of the base unit that is equal to one of this unit.
  amount_of_base: T,
}

/// A composite unit is a formal product and quotient of named units.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompositeUnit<T> {
  // Internally, we store a composite unit as a vector, sorted
  // alphabetically by unit name. A given unit name shall only appear
  // at most once in this vector, and any unit which appears in this
  // vector shall have a nonzero exponent.
  elements: Vec<UnitWithPower<T>>,
}

/// A named unit raised to an integer power.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitWithPower<T> {
  pub unit: Unit<T>,
  pub exponent: i64,
}

/// Helper newtype struct which implements `Eq` and `Ord` to compare
/// unit names alone.
#[derive(Debug)]
struct UnitByName<T>(Unit<T>);

impl<T> Unit<T> {
  /// Constructs a new unit, given the unit's name, dimension, and
  /// conversion factor to get to the base unit for the dimension.
  pub fn new(name: impl Into<String>, dimension: impl Into<Dimension>, amount_of_base: T) -> Self {
    Self {
      name: name.into(),
      dimension: dimension.into(),
      amount_of_base,
    }
  }

  pub fn name(&self) -> &str {
    &self.name
  }

  pub fn dimension(&self) -> &Dimension {
    &self.dimension
  }

  pub fn amount_of_base(&self) -> &T {
    &self.amount_of_base
  }

  /// Converts a scalar quantity from this unit to the base unit
  /// corresponding to this dimension.
  pub fn to_base<'a, U>(&'a self, amount: U) -> <U as Mul<&'a T>>::Output
  where U: Mul<&'a T> {
    amount * &self.amount_of_base
  }

  /// Converts a scalar quantity from the base unit of this dimension
  /// into this unit.
  pub fn from_base<'a, U>(&'a self, amount: U) -> <U as Div<&'a T>>::Output
  where U: Div<&'a T> {
    amount / &self.amount_of_base
  }

  /// Applies functions modifying the name and amount of this unit.
  ///
  /// This is most commonly used to generate derived units, such as
  /// creating "kilometers" from the definition of a "meter".
  pub fn augment<F, G, U>(self, name_fn: F, amount_of_base_fn: G) -> Unit<U>
  where F: FnOnce(String) -> String,
        G: FnOnce(T) -> U {
    Unit {
      name: name_fn(self.name),
      dimension: self.dimension,
      amount_of_base: amount_of_base_fn(self.amount_of_base),
    }
  }
}

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
    elements.sort_by(|a, b| a.unit.name.cmp(&b.unit.name));
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

  pub fn dimension(&self) -> Dimension {
    self.elements.iter()
      .map(UnitWithPower::dimension)
      .fold(Dimension::one(), |acc, dim| acc * dim)
  }
}

impl<T> UnitWithPower<T> {
  pub fn dimension(&self) -> Dimension {
    self.unit.dimension().pow(self.exponent)
  }

  pub fn to_base<'a, U>(&'a self, mut amount: U) -> U
  where U: Mul<&'a T, Output = U>,
        U: Div<&'a T, Output = U> {
    match self.exponent.cmp(&0) {
      Ordering::Greater => {
        for _ in 0..self.exponent {
          amount = self.unit.to_base(amount);
        }
      }
      Ordering::Less => {
        for _ in 0..(-self.exponent) {
          amount = self.unit.from_base(amount);
        }
      }
      Ordering::Equal => {},
    }
    amount
  }

  pub fn from_base<'a, U>(&'a self, mut amount: U) -> U
  where U: Mul<&'a T, Output = U>,
        U: Div<&'a T, Output = U> {
    match self.exponent.cmp(&0) {
      Ordering::Greater => {
        for _ in 0..self.exponent {
          amount = self.unit.from_base(amount);
        }
      }
      Ordering::Less => {
        for _ in 0..(-self.exponent) {
          amount = self.unit.to_base(amount);
        }
      }
      Ordering::Equal => {},
    }
    amount
  }
}

impl<T> From<Unit<T>> for CompositeUnit<T> {
  fn from(unit: Unit<T>) -> Self {
    CompositeUnit::new([UnitWithPower { unit, exponent: 1 }])
  }
}

impl<T> Display for Unit<T> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.name)
  }
}

impl<T> Display for UnitWithPower<T> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    if self.exponent == 1 {
      write!(f, "{}", self.unit)
    } else {
      write!(f, "{}^{}", self.unit, self.exponent)
    }
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

impl<T> PartialEq for UnitByName<T> {
  fn eq(&self, other: &Self) -> bool {
    self.0.name == other.0.name
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
    self.0.name.cmp(&other.0.name)
  }
}

impl<T> Hash for UnitByName<T> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.0.name.hash(state);
  }
}

impl<T> Mul for CompositeUnit<T> {
  type Output = CompositeUnit<T>;

  fn mul(self, rhs: Self) -> Self::Output {
    let mut elements = self.elements;
    elements.extend(rhs.elements);
    Self::new(elements)
  }
}

impl<T> Div for CompositeUnit<T> {
  type Output = CompositeUnit<T>;

  #[allow(clippy::suspicious_arithmetic_impl)] // Multiply by reciprocal is correct
  fn div(self, rhs: Self) -> Self::Output {
    self * rhs.recip()
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
  use crate::units::dimension::{Dimension, BaseDimension};

  fn meters() -> Unit<f64> {
    Unit::new("m", BaseDimension::Length, 1.0)
  }

  fn kilometers() -> Unit<f64> {
    Unit::new("km", BaseDimension::Length, 1000.0)
  }

  fn seconds() -> Unit<f64> {
    Unit::new("s", BaseDimension::Time, 1.0)
  }

  fn minutes() -> Unit<f64> {
    Unit::new("min", BaseDimension::Time, 60.0)
  }

  #[test]
  fn test_unit_fields() {
    assert_eq!(meters().name(), "m");
    assert_eq!(meters().dimension(), &Dimension::singleton(BaseDimension::Length));
  }

  #[test]
  fn test_to_base_from_base_on_base_unit() {
    assert_eq!(meters().to_base(100.0), 100.0);
    assert_eq!(seconds().to_base(0.5), 0.5);
    assert_eq!(meters().from_base(100.0), 100.0);
    assert_eq!(seconds().from_base(0.5), 0.5);
  }

  #[test]
  fn test_unit_to_base() {
    assert_eq!(kilometers().to_base(2.0), 2000.0);
    assert_eq!(minutes().to_base(30.0), 1800.0);
  }

  #[test]
  fn test_unit_from_base() {
    assert_eq!(kilometers().from_base(2000.0), 2.0);
    assert_eq!(minutes().from_base(1800.0), 30.0);
  }

  #[test]
  fn test_unit_with_power_dimension() {
    let unit = UnitWithPower { unit: kilometers(), exponent: 3 };
    assert_eq!(unit.dimension(), Dimension::singleton(BaseDimension::Length).pow(3));
    let unit = UnitWithPower { unit: seconds(), exponent: -2 };
    assert_eq!(unit.dimension(), Dimension::singleton(BaseDimension::Time).pow(-2));
    let unit = UnitWithPower { unit: kilometers(), exponent: 0 };
    assert_eq!(unit.dimension(), Dimension::one());
  }

  #[test]
  fn test_unit_with_power_to_base() {
    let unit = UnitWithPower { unit: kilometers(), exponent: 3 };
    assert_eq!(unit.to_base(2.0), 2_000_000_000.0);
    let unit = UnitWithPower { unit: kilometers(), exponent: -1 };
    assert_eq!(unit.to_base(2_000.0), 2.0);
    let unit = UnitWithPower { unit: kilometers(), exponent: 0 };
    assert_eq!(unit.to_base(199.0), 199.0);
  }

  #[test]
  fn test_unit_with_power_from_base() {
    let unit = UnitWithPower { unit: kilometers(), exponent: 3 };
    assert_eq!(unit.from_base(2_000_000_000.0), 2.0);
    let unit = UnitWithPower { unit: kilometers(), exponent: -1 };
    assert_eq!(unit.from_base(2.0), 2_000.0);
    let unit = UnitWithPower { unit: kilometers(), exponent: 0 };
    assert_eq!(unit.from_base(199.0), 199.0);
  }

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
