
//! Functional-style prisms for checked downcasts.

use std::marker::PhantomData;

/// A prism from `Up` to `Down` is an assertion of a subtype
/// relationship between `Up` and `Down`. Specifically, it asserts
/// that every `Down` can be seen as an `Up` in a well-defined way,
/// and that some `Up`s can be safely downcast to type `Down`.
///
/// This prism implementation is based loosely on [the Haskell lens
/// library](https://hackage.haskell.org/package/lens-5.3.2/docs/Control-Lens-Prism.html),
/// and prisms implementing this trait should satisfy similar laws.
///
/// * A widen followed by a narrow should reproduce the original
/// value. That is, for all `d: Down`,
/// `prism.narrow_type(prism.widen_type(d)) === Some(d)`.
///
/// * A successful narrow, followed by a widen, should reproduce the
/// original value completely. That is, for all `u: Up`, if
/// `prism.narrow_type(u) = Ok(d)`, then `prism.widen_type(d) === u`.
///
/// * A failed narrow shall return the original value. That is, for
/// all `u: Up`, if `prism.narrow_type(u) = Err(u1)`, then `u === u1`.
///
/// where `===` is taken to mean "conceptually equal". You can think
/// of this as `==` for types that implement `PartialEq`.
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
#[derive(Debug, Clone, Default)]
pub struct Identity {
  _private: (),
}

/// Composition of two prisms.
#[derive(Debug, Clone)]
pub struct Composed<X, Y, B> {
  left: X,
  right: Y,
  _phantom: PhantomData<fn() -> B>,
}

/// Equality prism. This prism accepts only values which are literally
/// equal (under [`PartialEq`]) to the given value and rejects all
/// others. Narrows to `()`.
#[derive(Debug, Clone, Default)]
pub struct Only<T> {
  value: T,
}

/// Lift a type-checker into each element of a `Vec`.
#[derive(Debug, Clone)]
pub struct OnVec<X> {
  inner: X,
}

/// Prism viewing the value on the inside of an `Option`.
#[derive(Debug, Clone, Default)]
pub struct InOption {
  _private: (),
}

impl Identity {
  pub fn new() -> Self {
    Self::default()
  }
}

impl<X, Y, B> Composed<X, Y, B> {
  pub fn new(left: X, right: Y) -> Self {
    Self { left, right, _phantom: PhantomData }
  }
}

impl<T> Only<T> {
  pub fn new(value: T) -> Self {
    Self { value }
  }

  pub fn value(&self) -> &T {
    &self.value
  }

  pub fn into_value(self) -> T {
    self.value
  }
}

impl<X> OnVec<X> {
  pub fn new(inner: X) -> Self {
    Self { inner }
  }
}

impl InOption {
  pub fn new() -> Self {
    Self::default()
  }
}

impl<T> Prism<T, T> for Identity {
  fn narrow_type(&self, input: T) -> Result<T, T> {
    Ok(input)
  }

  fn widen_type(&self, input: T) -> T {
    input
  }
}

impl<X, Y, A, B, C> Prism<A, C> for Composed<X, Y, B>
where X: Prism<A, B>,
      Y: Prism<B, C> {
  fn narrow_type(&self, input: A) -> Result<C, A> {
    let b = self.left.narrow_type(input)?;
    match self.right.narrow_type(b) {
      Ok(c) => Ok(c),
      Err(b) => Err(self.left.widen_type(b)),
    }
  }

  fn widen_type(&self, input: C) -> A {
    let b = self.right.widen_type(input);
    self.left.widen_type(b)
  }
}

impl<T: PartialEq + Clone> Prism<T, ()> for Only<T> {
  fn widen_type(&self, _: ()) -> T {
    self.value.clone()
  }
  fn narrow_type(&self, input: T) -> Result<(), T> {
    if input == self.value {
      Ok(())
    } else {
      Err(input)
    }
  }
}

impl<X, Up, Down> Prism<Vec<Up>, Vec<Down>> for OnVec<X>
where X: Prism<Up, Down> {
  fn narrow_type(&self, input: Vec<Up>) -> Result<Vec<Down>, Vec<Up>> {
    let mut output = Vec::with_capacity(input.len());
    let mut iter = input.into_iter();
    while let Some(elem) = iter.next() {
      match self.inner.narrow_type(elem) {
        Ok(elem) => output.push(elem),
        Err(elem) => {
          return Err(recover_failed_downcast(self, output, elem, iter));
        }
      }
    }
    Ok(output)
  }

  fn widen_type(&self, input: Vec<Down>) -> Vec<Up> {
    input.into_iter().map(|i| self.inner.widen_type(i)).collect()
  }
}

impl<T> Prism<Option<T>, T> for InOption {
  fn narrow_type(&self, input: Option<T>) -> Result<T, Option<T>> {
    input.ok_or(None)
  }

  fn widen_type(&self, input: T) -> Option<T> {
    Some(input)
  }
}

fn recover_failed_downcast<X, Up, Down, I>(
  vec_prism: &OnVec<X>,
  some_outputs: Vec<Down>,
  current_element: Up,
  rest_of_inputs: I,
) -> Vec<Up>
where X: Prism<Up, Down>,
      I: Iterator<Item=Up> {
  let mut inputs = Vec::with_capacity(some_outputs.len() * 2);
  for out in some_outputs {
    inputs.push(vec_prism.inner.widen_type(out));
  }
  inputs.push(current_element);
  inputs.extend(rest_of_inputs);
  inputs
}

