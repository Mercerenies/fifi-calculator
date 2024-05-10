
//! Miscellaneous stack shuffling commands and helpers.

use super::structure::Stack;
use super::error::StackError;

pub fn pop_one<T>(stack: &mut Stack<T>) -> Result<T, StackError> {
  let Ok([a]) = TryInto::<[_; 1]>::try_into(pop_several(stack, 1)?) else {
    panic!("Vec should have exactly one element");
  };
  Ok(a)
}

/// Returns a pair of elements, with the previous top of the stack
/// being the second element in the tuple.
pub fn pop_two<T>(stack: &mut Stack<T>) -> Result<(T, T), StackError> {
  let Ok([a, b]) = TryInto::<[_; 2]>::try_into(pop_several(stack, 2)?) else {
    panic!("Vec should have exactly two elements");
  };
  Ok((a, b))
}

/// Pops `count` elements off the stack. Returns those elements, with
/// the former top of the stack at the end of the vector. In case of a
/// [`StackError`], the `stack` has NOT been modified.
pub fn pop_several<T>(stack: &mut Stack<T>, count: usize) -> Result<Vec<T>, StackError> {
  if count > stack.len() {
    return Err(StackError::NotEnoughElements { expected: count, actual: stack.len() });
  }
  let mut result: Vec<T> = Vec::with_capacity(count);
  for _ in 0..count {
    // expect: Stack size was just asserted, so this must be safe.
    result.push(stack.pop().expect("Popping from empty stack"));
  }
  result.reverse();
  Ok(result)
}
