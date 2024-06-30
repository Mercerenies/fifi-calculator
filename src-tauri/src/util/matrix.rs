
//! Very rudimentary matrix type which enforces consistency in the
//! dimensions of its data.

use thiserror::Error;
use serde::{Serialize, Deserialize};

use std::ops::{Index, IndexMut};

/// A `Matrix<T>` is a vector of vectors of `T` in which each
/// constituent vector has the same length.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(try_from = "Vec<Vec<T>>")]
pub struct Matrix<T> {
  body: Vec<Vec<T>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MatrixIndex {
  pub y: usize,
  pub x: usize,
}

#[derive(Debug, Clone, Error)]
#[error("The dimensions of the matrix are inconsistent")]
pub struct MatrixDimsError {
  _priv: (),
}

impl<T> Matrix<T> {
  pub fn new(body: Vec<Vec<T>>) -> Result<Matrix<T>, MatrixDimsError> {
    if body.is_empty() {
      return Ok(Matrix { body });
    }
    if body.iter().any(|row| row.len() != body[0].len()) {
      return Err(MatrixDimsError { _priv: () });
    }
    Ok(Matrix { body })
  }

  pub fn from_generator<F>(height: usize, width: usize, mut generator: F) -> Self
  where F: FnMut(MatrixIndex) -> T {
    let body = (0..height)
      .map(|y| (0..width).map(|x| generator(MatrixIndex { y, x })).collect())
      .collect();
    Matrix::new(body).unwrap()
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
}

impl<T> TryFrom<Vec<Vec<T>>> for Matrix<T> {
  type Error = MatrixDimsError;

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
}