#[cfg(test)]
mod tests {
  use super::*;

  #[derive(Debug, Clone, PartialEq)]
  enum ExampleA {
    Empty,
    NonEmpty(ExampleB),
  }

  #[derive(Debug, Clone, PartialEq)]
  enum ExampleB {
    Empty,
    NonEmpty(i32),
  }

  #[derive(Debug)]
  struct ExampleAPrism;

  #[derive(Debug)]
  struct ExampleBPrism;

  impl Prism<ExampleA, ExampleB> for ExampleAPrism {
    fn narrow_type(&self, input: ExampleA) -> Result<ExampleB, ExampleA> {
      match input {
        ExampleA::Empty => Err(ExampleA::Empty),
        ExampleA::NonEmpty(b) => Ok(b),
      }
    }

    fn widen_type(&self, input: ExampleB) -> ExampleA {
      ExampleA::NonEmpty(input)
    }
  }

  impl Prism<ExampleB, i32> for ExampleBPrism {
    fn narrow_type(&self, input: ExampleB) -> Result<i32, ExampleB> {
      match input {
        ExampleB::Empty => Err(ExampleB::Empty),
        ExampleB::NonEmpty(i) => Ok(i),
      }
    }

    fn widen_type(&self, input: i32) -> ExampleB {
      ExampleB::NonEmpty(input)
    }
  }

  #[test]
  fn test_identity_prism() {
    let identity = Identity::new();
    assert_eq!(identity.narrow_type(100), Ok(100));
    assert_eq!(identity.widen_type(100), 100);
  }

  #[test]
  fn test_composed_prism_widening() {
    let composed = Composed::<_, _, ExampleB>::new(ExampleAPrism, ExampleBPrism);
    assert_eq!(composed.widen_type(9), ExampleA::NonEmpty(ExampleB::NonEmpty(9)));
  }

  #[test]
  fn test_composed_prism_narrowing_failure() {
    let composed = Composed::<_, _, ExampleB>::new(ExampleAPrism, ExampleBPrism);
    assert_eq!(composed.narrow_type(ExampleA::Empty), Err(ExampleA::Empty));
    assert_eq!(
      composed.narrow_type(ExampleA::NonEmpty(ExampleB::Empty)),
      Err(ExampleA::NonEmpty(ExampleB::Empty)),
    );
  }

  #[test]
  fn test_composed_prism_narrowing_success() {
    let composed = Composed::<_, _, ExampleB>::new(ExampleAPrism, ExampleBPrism);
    assert_eq!(
      composed.narrow_type(ExampleA::NonEmpty(ExampleB::NonEmpty(99))),
      Ok(99),
    );
  }

  #[test]
  fn test_option_prism() {
    let opt_prism = InOption::new();
    assert_eq!(opt_prism.widen_type(100), Some(100));
    assert_eq!(opt_prism.narrow_type(Some(100)), Ok(100));
    assert_eq!(opt_prism.narrow_type(None::<i32>), Err(None));
  }

  #[test]
  fn test_lifted_vec_prism_on_empty() {
    let vec_prism = OnVec::new(ExampleBPrism);
    assert_eq!(vec_prism.widen_type(vec![]), vec![]);
    assert_eq!(vec_prism.narrow_type(vec![]), Ok(vec![]));
  }

  #[test]
  fn test_lifted_vec_prism_widening() {
    let vec_prism = OnVec::new(ExampleBPrism);
    assert_eq!(
      vec_prism.widen_type(vec![0, 10, 20, 30]),
      vec![
        ExampleB::NonEmpty(0),
        ExampleB::NonEmpty(10),
        ExampleB::NonEmpty(20),
        ExampleB::NonEmpty(30),
      ],
    );
  }

  #[test]
  fn test_lifted_vec_prism_successful_narrowing() {
    let vec_prism = OnVec::new(ExampleBPrism);
    let input = vec![
      ExampleB::NonEmpty(0),
      ExampleB::NonEmpty(10),
      ExampleB::NonEmpty(20),
      ExampleB::NonEmpty(30),
    ];
    assert_eq!(
      vec_prism.narrow_type(input),
      Ok(vec![0, 10, 20, 30]),
    );
  }

  #[test]
  fn test_lifted_vec_prism_no_matches() {
    let vec_prism = OnVec::new(ExampleBPrism);
    let input = vec![
      ExampleB::Empty,
      ExampleB::Empty,
      ExampleB::Empty,
    ];
    assert_eq!(
      vec_prism.narrow_type(input.clone()),
      Err(input),
    );
  }

  #[test]
  fn test_lifted_vec_prism_partial_matches() {
    let vec_prism = OnVec::new(ExampleBPrism);
    let input = vec![
      ExampleB::Empty,
      ExampleB::NonEmpty(100),
      ExampleB::Empty,
      ExampleB::NonEmpty(99),
    ];
    assert_eq!(
      vec_prism.narrow_type(input.clone()),
      Err(input),
    );
  }

  #[test]
  fn test_equality_prism() {
    let prism = Only::new(10);
    assert_eq!(prism.narrow_type(10), Ok(()));
    assert_eq!(prism.narrow_type(9), Err(9));
    assert_eq!(prism.widen_type(()), 10);
  }
}
