
use super::error::StackError;

use std::ops::{Deref, DerefMut};

/// A stack-like structure, capable of pushing and popping elements
/// from the top.
pub trait StackLike<T> {
  /// Returns the length of the stack, in elements.
  fn len(&self) -> usize;

  /// Pushes a single element onto the top of the stack.
  fn push(&mut self, element: T);

  /// Pops a single element from the stack. Returns an appropriate
  /// [`StackError`] if the stack is empty.
  fn pop(&mut self) -> Result<T, StackError>;

  /// Pushes several elements onto the stack in the order we see them
  /// in the iterable. That is, the final element of the iterable will
  /// be at the top of the stack, after this method has executed.
  ///
  /// The default implementation merely pushes the elements in order,
  /// but a more efficient alternative can be provided by
  /// implementors.
  fn push_several(&mut self, elements: impl IntoIterator<Item = T>) {
    for element in elements {
      self.push(element);
    }
  }

  /// As [`StackLike::pop`], but with no result value. Use this
  /// function if you don't plan to use the result and don't care if
  /// the `pop` call fails due to an empty stack.
  fn pop_and_discard(&mut self) {
    let _ = self.pop();
  }

  /// Pops `count` elements off the stack and returns those elements,
  /// with the former top of the stack at the end of the vector. In
  /// case of a [`StackError`], `self` will NOT be modified.
  ///
  /// The default implementation pops the elements one at a time, but
  /// a more efficient version can be provided by implementing types.
  fn pop_several(&mut self, count: usize) -> Result<Vec<T>, StackError> {
    self.check_stack_size(count)?;
    let mut result = Vec::new();
    for _ in 0..count {
      result.push(self.pop().unwrap()); // unwrap: We checked the stack size already
    }
    result.reverse();
    Ok(result)
  }

  /// Pops all elements off the stack and returns them.
  fn pop_all(&mut self) -> Vec<T> {
    // unwrap: We're popping exactly as many elements as the stack contains.
    self.pop_several(self.len()).unwrap()
  }

  /// Returns true if the stack is empty.
  fn is_empty(&self) -> bool {
    self.len() == 0
  }

  /// Asserts that the stack has size at least `expected` but does not
  /// pop anything.
  fn check_stack_size(&self, expected: usize) -> Result<(), StackError> {
    let actual = self.len();
    if actual < expected {
      Err(StackError::NotEnoughElements { expected, actual })
    } else {
      Ok(())
    }
  }
}

/// A stack-like structure that provides random access to its
/// elements.
pub trait RandomAccessStackLike<T>: StackLike<T> {
  /// The type of immutable references to stack elements.
  type Ref<'a>: Deref<Target = T> where Self: 'a;

  /// The type of mutable references to stack elements.
  type Mut<'a>: DerefMut<Target = T> where Self: 'a;

  /// Returns a reference to the value at the given position on the
  /// stack, or a [`StackError`] if out of bounds.
  ///
  /// Nonnegative indices index from the top of the stack, while
  /// negative indices index from the bottom. For example, zero always
  /// refers to the top of the stack, while -1 always refers to the
  /// bottom.
  fn get(&self, index: i64) -> Result<Self::Ref<'_>, StackError>;

  /// Returns a mutable reference the given position on the stack,
  /// using the same indexing rules as [`get`](RandomAccessStackLike::get).
  fn get_mut(&mut self, index: i64) -> Result<Self::Mut<'_>, StackError>;

  /// Modifies the value at the given position, using the given
  /// function.
  ///
  /// By default, this is equivalent to simply calling
  /// [`get_mut`](RandomAccessStackLike::get_mut) and making the
  /// modification directly, but this may be overridden if desired.
  fn mutate<F>(&mut self, index: i64, f: F) -> Result<(), StackError>
  where F: FnOnce(&mut T) {
    let mut value = self.get_mut(index)?;
    f(value.deref_mut());
    Ok(())
  }
}
