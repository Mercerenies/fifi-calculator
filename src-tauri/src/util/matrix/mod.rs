
//! Very rudimentary matrix type which enforces consistency in the
//! dimensions of its data.

mod base;
mod column;
pub mod row_reduction;

pub use base::{MatrixElement, MatrixFieldElement};
pub use column::{Column, ColumnMut};

use crate::util::transpose;
use crate::util::prism::ErrorWithPayload;
use row_reduction::ReducibleMatrix;

use thiserror::Error;
use serde::{Serialize, Deserialize};
use num::{Zero, One};
use try_traits::ops::TryMul;

use std::fmt::{self, Debug};
use std::ops::{Add, Index, IndexMut};

/// A `Matrix<T>` is a vector of vectors of `T` in which each
/// constituent vector has the same length.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(try_from = "Vec<Vec<T>>")]
pub struct Matrix<T> {
  body: Vec<Vec<T>>,
}

/// An index into a matrix. Matrix indices are 0-based, like all Rust
/// data structures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MatrixIndex {
  pub y: usize,
  pub x: usize,
}

#[derive(Clone, Error)]
#[error("The dimensions of the matrix are inconsistent")]
pub struct MatrixDimsError<T> {
  original_data: Vec<Vec<T>>,
}

#[derive(Clone, Debug, Error)]
#[error("Matrix is singular")]
pub struct SingularMatrixError {
  _priv: (),
}

#[derive(Clone, Debug, Error)]
#[error("Dimension mismatch in matrix multiplication")]
pub struct IncompatibleMultiplicationError {
  _priv: (),
}

impl<T> Matrix<T> {
  pub fn new(body: Vec<Vec<T>>) -> Result<Matrix<T>, MatrixDimsError<T>> {
    if body.is_empty() {
      return Ok(Matrix { body });
    }
    if body.iter().any(|row| row.len() != body[0].len()) {
      return Err(MatrixDimsError { original_data: body });
    }
    Ok(Matrix { body })
  }

  /// Calls `generator` for each index in a new `height * width`
  /// matrix to produce elements for that matrix. The generator will
  /// be called in row-major order.
  pub fn from_generator<F>(height: usize, width: usize, mut generator: F) -> Self
  where F: FnMut(MatrixIndex) -> T {
    let body = (0..height)
      .map(|y| (0..width).map(|x| generator(MatrixIndex { y, x })).collect())
      .collect();
    Matrix::new(body).unwrap()
  }

  pub fn identity(size: usize) -> Self
  where T: Zero + One {
    Self::from_generator(size, size, |index| {
      if index.x == index.y { T::one() } else { T::zero() }
    })
  }

  pub fn of_value(height: usize, width: usize, value: T) -> Self
  where T: Clone {
    Matrix::from_generator(height, width, |_| value.clone())
  }

  pub fn of_default(height: usize, width: usize) -> Self
  where T: Default {
    Matrix::from_generator(height, width, |_| T::default())
  }

  pub fn empty() -> Self {
    Matrix::from_generator(0, 0, |_| panic!("Matrix::empty called"))
  }

  pub fn into_row_major(self) -> Vec<Vec<T>> {
    self.body
  }

  pub fn row(&self, index: usize) -> Option<&[T]> {
    self.body.get(index).map(|row| row.as_slice())
  }

  pub fn row_mut(&mut self, index: usize) -> Option<&mut [T]> {
    self.body.get_mut(index).map(|row| row.as_mut_slice())
  }

  pub fn remove_row(&mut self, index: usize) -> Option<Vec<T>> {
    if index >= self.height() {
      return None;
    }
    Some(self.body.remove(index))
  }

