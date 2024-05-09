
/// FIFO stack. Implemented internally as a vector whose "top" is at
/// the end, allowing for constant-time pushes and pops.
#[derive(Clone)]
pub struct Stack<T> {
  elements: Vec<T>,
}

impl<T> Stack<T> {

  pub fn push(&mut self, element: T) {
    self.elements.push(element);
  }

  pub fn pop(&mut self) -> Option<T> {
    self.elements.pop()
  }

  pub fn len(&self) -> usize {
    self.elements.len()
  }

  fn to_vec_index(&self, index: i64) -> usize {
    if index < 0 {
      (- index - 1) as usize
    } else {
      self.len() - index as usize - 1 as usize
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

}

impl<T> Default for Stack<T> {

  fn default() -> Self {
    Self {
      elements: Vec::with_capacity(10),
    }
  }

}
