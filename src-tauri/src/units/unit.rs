
use super::dimension::Dimension;
use super::composite::CompositeUnit;

use num::One;
use num::pow::Pow;
use thiserror::Error;

use std::fmt::{self, Formatter, Display};
use std::ops::{Mul, Div};
use std::cmp::Ordering;

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
  /// If present, this is a [`CompositeUnit`] made up ONLY of powers
  /// of units which are simple (per [`Unit::is_simple`]) and which
  /// has the same dimension as `self`. Simplifications can use this
  /// to "break down" the unit.
  ///
  /// Note that this unit is sometimes equivalent to `composed_units`
  /// (e.g. `mph` is equivalent to `mi / hr`), but this is not a
  /// requirement. The only requirement is that `composed_units`
  /// consist only of simple units and have the same dimension as
  /// `self`. These preconditions are enforced by this API.
  composed_units: Option<Box<CompositeUnit<T>>>,
}

/// A named unit raised to an integer power.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitWithPower<T> {
  pub unit: Unit<T>,
  pub exponent: i64,
}

/// Error type returned from [`Unit::with_composition`].
#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("{reason}")]
pub struct UnitCompositionError<T> {
  pub original_unit: Unit<T>,
  pub reason: UnitCompositionErrorReason,
  _priv: (),
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum UnitCompositionErrorReason {
  #[error("Composition of unit must be made up of simple units")]
  CompositionMustBeSimple,
  #[error("Dimension of unit composition must match dimension of the original unit")]
  DimensionMismatch,
}


impl<T> Unit<T> {
  /// Constructs a new unit, given the unit's name, dimension, and
  /// conversion factor to get to the base unit for the dimension.
  pub fn new(name: impl Into<String>, dimension: impl Into<Dimension>, amount_of_base: T) -> Self {
    Self {
      name: name.into(),
      dimension: dimension.into(),
      amount_of_base,
      composed_units: None,
    }
  }

  pub fn name(&self) -> &str {
    &self.name
  }

  /// A unit is considered simple if its dimension is simple. That is,
  /// this method is equivalent to `self.dimension().is_simple()`.
  pub fn is_simple(&self) -> bool {
    self.dimension().is_simple()
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

  /// Applies functions modifying the name, amount, and composition of
  /// this unit.
  ///
  /// This is most commonly used to generate derived units, such as
  /// creating "kilometers" from the definition of a "meter".
  pub fn augment<F, G, H, U>(self, name_fn: F, amount_of_base_fn: G, composed_fn: H) -> Unit<U>
  where F: FnOnce(String) -> String,
        G: FnOnce(T) -> U,
        H: FnOnce(CompositeUnit<T>) -> Option<CompositeUnit<U>> {
    let composed_units = self.composed_units.and_then(|u| {
      composed_fn(*u).map(Box::new)
    });
    Unit {
      name: name_fn(self.name),
      dimension: self.dimension,
      amount_of_base: amount_of_base_fn(self.amount_of_base),
      composed_units,
    }
  }

  /// Assigns composed units to this unit, builder-style. If this unit
  /// already has information about composed units, that information
  /// will be overwritten. Return the modified unit on success, or a
  /// [`UnitCompositionError`] (containing the original unit) on
  /// failure.
  ///
  /// The composed unit that makes up this unit must be made of a
  /// product, quotient, and powers of simple units, and the dimension
  /// must match the dimension of `self`.
  pub fn try_with_composed(mut self, composed_units: CompositeUnit<T>) -> Result<Self, UnitCompositionError<T>> {
    if self.dimension() != &composed_units.dimension() {
      return Err(UnitCompositionError {
        original_unit: self,
        reason: UnitCompositionErrorReason::DimensionMismatch,
        _priv: (),
      });
    }
    for unit_with_power in composed_units.iter() {
      if !unit_with_power.unit.is_simple() {
        return Err(UnitCompositionError {
          original_unit: self,
          reason: UnitCompositionErrorReason::CompositionMustBeSimple,
          _priv: (),
        });
      }
    }
    self.composed_units = Some(Box::new(composed_units));
    Ok(self)
  }

  /// Assigns composed units to this unit, builder-style. Panics if
  /// the composition is not valid. For a non-panicking variant, use
  /// [`Unit::try_with_composed`].
  pub fn with_composed(self, composed_units: CompositeUnit<T>) -> Self {
    self.try_with_composed(composed_units).unwrap_or_else(|err| {
      panic!("{err}");
    })
  }

  /// Removes any composed unit information from `self`.
  pub fn without_composed(mut self) -> Self {
    self.composed_units = None;
    self
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

impl<T> From<UnitWithPower<T>> for CompositeUnit<T> {
  fn from(unit: UnitWithPower<T>) -> Self {
    CompositeUnit::new([unit])
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

impl<T> Pow<i64> for Unit<T> {
  type Output = UnitWithPower<T>;

  fn pow(self, rhs: i64) -> Self::Output {
    UnitWithPower { unit: self, exponent: rhs }
  }
}

impl<T, S> Mul<S> for Unit<T>
where S: Into<CompositeUnit<T>> {
  type Output = CompositeUnit<T>;

  fn mul(self, rhs: S) -> Self::Output {
    CompositeUnit::from(self) * rhs
  }
}

impl<T, S> Div<S> for Unit<T>
where S: Into<CompositeUnit<T>> {
  type Output = CompositeUnit<T>;

  fn div(self, rhs: S) -> Self::Output {
    CompositeUnit::from(self) / rhs
  }
}

impl<T, S> Mul<S> for UnitWithPower<T>
where S: Into<CompositeUnit<T>> {
  type Output = CompositeUnit<T>;

  fn mul(self, rhs: S) -> Self::Output {
    CompositeUnit::from(self) * rhs
  }
}

impl<T, S> Div<S> for UnitWithPower<T>
where S: Into<CompositeUnit<T>> {
  type Output = CompositeUnit<T>;

  fn div(self, rhs: S) -> Self::Output {
    CompositeUnit::from(self) / rhs
  }
}

impl<T> Pow<i64> for UnitWithPower<T> {
  type Output = UnitWithPower<T>;

  fn pow(self, rhs: i64) -> Self::Output {
    UnitWithPower { unit: self.unit, exponent: self.exponent * rhs }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::units::dimension::{Dimension, BaseDimension};
  use crate::units::test_utils::{meters, kilometers, seconds, minutes};

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
  fn test_unit_construction_with_composite_unit() {
    let base_unit = Unit::new("L", BaseDimension::Length.pow(3), 1.0);
    base_unit
      .try_with_composed(CompositeUnit::new(vec![UnitWithPower { unit: meters(), exponent: 3 }]))
      .unwrap();
  }

  #[test]
  fn test_unit_construction_with_composite_unit_non_simple_composition() {
    let base_unit = Unit::new("L", BaseDimension::Length.pow(3), 1.0);
    let err = base_unit.clone()
      .try_with_composed(CompositeUnit::new(vec![UnitWithPower { unit: base_unit, exponent: 1 }]))
      .unwrap_err();
    assert_eq!(err.reason, UnitCompositionErrorReason::CompositionMustBeSimple);
  }

  #[test]
  fn test_unit_construction_with_composite_unit_dimension_mismatch() {
    let base_unit = Unit::new("L", BaseDimension::Length.pow(3), 1.0);
    let err = base_unit
      .clone()
      .try_with_composed(CompositeUnit::new(vec![UnitWithPower { unit: kilometers(), exponent: 2 }]))
      .unwrap_err();
    assert_eq!(err.reason, UnitCompositionErrorReason::DimensionMismatch);
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
}
