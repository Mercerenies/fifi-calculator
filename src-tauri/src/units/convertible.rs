
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
