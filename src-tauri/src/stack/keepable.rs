
use super::base::{StackLike, RandomAccessStackLike};
use super::error::StackError;

use std::marker::PhantomData;

/// A `KeepableStack` is a stack implementation that optionally
/// implements "keep" semantics. These semantics are enabled with a
/// Boolean flag.
///
/// With "keep" semantics off, the `KeepableStack` merely delegates to
/// its underlying stack with no modifications. With "keep" semantics
/// on, the `KeepableStack` will perform "push" operations normally
/// but, when a "pop" operation is requested, the desired elements
/// will be returned but will also be pushed back onto the stack.
///
/// To state the obvious, `T: Clone` must be true in order for the
/// keep semantics to work, as it involves cloning the underlying
/// value in order to both keep and return it.
///
/// Note that some care must be taken when using a [`KeepableStack`].
/// Notably, since the `pop`, `pop_several`, and `pop_all` methods
/// don't actually remove values from the stack, consecutive pops are
/// NOT equivalent to `pop_several`. That is, for a regular
/// (non-keepable) stack, popping two elements in sequence pops two
/// distinct elements, equivalent to `pop_several` with an argument of
/// two.
///
/// ```
/// # use fifi::stack::Stack;
/// # use fifi::stack::base::StackLike;
/// # let mut stack: Stack<i64> = Stack::new();
/// /// `stack` is a regular `Stack<i64>`.
/// stack.push_several(vec![10, 20, 30, 40]);
/// assert_eq!(stack.pop(), Ok(40));
/// assert_eq!(stack.pop(), Ok(30));
/// assert_eq!(stack.len(), 2);
/// ```
///
/// However, with a keepable stack, subsequent pops will pop the same
/// element repeatedly.
///
/// ```
/// # use fifi::stack::Stack;
/// # use fifi::stack::base::StackLike;
/// # use fifi::stack::keepable::KeepableStack;
/// # let mut stack: Stack<i64> = Stack::new();
/// # let mut stack = KeepableStack::new(&mut stack, true);
/// /// `stack` is a regular `KeepableStack<'_, _, i64`.
/// stack.push_several(vec![10, 20, 30, 40]);
/// assert_eq!(stack.pop(), Ok(40));
/// assert_eq!(stack.pop(), Ok(40));
/// assert_eq!(stack.len(), 4);
/// ```
///
/// In this case, `pop_several` with an argument of two will actually
/// pop two distinct values.
///
/// ```
/// # use fifi::stack::Stack;
/// # use fifi::stack::base::StackLike;
/// # use fifi::stack::keepable::KeepableStack;
/// # let mut stack: Stack<i64> = Stack::new();
/// # let mut stack = KeepableStack::new(&mut stack, true);
/// /// `stack` is a regular `KeepableStack<'_, _, i64`.
/// stack.push_several(vec![10, 20, 30, 40]);
/// assert_eq!(stack.pop_several(2), Ok(vec![30, 40]));
/// assert_eq!(stack.len(), 4);
/// ```
#[derive(Debug)]
pub struct KeepableStack<'a, S, T> {
  stack: &'a mut S,
  keep_semantics: bool,
  _marker: PhantomData<T>,
}

impl<'a, S, T> KeepableStack<'a, S, T>
where S: StackLike<T> {
  pub fn new(stack: &'a mut S, keep_semantics: bool) -> Self {
    Self {
      stack,
      keep_semantics,
      _marker: PhantomData,
    }
  }

  pub fn get_inner_mut(&mut self) -> &mut S {
    self.stack
  }

  pub fn keep_semantics(&self) -> bool {
    self.keep_semantics
  }
}

impl<'a, S, T> StackLike<T> for KeepableStack<'a, S, T>
where S: StackLike<T>,
      T: Clone {
  fn len(&self) -> usize {
    self.stack.len()
  }

  fn push(&mut self, element: T) {
    self.stack.push(element);
  }

  fn push_several(&mut self, elements: impl IntoIterator<Item = T>) {
    self.stack.push_several(elements);
  }

  fn pop(&mut self) -> Result<T, StackError> {
    let value = self.stack.pop()?;
    if self.keep_semantics {
      self.stack.push(value.clone());
    }
    Ok(value)
  }

  fn pop_several(&mut self, count: usize) -> Result<Vec<T>, StackError> {
    let values = self.stack.pop_several(count)?;
    if self.keep_semantics {
      self.stack.push_several(values.clone());
    }
    Ok(values)
  }
}

