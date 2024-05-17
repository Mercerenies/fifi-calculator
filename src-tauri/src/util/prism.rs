
//! Functional-style prisms for checked downcasts.

use std::marker::PhantomData;

/// A prism from `Up` to `Down` is an assertion of a subtype
/// relationship between `Up` and `Down`. Specifically, it asserts
/// that every `Down` can be seen as an `Up` in a well-defined way,
/// and that some `Up`s can be safely downcast to type `Down`.
pub trait Prism<Up, Down> {
  /// Attempts to downcast `input` to the type `Down`. This method
  /// shall either return the result of successfully downcasting (as
  /// an `Ok`) or the original input value (as an `Err`).
  fn narrow_type(&self, input: Up) -> Result<Down, Up>;

  /// Widens a `Down` value to its parent type. This must always
  /// succeed.
  fn widen_type(&self, input: Down) -> Up;
}

/// The identity prism, which always succeeds.
#[derive(Debug, Clone)]
pub struct Identity<T> {
  _phantom: PhantomData<T>,
}

/// Lift a type-checker into each element of a `Vec`.
#[derive(Debug, Clone, Default)]
pub struct OnVec<C, Up, Down> {
  inner: C,
  _phantom: PhantomData<(Up, Down)>,
}

impl<T> Identity<T> {
  pub fn new() -> Self {
    Self::default()
  }
}

impl<C, Up, Down> OnVec<C, Up, Down>
where C: Prism<Up, Down> {
  pub fn new(inner: C) -> Self {
    Self { inner, _phantom: PhantomData }
  }

  fn recover_failed_downcast<I>(
    &self,
    some_outputs: Vec<Down>,
    current_element: Up,
    rest_of_inputs: I,
  ) -> Vec<Up>
  where I: Iterator<Item=Up> {
    let mut inputs = Vec::with_capacity(some_outputs.len() * 2);
    for out in some_outputs {
      inputs.push(self.inner.widen_type(out));
    }
    inputs.push(current_element);
    inputs.extend(rest_of_inputs);
    inputs
  }
}

impl<T> Default for Identity<T> {
  fn default() -> Self {
    Identity { _phantom: PhantomData }
  }
}

impl<T> Prism<T, T> for Identity<T> {

  fn narrow_type(&self, input: T) -> Result<T, T> {
    Ok(input)
  }

  fn widen_type(&self, input: T) -> T {
    input
  }
}

impl<C, Up, Down> Prism<Vec<Up>, Vec<Down>> for OnVec<C, Up, Down>
where C: Prism<Up, Down> {
  fn narrow_type(&self, input: Vec<Up>) -> Result<Vec<Down>, Vec<Up>> {
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

  fn widen_type(&self, input: Vec<Down>) -> Vec<Up> {
    input.into_iter().map(|i| self.inner.widen_type(i)).collect()
  }
}