  pub fn column(&self, index: usize) -> Option<Column<'_, T>> {
    if index < self.width() {
      Some(Column { matrix: self, column_index: index })
    } else {
      None
    }
  }

  pub fn column_mut(&mut self, index: usize) -> Option<ColumnMut<'_, T>> {
    if index < self.width() {
      Some(ColumnMut { matrix: self, column_index: index })
    } else {
      None
    }
  }

  pub fn remove_column(&mut self, index: usize) -> Option<Vec<T>> {
    if index >= self.width() {
      return None;
    }
    Some(
      self.body.iter_mut()
        .map(|row| row.remove(index))
        .collect()
    )
  }

  pub fn get(&self, index: MatrixIndex) -> Option<&T> {
    self.body
      .get(index.y)
      .and_then(|row| row.get(index.x))
  }

  pub fn get_mut(&mut self, index: MatrixIndex) -> Option<&mut T> {
    self.body
      .get_mut(index.y)
      .and_then(|row| row.get_mut(index.x))
  }

  pub fn rows(&self) -> impl Iterator<Item = &[T]> + '_ {
    self.body.iter().map(|row| row.as_slice())
  }

  pub fn items(&self) -> impl Iterator<Item = &T> + '_ {
    self.body.iter().flat_map(|row| row.iter())
  }

  pub fn into_items(self) -> impl Iterator<Item = T> {
    self.body.into_iter().flatten()
  }

  pub fn width(&self) -> usize {
    self.body
      .first()
      .map(|row| row.len())
      .unwrap_or_default()
  }

  pub fn height(&self) -> usize {
    self.body.len()
  }

  /// Vertically concatenate two matrices. Panics if the matrices have
  /// different widths.
  pub fn vcat(mut self, other: Self) -> Self {
    assert!(self.width() == other.width(), "Cannot concatenate matrices with different widths");
    self.body.extend(other.body);
    self
  }

  /// Horizontally concatenate two matrices. Panics if the matrices
  /// have different heights.
  pub fn hcat(mut self, other: Self) -> Self {
    assert!(self.height() == other.height(), "Cannot concatenate matrices with different heights");
    for (self_row, other_row) in self.body.iter_mut().zip(other.body) {
      self_row.extend(other_row);
    }
    self
  }

  pub fn map<F, U>(self, mut f: F) -> Matrix<U>
  where F: FnMut(T) -> U {
    Matrix {
      body: self
        .body
        .into_iter()
        .map(|row| row.into_iter().map(&mut f).collect())
        .collect(),
    }
  }

  pub fn transpose(self) -> Self {
    Self::new(transpose(self.body)).unwrap()
  }

  pub fn diag(&self) -> impl Iterator<Item=&T> + '_ {
    let diagonal_count = self.width().min(self.height());
    (0..diagonal_count).map(|index| &self[MatrixIndex { x: index, y: index }])
  }

  pub fn trace<'a>(&'a self) -> T
  where T: Zero + Add<&'a T, Output = T> {
    self.diag().fold(T::zero(), |acc, item| acc + item)
  }
}

impl<T: MatrixFieldElement> Matrix<T> {
  /// The determinant of `self`. Panics if `self` is not a square
  /// matrix.
  pub fn determinant(mut self) -> T {
    assert!(self.width() == self.height(), "Can only calculate the determinant of square matrices");
    let mut red_matrix = ReducibleMatrix::new(&mut self);
    red_matrix.reduce_to_row_form();
    let diag_product = red_matrix.as_ref().diag().fold(T::one(), |acc, item| acc * item);
    diag_product / red_matrix.determinant_multiplier()
  }

  /// The inverse of a square matrix. Panics if `self` is not a square
  /// matrix. Returns an error object if `self` is a singular matrix.
  pub fn inverse_matrix(self) -> Result<Matrix<T>, SingularMatrixError> {
    assert!(self.width() == self.height(), "Can only calculate the inverse of square matrices");
    let size = self.width();

    let mut full_matrix = self.hcat(Self::identity(size));
    let mut red_matrix = ReducibleMatrix::new(&mut full_matrix);
    red_matrix.reduce_to_row_form();

    // Validate that the pivots are all non-zero. If any pivot is
    // zero, then the matrix is singular.
    for i in 0..red_matrix.height() {
      if red_matrix[MatrixIndex { x: i, y: i }].is_zero() {
        return Err(SingularMatrixError { _priv: () });
      }
    }

    // Now back-substitute to make the left-hand side resemble the
    // identity matrix.
    for i in 0..red_matrix.height() {
      let recip = T::one() / &red_matrix[MatrixIndex { x: i, y: i }];
      red_matrix.multiply(i, recip);
      for j in 0..i {
        red_matrix.add_to_row(j, - red_matrix[MatrixIndex { x: i, y: j }].clone(), i);
      }
    }

    let final_vec_of_vecs = red_matrix.as_mut().body.iter_mut()
      .map(|row| row.drain(size..).collect::<Vec<_>>())
      .collect::<Vec<_>>();
    Ok(Matrix::new(final_vec_of_vecs).unwrap())
  }
}

