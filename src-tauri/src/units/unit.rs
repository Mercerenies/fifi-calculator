
use super::dimension::Dimension;

use itertools::Itertools;
use num::One;

use std::fmt::{self, Formatter, Display};
use std::ops::{Mul, Div};
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
#[derive(Debug, Clone)]
pub struct CompositeUnit<T> {
  // Internally, we store a composite unit as a vector, sorted
  // alphabetically by unit name. A given unit name shall only appear
  // at most once in this vector, and any unit which appears in this
  // vector shall have a nonzero exponent.
  elements: Vec<UnitWithPower<T>>,
}

/// A named unit raised to an integer power.
#[derive(Debug, Clone)]
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
  pub fn new(name: impl Into<String>, dimension: Dimension, amount_of_base: T) -> Self {
    Self {
      name: name.into(),
      dimension,
      amount_of_base,
    }
  }

  pub fn name(&self) -> &str {
    &self.name
  }

  pub fn dimension(&self) -> &Dimension {
    &self.dimension
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

  pub fn to_base<'a>(&'a self, mut amount: T) -> T
  where T: Mul<&'a T, Output = T>,
        T: Div<&'a T, Output = T> {
    for elem in &self.elements {
      amount = elem.to_base(amount);
    }
    amount
  }

  pub fn from_base<'a>(&'a self, mut amount: T) -> T
  where T: Mul<&'a T, Output = T>,
        T: Div<&'a T, Output = T> {
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

  pub fn to_base<'a>(&'a self, mut amount: T) -> T
  where T: Mul<&'a T, Output = T>,
        T: Div<&'a T, Output = T> {
    if self.exponent > 0 {
      for _ in 0..self.exponent {
        amount = self.unit.to_base(amount);
      }
    } else if self.exponent < 0 {
      for _ in 0..(-self.exponent) {
        amount = self.unit.from_base(amount);
      }
    }
    amount
  }

  pub fn from_base<'a>(&'a self, mut amount: T) -> T
  where T: Mul<&'a T, Output = T>,
        T: Div<&'a T, Output = T> {
    if self.exponent > 0 {
      for _ in 0..self.exponent {
        amount = self.unit.from_base(amount);
      }
    } else if self.exponent < 0 {
      for _ in 0..(-self.exponent) {
        amount = self.unit.to_base(amount);
      }
    }
    amount
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
