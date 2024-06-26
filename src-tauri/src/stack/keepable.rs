
use super::base::{StackLike, RandomAccessStackLike};
use super::error::StackError;

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
/// // `stack` is a regular `Stack<i64>`.
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
/// # let mut stack = KeepableStack::new(Stack::new(), true);
/// // `stack` is a `KeepableStack`.
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
/// # let mut stack = KeepableStack::new(Stack::new(), true);
/// /// `stack` is a `KeepableStack`.
/// stack.push_several(vec![10, 20, 30, 40]);
/// assert_eq!(stack.pop_several(2), Ok(vec![30, 40]));
/// assert_eq!(stack.len(), 4);
/// ```
#[derive(Debug)]
pub struct KeepableStack<S> {
  stack: S,
  keep_semantics: bool,
}

impl<S> KeepableStack<S> where S: StackLike {
  pub fn new(stack: S, keep_semantics: bool) -> Self {
    Self {
      stack,
      keep_semantics,
    }
  }

  pub fn get_inner_mut(&mut self) -> &mut S {
    &mut self.stack
  }

  pub fn into_inner(self) -> S {
    self.stack
  }

  pub fn keep_semantics(&self) -> bool {
    self.keep_semantics
  }
}

impl<S> StackLike for KeepableStack<S>
where S: StackLike,
      S::Elem: Clone {
  type Elem = S::Elem;

  fn len(&self) -> usize {
    self.stack.len()
  }

  fn push(&mut self, element: S::Elem) {
    self.stack.push(element);
  }

  fn push_several(&mut self, elements: impl IntoIterator<Item = S::Elem>) {
    self.stack.push_several(elements);
  }

  fn pop(&mut self) -> Result<S::Elem, StackError> {
    let value = self.stack.pop()?;
    if self.keep_semantics {
      self.stack.push(value.clone());
    }
    Ok(value)
  }

  fn pop_several(&mut self, count: usize) -> Result<Vec<S::Elem>, StackError> {
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
impl<S> RandomAccessStackLike for KeepableStack<S>
where S: RandomAccessStackLike,
      S::Elem: Clone {
  type Ref<'a> = S::Ref<'a> where Self: 'a;
  type Mut<'a> = S::Mut<'a> where Self: 'a;

  fn get(&self, index: i64) -> Result<S::Ref<'_>, StackError> {
    self.stack.get(index)
  }

  fn get_mut(&mut self, index: i64) -> Result<S::Mut<'_>, StackError> {
    self.stack.get_mut(index)
  }

  fn insert(&mut self, index: usize, element: S::Elem) -> Result<(), StackError> {
    self.stack.insert(index, element)
  }

  fn pop_nth(&mut self, index: usize) -> Result<S::Elem, StackError> {
    if self.keep_semantics {
      self.stack.get(index as i64).map(|elem| elem.clone())
    } else {
      self.stack.pop_nth(index)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::stack::Stack;

  #[test]
  fn test_push_pop_with_no_keep_semantics() {
    let mut stack = KeepableStack::new(Stack::from(vec![0, 10]), false);
    stack.push(20);
    assert_eq!(stack.pop(), Ok(20));
    assert_eq!(stack.pop(), Ok(10));
    assert_eq!(stack.pop(), Ok(0));
    assert_eq!(stack.pop(), Err(StackError::NotEnoughElements { expected: 1, actual: 0 }));
  }

  #[test]
  fn test_push_pop_with_active_keep_semantics() {
    let mut stack = KeepableStack::new(Stack::from(vec![0, 10]), true);
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
    assert_eq!(stack.into_inner().into_iter().collect::<Vec<_>>(), vec![0, 10, 20, 30, 40]);
  }

  #[test]
  fn test_push_several_with_no_keep_semantics() {
    let stack = Stack::from(vec![0, 10, 20, 30, 40]);
    let mut stack = KeepableStack::new(stack, false);
    stack.push_several(vec![50, 60, 70]);
    let elements = stack.into_inner().into_iter().collect::<Vec<_>>();
    assert_eq!(elements, vec![0, 10, 20, 30, 40, 50, 60, 70]);
  }

  #[test]
  fn test_push_several_with_keep_semantics() {
    let stack = Stack::from(vec![0, 10, 20, 30, 40]);
    let mut stack = KeepableStack::new(stack, true);
    stack.push_several(vec![50, 60, 70]);
    let elements = stack.into_inner().into_iter().collect::<Vec<_>>();
    assert_eq!(elements, vec![0, 10, 20, 30, 40, 50, 60, 70]);
  }

  #[test]
  fn test_pop_several_with_no_keep_semantics() {
    let stack = Stack::from(vec![0, 10, 20, 30, 40]);
    let mut stack = KeepableStack::new(stack, false);
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
    let stack = Stack::from(vec![0, 10, 20, 30, 40]);
    let mut stack = KeepableStack::new(stack, true);
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
    let mut stack = KeepableStack::new(Stack::new(), false);
    assert_eq!(stack.len(), 0);
    stack.push(0);
    assert_eq!(stack.len(), 1);
    stack.push(0);
    stack.push(0);
    assert_eq!(stack.len(), 3);
    let _ = stack.pop();
    assert_eq!(stack.len(), 2);

    let stack = stack.into_inner();
    let mut stack = KeepableStack::new(stack, true);
    assert_eq!(stack.len(), 2);
    stack.push(0);
    assert_eq!(stack.len(), 3);
    stack.push(0);
    stack.push(0);
    assert_eq!(stack.len(), 5);
    let _ = stack.pop(); // Doesn't actually pop, since the stack is kept!
    assert_eq!(stack.len(), 5);
    assert_eq!(stack.into_inner().len(), 5);
  }

  #[test]
  fn test_is_empty_without_keep_semantics() {
    let stack = Stack::new();
    let mut stack = KeepableStack::new(stack, false);
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
    let stack = Stack::new();
    let mut stack = KeepableStack::new(stack, true);
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

  #[test]
  fn test_insert_with_no_keep_semantics() {
    let mut stack = KeepableStack::new(Stack::from(vec![0, 10]), false);
    stack.push(20);
    stack.push(30);
    stack.push(40);
    stack.insert(3, 50).unwrap();
    stack.insert(0, 60).unwrap();
    stack.insert(7, 70).unwrap();
    assert_eq!(stack.insert(9, 80).unwrap_err(), StackError::NotEnoughElements { expected: 9, actual: 8 });
    let elements = stack.into_inner().into_iter().collect::<Vec<_>>();
    assert_eq!(elements, vec![70, 0, 10, 50, 20, 30, 40, 60]);
  }

  #[test]
  fn test_insert_with_keep_semantics() {
    // keep_semantics do not affect insertion operations, so this
    // should behave equivalently to
    // test_insert_with_no_keep_semantics()
    let mut stack = KeepableStack::new(Stack::from(vec![0, 10]), true);
    stack.push(20);
    stack.push(30);
    stack.push(40);
    stack.insert(3, 50).unwrap();
    stack.insert(0, 60).unwrap();
    stack.insert(7, 70).unwrap();
    assert_eq!(stack.insert(9, 80).unwrap_err(), StackError::NotEnoughElements { expected: 9, actual: 8 });
    let elements = stack.into_inner().into_iter().collect::<Vec<_>>();
    assert_eq!(elements, vec![70, 0, 10, 50, 20, 30, 40, 60]);
  }

  #[test]
  fn test_pop_nth_with_no_keep_semantics() {
    let stack = Stack::from(vec![10, 20, 30, 40, 50]);
    let mut stack = KeepableStack::new(stack, false);
    assert_eq!(stack.pop_nth(0), Ok(50));
    assert_eq!(stack.pop_nth(2), Ok(20));
    assert_eq!(stack.pop_nth(2), Ok(10));
    assert_eq!(stack.pop_nth(2), Err(StackError::NotEnoughElements { expected: 3, actual: 2 }));
    let elements = stack.into_inner().into_iter().collect::<Vec<_>>();
    assert_eq!(elements, vec![30, 40]);
  }

  #[test]
  fn test_pop_nth_with_keep_semantics() {
    let stack = Stack::from(vec![10, 20, 30, 40, 50]);
    let mut stack = KeepableStack::new(stack, true);
    assert_eq!(stack.pop_nth(0), Ok(50));
    assert_eq!(stack.pop_nth(0), Ok(50));
    assert_eq!(stack.pop_nth(0), Ok(50));
    assert_eq!(stack.pop_nth(2), Ok(30));
    assert_eq!(stack.pop_nth(2), Ok(30));
    assert_eq!(stack.pop_nth(4), Ok(10));
    assert_eq!(stack.pop_nth(4), Ok(10));
    assert_eq!(stack.pop_nth(5), Err(StackError::NotEnoughElements { expected: 6, actual: 5 }));
    let elements = stack.into_inner().into_iter().collect::<Vec<_>>();
    assert_eq!(elements, vec![10, 20, 30, 40, 50]);
  }
}
