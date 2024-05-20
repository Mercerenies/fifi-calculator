
//! A delegating stack is a [`Stack`] that notifies a
//! [`StackDelegate`] implementation of any changes to the stack.

use super::structure::Stack;
use super::error::StackError;

use std::hash::{Hash, Hasher};

pub trait StackDelegate<T> {
  /// Called before a new value is pushed onto the stack.
  ///
  /// If several values are pushed at once, this method may be called
  /// before each one or several before any are pushed, but it will be
  /// called in the right order.
  fn on_push(&mut self, new_value: &T);

  /// Called after a value is popped off the stack.
  ///
  /// If several values are popped at once, this method may be called
  /// after each one individually, or after all of them have been
  /// popped.
  fn on_pop(&mut self, old_value: &T);

  /// Called after a value on the stack is modified in-place.
  fn on_mutate(&mut self, old_value: T, new_value: &T);
}

/// Null Object implementation of [`StackDelegate`]. Never performs
/// any action in reaction to stack delegate events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NullStackDelegate;

#[derive(Debug)]
pub struct DelegatingStack<'a, T, D> {
  stack: &'a mut Stack<T>,
  delegate: D,
}

impl<'a, T, D> DelegatingStack<'a, T, D>
where D: StackDelegate<T> {
  pub fn new(stack: &'a mut Stack<T>, delegate: D) -> DelegatingStack<'a, T, D> {
    DelegatingStack { stack, delegate }
  }

  pub fn check_stack_size(&self, expected: usize) -> Result<(), StackError> {
    self.stack.check_stack_size(expected)
  }

  pub fn push(&mut self, element: T) {
    self.delegate.on_push(&element);
    self.stack.push(element);
  }

  pub fn push_several(&mut self, elements: impl IntoIterator<Item = T>) {
    for element in elements {
      self.push(element);
    }
  }

  pub fn pop(&mut self) -> Result<T, StackError> {
    let value = self.stack.pop()?;
    self.delegate.on_pop(&value);
    Ok(value)
  }

  pub fn pop_and_discard(&mut self) {
    let _ = self.pop();
  }

  pub fn pop_several(&mut self, count: usize) -> Result<Vec<T>, StackError> {
    let values = self.stack.pop_several(count)?;
    for value in values.iter().rev() {
      self.delegate.on_pop(value);
    }
    Ok(values)
  }

  pub fn pop_all(&mut self) -> Vec<T> {
    let values = self.stack.pop_all();
    for value in values.iter().rev() {
      self.delegate.on_pop(value);
    }
    values
  }

  pub fn len(&self) -> usize {
    self.stack.len()
  }

  pub fn is_empty(&self) -> bool {
    self.stack.is_empty()
  }

  /// Gets a value from the stack. Nonnegative indices index from the
  /// top of the stack (with index 0 being the top), while negative
  /// indices index from the bottom (with index -1 being the very
  /// bottom).
  pub fn get(&self, index: i64) -> Result<&T, StackError> {
    self.stack.get(index)
  }

  /// Iterates from the bottom of the stack.
  pub fn iter(&self) -> impl DoubleEndedIterator<Item = &T> {
    self.stack.iter()
  }
}

/// Comparisons on a `DelegatingStack` simply compare the stacks and
/// ignore the delegates.
impl<'a, T: PartialEq, D> PartialEq for DelegatingStack<'a, T, D> {
  fn eq(&self, other: &Self) -> bool {
    self.stack == other.stack
  }
}

impl<'a, T: Eq, D> Eq for DelegatingStack<'a, T, D> {}

impl<'a, T: Hash, D> Hash for DelegatingStack<'a, T, D> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.stack.hash(state);
  }
}

impl<T> StackDelegate<T> for NullStackDelegate {
  fn on_push(&mut self, _: &T) {}
  fn on_pop(&mut self, _: &T) {}
  fn on_mutate(&mut self, _: T, _: &T) {}
}

