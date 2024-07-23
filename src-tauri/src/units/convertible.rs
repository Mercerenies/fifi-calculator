
//! This module defines traits for the minimal requirements of a type
//! that can be used in unit conversion.

use std::ops::{Mul, Div};

/// A type which implements this trait can be used as the scalar in a
/// unit conversion involving unit values of type `U`. This trait is
/// merely the intersection of `Mul<&U>` and `Div<&U>` and should not
/// be implemented by hand. Any type implementing the prior two traits
/// automatically implements this one.
pub trait UnitConvertible<U>: for<'a> Mul<&'a U, Output = Self> + for<'a> Div<&'a U, Output = Self> {}

impl<S, U> UnitConvertible<U> for S
where S: for<'a> Mul<&'a U, Output = S> + for<'a> Div<&'a U, Output = S> {}

/// A type which implements this trait can be used as the scalar in a
/// temperature-sensitive unit conversion. This trait depends on
/// [`UnitConvertible`] but must be implemented by hand, as its
/// functionality is not provided by any other specific traits.
pub trait TemperatureConvertible<U>: UnitConvertible<U> {
  /// The output type produced after offsetting a value of type `Self`
  /// by an offset of type `U`.
  type Output;

  /// Offset `self` by the given offset value.
  fn offset(self, offset: Option<&U>) -> <Self as TemperatureConvertible<U>>::Output;

  /// Subtracts the offset value from the given output. Note that this
  /// might NOT be an exact inverse to `Self::offset`, but it should
  /// produce a mathematical expression that represents the same
  /// quantity in spirit.
  fn unoffset(input: <Self as TemperatureConvertible<U>>::Output, offset: Option<&U>) -> Self;
}

impl TemperatureConvertible<f64> for f64 {
  type Output = f64;

  fn offset(self, offset: Option<&f64>) -> f64 {
    self + offset.unwrap_or(&0.0)
  }

  fn unoffset(input: f64, offset: Option<&f64>) -> f64 {
    input - offset.unwrap_or(&0.0)
  }
}
