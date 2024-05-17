
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

/// Lift a type-checker into each element of a `Vec`.
#[derive(Debug, Clone, Default)]
pub struct OnVec<C, T> {
  inner: C,
  _phantom: PhantomData<T>,
}

impl<I> Identity<I> {
  pub fn new() -> Self {
    Self::default()
  }
}

impl IsNumber {
  pub fn new() -> Self {
    Self::default()
  }
}

impl<C, T> OnVec<C, T>
where C: TypeChecker<T> {
  pub fn new(inner: C) -> Self {
    Self { inner, _phantom: PhantomData }
  }

  fn recover_failed_downcast<I>(
    &self,
    some_outputs: Vec<C::Output>,
    current_element: T,
    rest_of_inputs: I,
  ) -> Vec<T>
  where I: Iterator<Item=T> {
    let mut inputs = Vec::with_capacity(some_outputs.len() * 2);
    for out in some_outputs {
      inputs.push(self.inner.widen_type(out));
    }
    inputs.push(current_element);
    inputs.extend(rest_of_inputs);
    inputs
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

impl TypeChecker<Expr> for IsNumber {
  type Output = Number;

  fn narrow_type(&self, input: Expr) -> Result<Number, Expr> {
    Number::try_from(input).map_err(|err| err.original_expr)
  }

  fn widen_type(&self, value: Number) -> Expr {
    Expr::from(value)
  }
}

impl<C, T> TypeChecker<Vec<T>> for OnVec<C, T>
where C: TypeChecker<T> {
  type Output = Vec<C::Output>;

  fn narrow_type(&self, input: Vec<T>) -> Result<Vec<C::Output>, Vec<T>> {
    let mut output = Vec::with_capacity(input.len());
    let mut iter = input.into_iter();
    while let Some(elem) = iter.next() {
      match self.inner.narrow_type(elem) {
        Ok(elem) => output.push(elem),
        Err(elem) => {
          return Err(self.recover_failed_downcast(output, elem, iter));
        }
      }
    }
    Ok(output)
  }

  fn widen_type(&self, value: Vec<C::Output>) -> Vec<T> {
    value.into_iter().map(|i| self.inner.widen_type(i)).collect()
  }
}
