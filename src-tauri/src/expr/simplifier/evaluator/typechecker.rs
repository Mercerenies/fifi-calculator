
// TODO These are just prisms. Be honest with the naming :)

//! Type-checker structs for use with function arguments.

use crate::expr::Expr;
use crate::expr::number::Number;

use std::marker::PhantomData;

/// A `TypeChecker` for a given type `Input` functions as a checked
/// downcast to some other type `Self::Output`.
pub trait TypeChecker<Input> {
  type Output;

  /// Attempts to downcast `input` to the type `Self::Output`. This
  /// method shall either return the result of successfully
  /// downcasting (as an `Ok`) or the original input value (as an
  /// `Err`).
  fn narrow_type(&self, input: Input) -> Result<Self::Output, Input>;

  /// Widens an output value to its parent type. This must always
  /// succeed.
  fn widen_type(&self, value: Self::Output) -> Input;
}

/// The identity type check, which always succeeds.
#[derive(Debug, Clone)]
pub struct Identity<I> {
  _phantom: PhantomData<I>,
}

/// Type-check which downcasts an [`Expr`] to a contained [`Number`].
#[derive(Debug, Clone, Default)]
pub struct IsNumber {
  _private: (),
}

impl<I> Identity<I> {
  pub fn new() -> Self {
    Self::default()
  }
}

impl<I> Default for Identity<I> {
  fn default() -> Self {
    Identity { _phantom: PhantomData }
  }
}

impl<I> TypeChecker<I> for Identity<I> {
  type Output = I;

  fn narrow_type(&self, input: I) -> Result<I, I> {
    Ok(input)
  }

  fn widen_type(&self, value: I) -> I {
    value
  }
}

impl IsNumber {
  pub fn new() -> Self {
    Self::default()
  }
}

impl TypeChecker<Expr> for IsNumber {
  type Output = Number;

  fn narrow_type(&self, input: Expr) -> Result<Number, Expr> {
    Number::try_from(input).map_err(|err| err.original_expr)
  }

  fn widen_type(&self, value: Number) -> Expr {
    Expr::from(value)
  }
}