#[cfg(test)]
mod tests {
  use super::*;

  #[derive(Default, Debug, PartialEq, Eq)]
  struct TestDelegate {
    pushes: Vec<i32>,
    pops: Vec<i32>,
    mutations: Vec<(i32, i32)>,
  }

  impl StackDelegate<i32> for TestDelegate {
    fn on_push(&mut self, value: &i32) {
      self.pushes.push(*value);
    }

    fn on_pop(&mut self, value: &i32) {
      self.pops.push(*value);
    }

    fn on_mutate(&mut self, old_value: i32, new_value: &i32) {
      self.mutations.push((old_value, *new_value));
    }
  }

  #[test]
  fn test_push() {
    let mut stack = Stack::new();
    let mut stack = DelegatingStack::new(&mut stack, TestDelegate::default());
    stack.push(1);
    stack.push(2);
    stack.push(3);
    assert_eq!(stack.len(), 3);
    assert_eq!(stack.delegate, TestDelegate {
      pushes: vec![1, 2, 3],
      pops: vec![],
      mutations: vec![],
    });
  }

  #[test]
  fn test_push_several() {
    let mut stack = Stack::new();
    let mut stack = DelegatingStack::new(&mut stack, TestDelegate::default());
    stack.push_several([1, 2, 3]);
    assert_eq!(stack.len(), 3);
    assert_eq!(stack.delegate, TestDelegate {
      pushes: vec![1, 2, 3],
      pops: vec![],
      mutations: vec![],
    });
  }

  #[test]
  fn test_pop() {
    let mut stack = Stack::from(vec![10, 20, 30]);
    let mut stack = DelegatingStack::new(&mut stack, TestDelegate::default());
    assert_eq!(stack.pop(), Ok(30));
    assert_eq!(stack.pop(), Ok(20));
    assert_eq!(stack.len(), 1);
    assert_eq!(stack.delegate, TestDelegate {
      pushes: vec![],
      pops: vec![30, 20],
      mutations: vec![],
    });
  }

  #[test]
  fn test_pop_and_discard() {
    let mut stack = Stack::from(vec![10, 20, 30]);
    let mut stack = DelegatingStack::new(&mut stack, TestDelegate::default());
    for _ in 0..10 {
      stack.pop_and_discard();
    }
    assert!(stack.is_empty());
    assert_eq!(stack.delegate, TestDelegate {
      pushes: vec![],
      pops: vec![30, 20, 10],
      mutations: vec![],
    });
  }

  #[test]
  fn test_pop_several() {
    let mut stack = Stack::from(vec![10, 20, 30, 40, 50]);
    let mut stack = DelegatingStack::new(&mut stack, TestDelegate::default());
    assert_eq!(stack.pop_several(3), Ok(vec![30, 40, 50]));
    assert_eq!(stack.len(), 2);
    assert_eq!(stack.delegate, TestDelegate {
      pushes: vec![],
      pops: vec![50, 40, 30],
      mutations: vec![],
    });

    // A failing pop_several should not pop anything or call the
    // delegate.
    assert_eq!(stack.pop_several(3), Err(StackError::NotEnoughElements { expected: 3, actual: 2 }));
    assert_eq!(stack.len(), 2);
    assert_eq!(stack.delegate, TestDelegate {
      pushes: vec![],
      pops: vec![50, 40, 30],
      mutations: vec![],
    });
  }

  #[test]
  fn test_pop_all() {
    let mut stack = Stack::from(vec![10, 20, 30, 40, 50]);
    let mut stack = DelegatingStack::new(&mut stack, TestDelegate::default());
    assert_eq!(stack.pop_all(), vec![10, 20, 30, 40, 50]);
    assert!(stack.is_empty());
    assert_eq!(stack.delegate, TestDelegate {
      pushes: vec![],
      pops: vec![50, 40, 30, 20, 10],
      mutations: vec![],
    });
  }
}
