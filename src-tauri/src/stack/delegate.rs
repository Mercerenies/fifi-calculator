
//! A delegating stack is a [`Stack`] that notifies a
//! [`StackDelegate`] implementation of any changes to the stack.

use super::structure::Stack;
use super::error::StackError;

use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};

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
  fn on_pop(&mut self, index: usize, old_value: &T);

  /// Called after a value on the stack is modified in-place.
  fn on_mutate(&mut self, index: i64, old_value: &T, new_value: &T);
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

pub struct RefMut<'a, T, D: StackDelegate<T>> {
  index: i64,
  delegate: &'a mut D,
  original_value: T,
  value: &'a mut T,
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
    self.delegate.on_pop(0, &value);
    Ok(value)
  }

  pub fn pop_nth(&mut self, index: usize) -> Result<T, StackError> {
    let value = self.stack.pop_nth(index)?;
    self.delegate.on_pop(index, &value);
    Ok(value)
  }

  pub fn pop_and_discard(&mut self) {
    let _ = self.pop();
  }

  pub fn pop_several(&mut self, count: usize) -> Result<Vec<T>, StackError> {
    let values = self.stack.pop_several(count)?;
    for value in values.iter().rev() {
      self.delegate.on_pop(0, value);
    }
    Ok(values)
  }

  pub fn pop_all(&mut self) -> Vec<T> {
    let values = self.stack.pop_all();
    for value in values.iter().rev() {
      self.delegate.on_pop(0, value);
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

  /// Gets a mutable reference from the stack. When this reference is
  /// dropped, the mutation will be logged to the delegate, which
  /// requires `T: Clone` in order to be fully implemented correctly.
  pub fn get_mut(&mut self, index: i64) -> Result<RefMut<'_, T, D>, StackError>
  where T: Clone {
    let value = self.stack.get_mut(index)?;
    Ok(RefMut {
      index,
      delegate: &mut self.delegate,
      original_value: value.clone(),
      value,
    })
  }

  /// Modifies the value at the given position, using the given
  /// function. In order to properly invoke the delegate, `T` must be
  /// `Clone`, so that the delegate can be called with both the old
  /// and new values.
  pub fn mutate<F>(&mut self, index: i64, f: F) -> Result<(), StackError>
  where F: FnOnce(&mut T),
        T: Clone {
    let value = self.stack.get_mut(index)?;
    let old_value = value.clone();
    f(value);
    self.delegate.on_mutate(index, &old_value, value);
    Ok(())
  }

  /// Iterates from the bottom of the stack.
  pub fn iter(&self) -> impl DoubleEndedIterator<Item = &T> {
    self.stack.iter()
  }

  pub fn foreach_mut<F>(&mut self, mut f: F)
  where F: FnMut(&mut T),
        T: Clone {
    let len = self.len();
    for (i, elem) in self.stack.iter_mut().enumerate() {
      let original_elem = elem.clone();
      f(elem);
      self.delegate.on_mutate((len - i - 1) as i64, &original_elem, elem);
    }
  }
}

impl<'a, T, D: StackDelegate<T>> Deref for RefMut<'a, T, D> {
  type Target = T;

  fn deref(&self) -> &T {
    &self.value
  }
}

impl<'a, T, D: StackDelegate<T>> DerefMut for RefMut<'a, T, D> {
  fn deref_mut(&mut self) -> &mut T {
    &mut self.value
  }
}

impl<'a, T, D: StackDelegate<T>> Drop for RefMut<'a, T, D> {
  fn drop(&mut self) {
    self.delegate.on_mutate(self.index, &self.original_value, &self.value);
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
  fn on_pop(&mut self, _: usize, _: &T) {}
  fn on_mutate(&mut self, _: i64, _: &T, _: &T) {}
}

#[cfg(test)]
mod tests {
  use super::*;

  #[derive(Default, Debug, PartialEq, Eq)]
  struct TestDelegate {
    pushes: Vec<i32>,
    pops: Vec<(usize, i32)>,
    mutations: Vec<(i64, i32, i32)>,
  }

  impl StackDelegate<i32> for TestDelegate {
    fn on_push(&mut self, value: &i32) {
      self.pushes.push(*value);
    }

    fn on_pop(&mut self, index: usize, value: &i32) {
      self.pops.push((index, *value));
    }

    fn on_mutate(&mut self, index: i64, old_value: &i32, new_value: &i32) {
      self.mutations.push((index, *old_value, *new_value));
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
      pops: vec![(0, 30), (0, 20)],
      mutations: vec![],
    });
  }

  #[test]
  fn test_pop_nth() {
    let mut stack = Stack::from(vec![10, 20, 30, 40, 50]);
    let mut stack = DelegatingStack::new(&mut stack, TestDelegate::default());
    assert_eq!(stack.pop_nth(1), Ok(40));
    assert_eq!(stack.pop_nth(2), Ok(20));
    assert_eq!(stack.pop_nth(3), Err(StackError::NotEnoughElements { expected: 4, actual: 3 }));
    assert_eq!(stack.len(), 3);
    assert_eq!(stack.stack.iter().collect::<Vec<_>>(), vec![&10, &30, &50]);
    assert_eq!(stack.delegate, TestDelegate {
      pushes: vec![],
      pops: vec![(1, 40), (2, 20)],
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
      pops: vec![(0, 30), (0, 20), (0, 10)],
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
      pops: vec![(0, 50), (0, 40), (0, 30)],
      mutations: vec![],
    });

    // A failing pop_several should not pop anything or call the
    // delegate.
    assert_eq!(stack.pop_several(3), Err(StackError::NotEnoughElements { expected: 3, actual: 2 }));
    assert_eq!(stack.len(), 2);
    assert_eq!(stack.delegate, TestDelegate {
      pushes: vec![],
      pops: vec![(0, 50), (0, 40), (0, 30)],
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
      pops: vec![(0, 50), (0, 40), (0, 30), (0, 20), (0, 10)],
      mutations: vec![],
    });
  }

  #[test]
  fn test_mutate() {
    let mut stack = Stack::from(vec![10, 20, 30, 40, 50]);
    let mut stack = DelegatingStack::new(&mut stack, TestDelegate::default());
    assert_eq!(stack.mutate(1, |value| *value += 1), Ok(()));
    assert_eq!(stack.mutate(-2, |value| *value += 2), Ok(()));
    assert_eq!(
      stack.mutate(-999, |value| *value += 3),
      Err(StackError::NotEnoughElements { expected: 999, actual: 5 }),
    );
    assert_eq!(
      stack.mutate(999, |value| *value += 4),
      Err(StackError::NotEnoughElements { expected: 1000, actual: 5 }),
    );
    assert_eq!(stack.stack.iter().collect::<Vec<_>>(), vec![&10, &22, &30, &41, &50]);
    assert_eq!(stack.delegate, TestDelegate {
      pushes: vec![],
      pops: vec![],
      mutations: vec![(1, 40, 41), (-2, 20, 22)],
    });
  }

  #[test]
  fn test_get_mut() {
    let mut stack = Stack::from(vec![10, 20, 30, 40, 50]);
    let mut stack = DelegatingStack::new(&mut stack, TestDelegate::default());

    {
      let mut v = stack.get_mut(1).unwrap();
      *v += 1;
    }
    {
      let mut v = stack.get_mut(-2).unwrap();
      *v += 2;
    }

    assert_eq!(stack.stack.iter().collect::<Vec<_>>(), vec![&10, &22, &30, &41, &50]);
    assert_eq!(stack.delegate, TestDelegate {
      pushes: vec![],
      pops: vec![],
      mutations: vec![(1, 40, 41), (-2, 20, 22)],
    });
  }

  #[test]
  fn test_foreach_mut() {
    let mut stack = Stack::from(vec![10, 20, 30, 40, 50]);
    let mut stack = DelegatingStack::new(&mut stack, TestDelegate::default());
    stack.foreach_mut(|value| *value += 1);
    assert_eq!(stack.stack.iter().collect::<Vec<_>>(), vec![&11, &21, &31, &41, &51]);
    assert_eq!(stack.delegate, TestDelegate {
      pushes: vec![],
      pops: vec![],
      mutations: vec![(4, 10, 11), (3, 20, 21), (2, 30, 31), (1, 40, 41), (0, 50, 51)],
    });
  }

}