impl MatrixIndex {
  pub fn flipped(self) -> MatrixIndex {
    MatrixIndex { x: self.y, y: self.x }
  }
}

impl<T> Debug for MatrixDimsError<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "MatrixDimsError {{ ... }}")
  }
}

impl<T: MatrixElement> TryMul<&Matrix<T>> for &Matrix<T> {
  type Output = Matrix<T>;
  type Error = IncompatibleMultiplicationError;

  fn try_mul(self, other: &Matrix<T>) -> Result<Matrix<T>, IncompatibleMultiplicationError> {
    if self.width() != other.height() {
      return Err(IncompatibleMultiplicationError { _priv: () });
    }
    let middle_dimension = self.width();
    Ok(Matrix::from_generator(self.height(), other.width(), |index| {
      (0..middle_dimension)
        .map(|i| self[ MatrixIndex { x: i, y: index.y } ].clone() * &other[ MatrixIndex { x: index.x, y: i } ] )
        .fold(T::zero(), |acc, item| acc + item)
    }))
  }
}

impl<T> Default for Matrix<T> {
  fn default() -> Self {
    Matrix::empty()
  }
}

impl<T> TryFrom<Vec<Vec<T>>> for Matrix<T> {
  type Error = MatrixDimsError<T>;

  fn try_from(body: Vec<Vec<T>>) -> Result<Self, Self::Error> {
    Self::new(body)
  }
}

impl<T> Index<usize> for Matrix<T> {
  type Output = [T];

  fn index(&self, index: usize) -> &Self::Output {
    &self.body[index]
  }
}

impl<T> Index<MatrixIndex> for Matrix<T> {
  type Output = T;

  fn index(&self, index: MatrixIndex) -> &Self::Output {
    &self.body[index.y][index.x]
  }
}

impl<T> IndexMut<MatrixIndex> for Matrix<T> {
  fn index_mut(&mut self, index: MatrixIndex) -> &mut Self::Output {
    &mut self.body[index.y][index.x]
  }
}

impl<T: Serialize> Serialize for Matrix<T> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: serde::Serializer {
    self.body.serialize(serializer)
  }
}

impl<T> ErrorWithPayload<Vec<Vec<T>>> for MatrixDimsError<T> {
  fn recover_payload(self) -> Vec<Vec<T>> {
    self.original_data
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_roundtrip_serialize() {
    let matrix = Matrix::from_generator(5, 5, |idx| idx.y + idx.x);
    let json = serde_json::to_string(&matrix).unwrap();
    let deserialized_matrix: Matrix<usize> = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized_matrix, matrix);
  }

  #[test]
  fn test_row_accessor() {
    let matrix = Matrix::new(vec![
      vec![1, 2, 3],
      vec![4, 5, 6],
      vec![7, 8, 9],
    ]).unwrap();
    assert_eq!(&matrix[0], [1, 2, 3].as_slice());
    assert_eq!(&matrix[1], [4, 5, 6].as_slice());
    assert_eq!(&matrix[2], [7, 8, 9].as_slice());
    assert_eq!(matrix.row(0), Some([1, 2, 3].as_slice()));
    assert_eq!(matrix.row(1), Some([4, 5, 6].as_slice()));
    assert_eq!(matrix.row(2), Some([7, 8, 9].as_slice()));
    assert_eq!(matrix.row(3), None);
  }

  #[test]
  fn test_column_accessor() {
    let mut matrix = Matrix::new(vec![
      vec![1, 2, 3],
      vec![4, 5, 6],
      vec![7, 8, 9],
    ]).unwrap();
    assert_eq!(matrix.column(0).unwrap().to_owned(), vec![1, 4, 7]);
    assert_eq!(matrix.column(1).unwrap().to_owned(), vec![2, 5, 8]);
    assert_eq!(matrix.column(2).unwrap().to_owned(), vec![3, 6, 9]);
    assert!(matrix.column(3).is_none());

    let mut col = matrix.column_mut(1).unwrap();
    col[2] = 99;
    assert_eq!(matrix.into_row_major(), vec![
      vec![1, 2, 3],
      vec![4, 5, 6],
      vec![7, 99, 9],
    ]);
  }

