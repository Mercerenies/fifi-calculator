
use super::error::StackError;
use super::base::{StackLike, RandomAccessStackLike};

use std::slice;

/// FIFO stack. Implemented internally as a vector whose "top" is at
/// the end, allowing for constant-time pushes and pops.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Stack<T> {
  elements: Vec<T>,
}

impl<T> Stack<T> {
  pub fn new() -> Self {
    Self::default()
  }

  /// Pops the nth element (0-indexed and counting from the top) and
  /// returns it. If out of bounds, returns None. This function does
  /// NOT support negative indexing.
  pub fn pop_nth(&mut self, index: usize) -> Result<T, StackError> {
    if index >= self.len() {
      return Err(StackError::NotEnoughElements { expected: index + 1, actual: self.len() })
    }
    Ok(self.elements.remove(self.len() - index - 1))
  }

  /// Inserts an element at the given position, 0-indexed from the top
  /// of the stack. `self.insert(0, x)` is equivalent to
  /// `self.push(x)`.
  pub fn insert(&mut self, index: usize, element: T) -> Result<(), StackError> {
    self.check_stack_size(index)?;
    self.elements.insert(self.len() - index, element);
    Ok(())
  }

  /// Returns either an index into the internal vector (as an `Ok`) or
  /// an appropriate [`StackError`]. If an `Ok` is returned, it is
  /// guaranteed to be in-bounds for the vector.
  fn to_vec_index(&self, index: i64) -> Result<usize, StackError> {
    if index < 0 {
      // Negative index, count from the bottom of the stack.
      let index = (- index - 1) as usize;
      if index >= self.len() {
        Err(StackError::NotEnoughElements { expected: index + 1, actual: self.len() })
      } else {
        Ok(index)
      }
    } else if index as usize >= self.len() {
      // Non-negative index out-of-bounds, report error.
      Err(StackError::NotEnoughElements { expected: index as usize + 1, actual: self.len() })
    } else {
      // Non-negative index in-bounds, ok.
      Ok(self.len() - ((index + 1) as usize))
    }
  }

  /// Iterates from the bottom of the stack.
  pub fn iter(&self) -> slice::Iter<'_, T> {
    self.elements.iter()
  }

  /// Iterates (with mutable references) from the bottom of the stack.
  pub fn iter_mut(&mut self) -> slice::IterMut<'_, T> {
    self.elements.iter_mut()
  }

}

impl<T> StackLike for Stack<T> {
  type Elem = T;

  fn pop_all(&mut self) -> Vec<T> {
    self.elements.drain(..).collect()
  }

  fn len(&self) -> usize {
    self.elements.len()
  }

  fn push(&mut self, element: T) {
    self.elements.push(element);
  }

  /// Push in the order we see them, so that the last element in the
  /// iterable is at the top of the resulting stack.
  fn push_several(&mut self, elements: impl IntoIterator<Item = T>) {
    self.elements.extend(elements);
  }

  fn pop(&mut self) -> Result<T, StackError> {
    self.elements.pop().ok_or(StackError::NotEnoughElements { expected: 1, actual: 0 })
  }

  /// Pops `count` elements off the stack and returns those elements,
  /// with the former top of the stack at the end of the vector. In
  /// case of a [`StackError`], `self` will NOT be modified.
  fn pop_several(&mut self, count: usize) -> Result<Vec<T>, StackError> {
    self.check_stack_size(count)?;
    Ok(self.elements.split_off(self.len() - count))
  }
}

impl<T> RandomAccessStackLike for Stack<T> {
  type Ref<'a> = &'a T where Self: 'a;
  type Mut<'a> = &'a mut T where Self: 'a;

  fn get(&self, index: i64) -> Result<&T, StackError> {
    let index = self.to_vec_index(index)?;
    Ok(&self.elements[index])
  }

  fn get_mut(&mut self, index: i64) -> Result<&mut T, StackError> {
    let index = self.to_vec_index(index)?;
    Ok(&mut self.elements[index])
  }
}

impl<T> IntoIterator for Stack<T> {
  type Item = T;
  type IntoIter = std::vec::IntoIter<Self::Item>;