/// Random access to a [`KeepableStack`] delegates to the underlying
/// stack and never utilizes any "keep" semantics, regardless of the
/// value of [`KeepableStack::keep_semantics`].
impl<'a, S, T> RandomAccessStackLike<T> for KeepableStack<'a, S, T>
where S: RandomAccessStackLike<T>,
      T: Clone {
  type Ref<'b> = S::Ref<'b> where Self: 'b;
  type Mut<'b> = S::Mut<'b> where Self: 'b;

  fn get(&self, index: i64) -> Result<S::Ref<'_>, StackError> {
    self.stack.get(index)
  }

  fn get_mut(&mut self, index: i64) -> Result<S::Mut<'_>, StackError> {
    self.stack.get_mut(index)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::stack::Stack;

  #[test]
  fn test_push_pop_with_no_keep_semantics() {
    let mut stack = Stack::from(vec![0, 10]);
    let mut stack = KeepableStack::new(&mut stack, false);
    stack.push(20);
    assert_eq!(stack.pop(), Ok(20));
    assert_eq!(stack.pop(), Ok(10));
    assert_eq!(stack.pop(), Ok(0));
    assert_eq!(stack.pop(), Err(StackError::NotEnoughElements { expected: 1, actual: 0 }));
  }

  #[test]
  fn test_push_pop_with_active_keep_semantics() {
    let mut stack = Stack::from(vec![0, 10]);
    {
      let mut stack = KeepableStack::new(&mut stack, true);
      stack.push(20);
      stack.push(30);
      assert_eq!(stack.len(), 4);
      assert_eq!(stack.pop(), Ok(30));
      assert_eq!(stack.pop(), Ok(30));
      assert_eq!(stack.pop(), Ok(30));
      assert_eq!(stack.pop(), Ok(30));
      assert_eq!(stack.pop(), Ok(30));
      assert_eq!(stack.len(), 4);
      stack.push(40);
      assert_eq!(stack.pop(), Ok(40));
      assert_eq!(stack.pop(), Ok(40));
      assert_eq!(stack.len(), 5);
    }
    assert_eq!(stack.into_iter().collect::<Vec<_>>(), vec![0, 10, 20, 30, 40]);
  }

  #[test]
  fn test_push_several_with_no_keep_semantics() {
    let mut stack = Stack::from(vec![0, 10, 20, 30, 40]);
    {
      let mut stack = KeepableStack::new(&mut stack, false);
      stack.push_several(vec![50, 60, 70]);
    }
    let elements = stack.into_iter().collect::<Vec<_>>();
    assert_eq!(elements, vec![0, 10, 20, 30, 40, 50, 60, 70]);
  }

  #[test]
  fn test_push_several_with_keep_semantics() {
    let mut stack = Stack::from(vec![0, 10, 20, 30, 40]);
    {
      let mut stack = KeepableStack::new(&mut stack, true);
      stack.push_several(vec![50, 60, 70]);
    }
    let elements = stack.into_iter().collect::<Vec<_>>();
    assert_eq!(elements, vec![0, 10, 20, 30, 40, 50, 60, 70]);
  }

  #[test]
  fn test_pop_several_with_no_keep_semantics() {
    let mut stack = Stack::from(vec![0, 10, 20, 30, 40]);
    let mut stack = KeepableStack::new(&mut stack, false);
    assert_eq!(stack.pop_several(3), Ok(vec![20, 30, 40]));
    assert_eq!(stack.len(), 2);
    assert_eq!(stack.pop_several(3), Err(StackError::NotEnoughElements { expected: 3, actual: 2 }));
    assert_eq!(stack.len(), 2);
    assert_eq!(stack.pop_several(2), Ok(vec![0, 10]));
    assert_eq!(stack.len(), 0);
    assert!(stack.is_empty());
  }

  #[test]
  fn test_pop_several_with_active_keep_semantics() {
    let mut stack = Stack::from(vec![0, 10, 20, 30, 40]);
    let mut stack = KeepableStack::new(&mut stack, true);
    assert_eq!(stack.pop_several(1), Ok(vec![40]));
    assert_eq!(stack.len(), 5);
    assert_eq!(stack.pop_several(2), Ok(vec![30, 40]));
    assert_eq!(stack.len(), 5);
    assert_eq!(stack.pop_several(3), Ok(vec![20, 30, 40]));
    assert_eq!(stack.len(), 5);
    assert_eq!(stack.pop_several(4), Ok(vec![10, 20, 30, 40]));
    assert_eq!(stack.len(), 5);
    assert_eq!(stack.pop_several(5), Ok(vec![0, 10, 20, 30, 40]));
    assert_eq!(stack.len(), 5);
    assert_eq!(stack.pop_several(6), Err(StackError::NotEnoughElements { expected: 6, actual: 5 }));
    assert_eq!(stack.len(), 5);
    assert_eq!(stack.pop_all(), vec![0, 10, 20, 30, 40]);
    assert_eq!(stack.len(), 5);
  }

  #[test]
  fn test_len() {
    let mut stack = Stack::new();
    {
      let mut stack = KeepableStack::new(&mut stack, false);
      assert_eq!(stack.len(), 0);
      stack.push(0);
      assert_eq!(stack.len(), 1);
      stack.push(0);
      stack.push(0);
      assert_eq!(stack.len(), 3);
      let _ = stack.pop();
      assert_eq!(stack.len(), 2);
    }
    {
      let mut stack = KeepableStack::new(&mut stack, true);
      assert_eq!(stack.len(), 2);
      stack.push(0);
      assert_eq!(stack.len(), 3);
      stack.push(0);
      stack.push(0);
      assert_eq!(stack.len(), 5);
      let _ = stack.pop(); // Doesn't actually pop, since the stack is kept!
      assert_eq!(stack.len(), 5);
    }
  }

  #[test]
  fn test_is_empty_without_keep_semantics() {
    let mut stack = Stack::new();
    let mut stack = KeepableStack::new(&mut stack, false);
    assert!(stack.is_empty());
    stack.push(0);
    assert!(!stack.is_empty());
    stack.push(10);
    assert!(!stack.is_empty());
    assert_eq!(stack.pop().unwrap(), 10);
    assert!(!stack.is_empty());
    assert_eq!(stack.pop().unwrap(), 0);
    assert!(stack.is_empty());
  }

  #[test]
  fn test_is_empty_with_keep_semantics() {
    let mut stack = Stack::new();
    let mut stack = KeepableStack::new(&mut stack, true);
    assert!(stack.is_empty());
    stack.push(0);
    assert_eq!(stack.pop().unwrap(), 0);
    assert!(!stack.is_empty());
    // We can keep popping, because the stack is kept!
    assert_eq!(stack.pop().unwrap(), 0);
    assert!(!stack.is_empty());
    assert_eq!(stack.pop().unwrap(), 0);
    assert!(!stack.is_empty());
  }
}