  #[test]
  fn test_remove_row() {
    let mut matrix = Matrix::new(vec![
      vec![1, 2, 3],
      vec![4, 5, 6],
      vec![7, 8, 9],
    ]).unwrap();
    assert_eq!(matrix.remove_row(0), Some(vec![1, 2, 3]));
    assert_eq!(matrix.into_row_major(), vec![
      vec![4, 5, 6],
      vec![7, 8, 9],
    ]);
  }

  #[test]
  fn test_remove_column() {
    let mut matrix = Matrix::new(vec![
      vec![1, 2, 3],
      vec![4, 5, 6],
      vec![7, 8, 9],
    ]).unwrap();
    assert_eq!(matrix.remove_column(0), Some(vec![1, 4, 7]));
    assert_eq!(matrix.into_row_major(), vec![
      vec![2, 3],
      vec![5, 6],
      vec![8, 9],
    ]);
  }

  #[test]
  #[should_panic]
  fn test_determinant_non_square() {
    let matrix = Matrix::new(vec![
      vec![1.0, 2.0],
      vec![3.0, 4.0],
      vec![5.0, 6.0],
    ]).unwrap();
    matrix.determinant();
  }

  #[test]
  fn test_determinant() {
    let matrix = Matrix::new(vec![
      vec![1.0, 2.0, 3.0],
      vec![4.0, 5.0, 6.0],
      vec![7.0, -8.0, 9.0],
    ]).unwrap();
    assert_eq!(matrix.determinant(), -96.0);
  }

  #[test]
  fn test_determinant_zero() {
    let matrix = Matrix::new(vec![
      vec![1.0, 2.0, 3.0],
      vec![4.0, 5.0, 6.0],
      vec![7.0, 8.0, 9.0],
    ]).unwrap();
    assert_eq!(matrix.determinant(), 0.0);
  }

  #[test]
  fn test_inverse_matrix() {
    let matrix = Matrix::new(vec![
      vec![1.0, 2.0],
      vec![2.0, 2.0],
    ]).unwrap();
    assert_eq!(matrix.inverse_matrix().unwrap(), Matrix::new(vec![
      vec![-1.0, 1.0],
      vec![1.0, -0.5],
    ]).unwrap());
  }

  #[test]
  fn test_inverse_matrix_of_singular_matrix() {
    let matrix = Matrix::new(vec![
      vec![1.0, 2.0, 3.0],
      vec![4.0, 5.0, 6.0],
      vec![7.0, 8.0, 9.0],
    ]).unwrap();
    matrix.inverse_matrix().unwrap_err();
  }

  #[test]
  fn test_matrix_multiplication() {
    let a = Matrix::new(vec![
      vec![1, 2, 3],
      vec![4, 5, 6],
      vec![7, 8, 9],
    ]).unwrap();
    let b = Matrix::new(vec![
      vec![1, 0, 2],
      vec![0, 3, 1],
      vec![2, 1, 0],
    ]).unwrap();
    assert_eq!(a.try_mul(&b).unwrap(), Matrix::new(vec![
      vec![7, 9, 4],
      vec![16, 21, 13],
      vec![25, 33, 22],
    ]).unwrap());
  }

  #[test]
  fn test_matrix_multiplication_non_square() {
    let a = Matrix::new(vec![
      vec![1, 2],
      vec![4, 5],
      vec![7, 8],
    ]).unwrap();
    let b = Matrix::new(vec![
      vec![0, 3, 1],
      vec![2, 1, 0],
    ]).unwrap();
    assert_eq!(a.try_mul(&b).unwrap(), Matrix::new(vec![
      vec![4, 5, 1],
      vec![10, 17, 4],
      vec![16, 29, 7],
    ]).unwrap());
  }

  #[test]
  fn test_matrix_multiplication_with_bad_dims() {
    let a = Matrix::new(vec![
      vec![1, 2, 3],
      vec![4, 5, 6],
      vec![7, 8, 9],
    ]).unwrap();
    let b = Matrix::new(vec![
      vec![0, 3, 1],
      vec![2, 1, 0],
    ]).unwrap();
    a.try_mul(&b).unwrap_err();
  }
}