  /// Iterates (by value) from the bottom of the stack.
  fn into_iter(self) -> Self::IntoIter {
    self.elements.into_iter()
  }
}

/// Converts a vector to a stack, where the top of the stack is at the
/// end.
impl<T> From<Vec<T>> for Stack<T> {
  fn from(elements: Vec<T>) -> Self {
    Self { elements }
  }
}

impl<T> Default for Stack<T> {
  fn default() -> Self {
    Self {
      elements: Vec::with_capacity(10),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_new_empty() {
    let empty_stack = Stack::<i32>::new();
    assert_eq!(empty_stack.len(), 0);
    let empty_stack = Stack::<i32>::default();
    assert_eq!(empty_stack.len(), 0);
  }

  #[test]
  fn test_from_vec() {
    let stack1 = Stack::from(vec![0, 10, 20, 25]);
    let stack2 = {
      let mut stack2 = Stack::new();
      stack2.push(0);
      stack2.push(10);
      stack2.push(20);
      stack2.push(25);
      stack2
    };
    assert_eq!(stack2, stack1);
  }

  #[test]
  fn test_push_pop() {
    let mut stack = Stack::from(vec![0, 10]);
    stack.push(20);
    assert_eq!(stack.pop(), Ok(20));
    assert_eq!(stack.pop(), Ok(10));
    assert_eq!(stack.pop(), Ok(0));
    assert_eq!(stack.pop(), Err(StackError::NotEnoughElements { expected: 1, actual: 0 }));
  }

  #[test]
  fn test_pop_several() {
    let mut stack = Stack::from(vec![0, 10, 20, 30, 40]);
    assert_eq!(stack.pop_several(3), Ok(vec![20, 30, 40]));
    assert_eq!(stack.len(), 2);
    assert_eq!(stack.pop_several(3), Err(StackError::NotEnoughElements { expected: 3, actual: 2 }));
    assert_eq!(stack.len(), 2);
    assert_eq!(stack.pop_several(2), Ok(vec![0, 10]));
    assert_eq!(stack.len(), 0);
    assert!(stack.is_empty());
  }

  #[test]
  fn test_pop_nth() {
    let mut stack = Stack::from(vec![0, 10, 20, 30, 40]);
    assert_eq!(stack.pop_nth(0), Ok(40));
    assert_eq!(stack.clone().into_iter().collect::<Vec<_>>(), vec![0, 10, 20, 30]);
    assert_eq!(stack.pop_nth(1), Ok(20));
    assert_eq!(stack.clone().into_iter().collect::<Vec<_>>(), vec![0, 10, 30]);
    assert_eq!(stack.pop_nth(3), Err(StackError::NotEnoughElements { expected: 4, actual: 3 }));
    assert_eq!(stack.pop_nth(9), Err(StackError::NotEnoughElements { expected: 10, actual: 3 }));
    assert_eq!(stack.pop_nth(99), Err(StackError::NotEnoughElements { expected: 100, actual: 3 }));
    assert_eq!(stack.into_iter().collect::<Vec<_>>(), vec![0, 10, 30]);
  }

  #[test]
  fn test_len() {
    let mut stack = Stack::new();
    assert_eq!(stack.len(), 0);
    stack.push(0);
    assert_eq!(stack.len(), 1);
    stack.push(0);
    stack.push(0);
    assert_eq!(stack.len(), 3);
    let _ = stack.pop();
    assert_eq!(stack.len(), 2);
  }

  #[test]
  fn test_is_empty() {
    let mut stack = Stack::new();
    assert!(stack.is_empty());
    stack.push(0);
    assert!(!stack.is_empty());
    stack.push(0);
    assert!(!stack.is_empty());
    let _ = stack.pop();
    assert!(!stack.is_empty());
    let _ = stack.pop();
    assert!(stack.is_empty());
  }

  #[test]
  fn test_get() {
    let stack = Stack::from(vec!['A', 'B', 'C', 'D']);
    // Regular indexing
    assert_eq!(stack.get(0), Ok(&'D'));
    assert_eq!(stack.get(1), Ok(&'C'));
    assert_eq!(stack.get(2), Ok(&'B'));
    assert_eq!(stack.get(3), Ok(&'A'));
    assert_eq!(stack.get(4), Err(StackError::NotEnoughElements { expected: 5, actual: 4 }));
    // Negative indexing
    assert_eq!(stack.get(-1), Ok(&'A'));
    assert_eq!(stack.get(-2), Ok(&'B'));
    assert_eq!(stack.get(-3), Ok(&'C'));
    assert_eq!(stack.get(-4), Ok(&'D'));
    assert_eq!(stack.get(-5), Err(StackError::NotEnoughElements { expected: 5, actual: 4 }));
  }

  #[test]
  fn test_get_mut() {
    let mut stack = Stack::from(vec!['A', 'B', 'C', 'D']);
    // Regular indexing
    assert_eq!(stack.get_mut(0), Ok(&mut 'D'));
    assert_eq!(stack.get_mut(1), Ok(&mut 'C'));
    assert_eq!(stack.get_mut(2), Ok(&mut 'B'));
    assert_eq!(stack.get_mut(3), Ok(&mut 'A'));
    assert_eq!(stack.get_mut(4), Err(StackError::NotEnoughElements { expected: 5, actual: 4 }));
    // Negative indexing
    assert_eq!(stack.get_mut(-1), Ok(&mut 'A'));
    assert_eq!(stack.get_mut(-2), Ok(&mut 'B'));
    assert_eq!(stack.get_mut(-3), Ok(&mut 'C'));
    assert_eq!(stack.get_mut(-4), Ok(&mut 'D'));
    assert_eq!(stack.get_mut(-5), Err(StackError::NotEnoughElements { expected: 5, actual: 4 }));
  }

  #[test]
  fn test_get_mut_modifications() {
    let mut stack = Stack::new();
    stack.push(1);
    stack.push(2);
    stack.push(3);
    stack.push(4);
    stack.push(5);
    let ptr = stack.get_mut(1).unwrap();
    *ptr = 999;
    assert_eq!(stack.pop(), Ok(5));
    assert_eq!(stack.pop(), Ok(999));
    assert_eq!(stack.pop(), Ok(3));
    assert_eq!(stack.pop(), Ok(2));
    assert_eq!(stack.pop(), Ok(1));
    assert_eq!(stack.pop(), Err(StackError::NotEnoughElements { expected: 1, actual: 0 }));
  }

  #[test]
  fn test_into_iter() {
    let stack = Stack::from(vec!['A', 'B', 'C', 'D']);
    let vec = stack.into_iter().collect::<Vec<_>>();
    assert_eq!(vec, vec!['A', 'B', 'C', 'D']);
  }

  #[test]
  fn test_mutate() {
    let mut stack = Stack::from(vec![10, 20, 30, 40, 50]);
    assert_eq!(stack.mutate(1, |x| { *x += 5 }), Ok(()));
    assert_eq!(stack.pop_all(), vec![10, 20, 30, 45, 50]);
  }

  #[test]
  fn test_mutate_negative_index() {
    let mut stack = Stack::from(vec![10, 20, 30, 40, 50]);
    assert_eq!(stack.mutate(-2, |x| { *x += 5 }), Ok(()));
    assert_eq!(stack.pop_all(), vec![10, 25, 30, 40, 50]);
  }

  #[test]
  fn test_mutate_out_of_bounds() {
    let mut stack = Stack::from(vec![10, 20, 30, 40, 50]);
    assert_eq!(
      stack.mutate(9, |x| { *x += 5 }),
      Err(StackError::NotEnoughElements { expected: 10, actual: 5 }),
    );
    assert_eq!(stack.pop_all(), vec![10, 20, 30, 40, 50]);
  }

  #[test]
  fn test_mutate_out_of_bounds_negative_index() {
    let mut stack = Stack::from(vec![10, 20, 30, 40, 50]);
    assert_eq!(
      stack.mutate(-8, |x| { *x += 5 }),
      Err(StackError::NotEnoughElements { expected: 8, actual: 5 }),
    );
    assert_eq!(stack.pop_all(), vec![10, 20, 30, 40, 50]);
  }
}
