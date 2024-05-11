
/// FIFO stack. Implemented internally as a vector whose "top" is at
/// the end, allowing for constant-time pushes and pops.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stack<T> {
  elements: Vec<T>,
}

impl<T> Stack<T> {

  pub fn new() -> Self {
    Self::default()
  }

  pub fn push(&mut self, element: T) {
    self.elements.push(element);
  }

  pub fn pop(&mut self) -> Option<T> {
    self.elements.pop()
  }

  pub fn len(&self) -> usize {
    self.elements.len()
  }

  pub fn is_empty(&self) -> bool {
    self.elements.is_empty()
  }

  fn to_vec_index(&self, index: i64) -> usize {
    if index < 0 {
      (- index - 1) as usize
    } else {
      // Wrap at the boundary. If the index is out of bounds, we'll
      // get an absurdly large number, and `Vec::get` will simply
      // return `None`.
      usize::wrapping_sub(self.len(), (index + 1) as usize)
    }
  }

  /// Stacks index from the top of the stack, so index zero is always
  /// the very top. Negative indices can be used to index from the
  /// bottom.
  pub fn get(&self, index: i64) -> Option<&T> {
    let index = self.to_vec_index(index);
    self.elements.get(index)
  }

  pub fn get_mut(&mut self, index: i64) -> Option<&mut T> {
    let index = self.to_vec_index(index);
    self.elements.get_mut(index)
  }

  /// Iterates from the top of the stack.
  pub fn iter(&self) -> impl DoubleEndedIterator<Item = &T> {
    self.elements.iter().rev()
  }

  /// Iterates (with mutable references) from the top of the stack.
  pub fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut T> {
    self.elements.iter_mut().rev()
  }

  /// Iterates (by value) from the top of the stack.
  pub fn into_iter(self) -> impl DoubleEndedIterator<Item = T> {
    self.elements.into_iter().rev()
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
    assert_eq!(stack.pop(), Some(20));
    assert_eq!(stack.pop(), Some(10));
    assert_eq!(stack.pop(), Some(0));
    assert_eq!(stack.pop(), None);
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
    stack.pop();
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
    stack.pop();
    assert!(!stack.is_empty());
    stack.pop();
    assert!(stack.is_empty());
  }

  #[test]
  fn test_get() {
    let stack = Stack::from(vec!['A', 'B', 'C', 'D']);
    // Regular indexing
    assert_eq!(stack.get(0), Some(&'D'));
    assert_eq!(stack.get(1), Some(&'C'));
    assert_eq!(stack.get(2), Some(&'B'));
    assert_eq!(stack.get(3), Some(&'A'));
    assert_eq!(stack.get(4), None);
    // Negative indexing
    assert_eq!(stack.get(-1), Some(&'A'));
    assert_eq!(stack.get(-2), Some(&'B'));
    assert_eq!(stack.get(-3), Some(&'C'));
    assert_eq!(stack.get(-4), Some(&'D'));
    assert_eq!(stack.get(-5), None);
  }

  #[test]
  fn test_get_mut() {
    let mut stack = Stack::from(vec!['A', 'B', 'C', 'D']);
    // Regular indexing
    assert_eq!(stack.get_mut(0), Some(&mut 'D'));
    assert_eq!(stack.get_mut(1), Some(&mut 'C'));
    assert_eq!(stack.get_mut(2), Some(&mut 'B'));
    assert_eq!(stack.get_mut(3), Some(&mut 'A'));
    assert_eq!(stack.get_mut(4), None);
    // Negative indexing
    assert_eq!(stack.get_mut(-1), Some(&mut 'A'));
    assert_eq!(stack.get_mut(-2), Some(&mut 'B'));
    assert_eq!(stack.get_mut(-3), Some(&mut 'C'));
    assert_eq!(stack.get_mut(-4), Some(&mut 'D'));
    assert_eq!(stack.get_mut(-5), None);
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
    assert_eq!(stack.pop(), Some(5));
    assert_eq!(stack.pop(), Some(999));
    assert_eq!(stack.pop(), Some(3));
    assert_eq!(stack.pop(), Some(2));
    assert_eq!(stack.pop(), Some(1));
    assert_eq!(stack.pop(), None);
  }

  #[test]
  fn test_into_iter() {
    let stack = Stack::from(vec!['A', 'B', 'C', 'D']);
    let vec = stack.into_iter().collect::<Vec<_>>();
    assert_eq!(vec, vec!['D', 'C', 'B', 'A']);
  }
}
