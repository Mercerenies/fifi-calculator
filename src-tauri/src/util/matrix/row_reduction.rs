
use super::Matrix;
use super::base::MatrixElement;
use crate::util::double_borrow_mut;

/// A mutable reference to a [`Matrix`], on which elementary row
/// operations can be applied.
pub struct ReducibleMatrix<'a, T> {
  matrix: &'a mut Matrix<T>,
  determinant_multiplier: T,
}

impl<'a, T: MatrixElement> ReducibleMatrix<'a, T> {
  pub fn new(matrix: &'a mut Matrix<T>) -> Self {
    Self {
      matrix,
      determinant_multiplier: T::one(),
    }
  }

  pub fn height(&self) -> usize {
    self.matrix.height()
  }

  pub fn width(&self) -> usize {
    self.matrix.width()
  }

  pub fn determinant_multiplier(&self) -> &T {
    &self.determinant_multiplier
  }

  /// Swaps two rows of the matrix. Panics on out of bounds.
  pub fn swap_rows(&mut self, a: usize, b: usize) {
    if a == b {
      return; // No-op
    }
    self.matrix.body.swap(a, b);
    self.determinant_multiplier = - T::one() * &self.determinant_multiplier;
  }

  /// Multiplies a row of the matrix by a nonzero scalar. It is the
  /// caller's responsibility to ensure that the scalar is nonzero.
  pub fn multiply(&mut self, row_index: usize, multiplier: T) {
    for elem in &mut self.matrix.body[row_index] {
      *elem = elem.clone() * &multiplier;
    }
    self.determinant_multiplier = multiplier * &self.determinant_multiplier;
  }

  /// Adds a scalar multiple of a row to another row.
  ///
  /// Panics if the two row indices are equal.
  pub fn add_to_row(&mut self, row_index: usize, multiplier: T, addend_index: usize) {
    assert!(row_index != addend_index, "add_to_row cannot add a row to itself");
    let (row, addend_row) = double_borrow_mut(&mut self.matrix.body, row_index, addend_index);

    for (elem, addend) in row.iter_mut().zip(addend_row) {
      *elem = multiplier.clone() * &*addend + elem;
    }
  }
}

impl<T> AsRef<Matrix<T>> for ReducibleMatrix<'_, T> {
  fn as_ref(&self) -> &Matrix<T> {
    self.matrix
  }
}

impl<T> AsMut<Matrix<T>> for ReducibleMatrix<'_, T> {
  fn as_mut(&mut self) -> &mut Matrix<T> {
    self.matrix
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn sample_matrix() -> Matrix<i32> {
    Matrix::new(vec![
      vec![1, 2, 3],
      vec![4, 5, 6],
      vec![7, 8, 9],
    ]).unwrap()
  }

  #[test]
  fn test_swap_rows() {
    let mut matrix = sample_matrix();
    let mut red_matrix = ReducibleMatrix::new(&mut matrix);
    red_matrix.swap_rows(0, 1);
    assert_eq!(red_matrix.matrix.body, vec![
      vec![4, 5, 6],
      vec![1, 2, 3],
      vec![7, 8, 9],
    ]);
    assert_eq!(red_matrix.determinant_multiplier, -1);
  }

  #[test]
  fn test_swap_rows_noop() {
    let mut matrix = sample_matrix();
    let mut red_matrix = ReducibleMatrix::new(&mut matrix);
    red_matrix.swap_rows(0, 0);
    assert_eq!(red_matrix.matrix.body, vec![
      vec![1, 2, 3],
      vec![4, 5, 6],
      vec![7, 8, 9],
    ]);
    assert_eq!(red_matrix.determinant_multiplier, 1);
  }

  #[test]
  fn test_multiply_by_scalar() {
    let mut matrix = sample_matrix();
    let mut red_matrix = ReducibleMatrix::new(&mut matrix);
    red_matrix.multiply(0, 2);
    assert_eq!(red_matrix.matrix.body, vec![
      vec![2, 4, 6],
      vec![4, 5, 6],
      vec![7, 8, 9],
    ]);
    assert_eq!(red_matrix.determinant_multiplier, 2);
  }

  #[test]
  fn test_add_row() {
    let mut matrix = sample_matrix();
    let mut red_matrix = ReducibleMatrix::new(&mut matrix);
    red_matrix.add_to_row(0, -3, 1);
    assert_eq!(red_matrix.matrix.body, vec![
      vec![-11, -13, -15],
      vec![4, 5, 6],
      vec![7, 8, 9],
    ]);
    assert_eq!(red_matrix.determinant_multiplier, 1);
  }
}
