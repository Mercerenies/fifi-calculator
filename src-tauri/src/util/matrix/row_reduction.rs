
use super::{Matrix, MatrixIndex};
use super::base::{MatrixElement, MatrixFieldElement};
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

  /// Multiplies a row of the matrix by a nonzero scalar. Panics if
  /// the scalar is zero.
  pub fn multiply(&mut self, row_index: usize, multiplier: T) {
    assert!(!multiplier.is_zero(), "Cannot multiply row by zero");
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
      *elem = multiplier.clone() * &*addend + &*elem;
    }
  }
}

impl<'a, T: MatrixFieldElement> ReducibleMatrix<'a, T> {
  /// Reduces the matrix to row echelon form. That is, this function
  /// applies elementary row operations to the matrix such that the
  /// resulting matrix
  ///
  /// * is upper triangular,
  ///
  /// * has all of its zero rows at the bottom, and
  ///
  /// * the pivot of each row is strictly to the right of the pivot of
  /// the row above it.
  pub fn reduce_to_row_form(&mut self) {
    fn find_pivot<T: MatrixFieldElement>(matrix: &Matrix<T>, column_index: usize) -> Option<usize> {
      (column_index..matrix.height()).find(|i| !matrix[MatrixIndex { y: column_index, x: *i }].is_zero())
    }

    for i in 0..self.height().min(self.width()) {
      // Find the pivot for column i and swap it into row i.
      let Some(pivot_index) = find_pivot(self.matrix, i) else { continue; };
      self.swap_rows(i, pivot_index);
      // Reduce all values below the pivot to zero.
      let pivot_value = self.matrix[MatrixIndex { y: i, x: i }].clone();
      for j in (i+1)..self.height() {
        let curr_value = &self.matrix[MatrixIndex { y: j, x: i }];
        if !curr_value.is_zero() {
          self.add_to_row(j, - curr_value.clone() / &pivot_value, i);
        }
      }
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

  #[test]
  fn test_row_reduce_matrix_already_in_row_echelon_form() {
    let mut matrix: Matrix<f64> = Matrix::new(vec![
      vec![1.0, 2.0, 3.0, 6.0],
      vec![0.0, 2.0, 1.0, 9.0],
      vec![0.0, 0.0, 1.0, -1.0],
    ]).unwrap();
    let original_matrix = matrix.clone();
    ReducibleMatrix::new(&mut matrix).reduce_to_row_form();
    assert_eq!(matrix, original_matrix);
  }

  #[test]
  fn test_row_reduce_matrix_with_only_swaps() {
    let mut matrix: Matrix<f64> = Matrix::new(vec![
      vec![0.0, 0.0, 1.0, -1.0],
      vec![0.0, 2.0, 1.0, 9.0],
      vec![1.0, 2.0, 3.0, 6.0],
    ]).unwrap();
    ReducibleMatrix::new(&mut matrix).reduce_to_row_form();
    assert_eq!(matrix, Matrix::new(vec![
      vec![1.0, 2.0, 3.0, 6.0],
      vec![0.0, 2.0, 1.0, 9.0],
      vec![0.0, 0.0, 1.0, -1.0],
    ]).unwrap());
  }

  #[test]
  fn test_row_reduce_matrix() {
    let mut matrix: Matrix<f64> = Matrix::new(vec![
      vec![1.0, 2.0, 3.0],
      vec![1.0, 2.0, 4.0],
      vec![1.0, -1.0, 5.0],
    ]).unwrap();
    ReducibleMatrix::new(&mut matrix).reduce_to_row_form();
    assert_eq!(matrix, Matrix::new(vec![
      vec![1.0, 2.0, 3.0],
      vec![0.0, -3.0, 2.0],
      vec![0.0, 0.0, 1.0],
    ]).unwrap());
  }
}
