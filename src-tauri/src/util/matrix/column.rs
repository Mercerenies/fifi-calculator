
use super::{Matrix, MatrixIndex};

use std::ops::{Index, IndexMut};

/// A (borrowed) column of a [`Matrix`]. It can be assumed that a
/// [`Column`] is always in-bounds.
#[derive(Debug, Clone)]
pub struct Column<'a, T> {
  pub(super) matrix: &'a Matrix<T>,
  pub(super) column_index: usize,
}

/// A column of a [`Matrix`], borrowed mutably. It can be assumed that
/// a [`ColumnMut`] is always in-bounds.
#[derive(Debug)]
pub struct ColumnMut<'a, T> {
  pub(super) matrix: &'a mut Matrix<T>,
  pub(super) column_index: usize,
}

impl<'a, T> Column<'a, T> {
  pub fn get(&self, index: usize) -> Option<&T> {
    self.matrix.get(MatrixIndex { y: index, x: self.column_index })
  }

  pub fn len(&self) -> usize {
    self.matrix.height()
  }

  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }

  pub fn iter(&self) -> impl Iterator<Item = &T> + '_ {
    self.matrix.body.iter().map(|row| &row[self.column_index])
  }

  pub fn to_owned(&self) -> Vec<T::Owned>
  where T: ToOwned {
    self.iter().map(T::to_owned).collect()
  }
}

impl<'a, T> ColumnMut<'a, T> {
  pub fn get(&self, index: usize) -> Option<&T> {
    self.matrix.get(MatrixIndex { y: index, x: self.column_index })
  }

  pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
    self.matrix.get_mut(MatrixIndex { y: index, x: self.column_index })
  }

  pub fn len(&self) -> usize {
    self.matrix.height()
  }

  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }

  pub fn iter(&self) -> impl Iterator<Item = &T> + '_ {
    self.matrix.body.iter().map(|row| &row[self.column_index])
  }

  pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> + '_ {
    self.matrix.body.iter_mut().map(|row| &mut row[self.column_index])
  }

  pub fn to_owned(&self) -> Vec<T::Owned>
  where T: ToOwned {
    self.iter().map(T::to_owned).collect()
  }
}

impl<'a, T> Index<usize> for Column<'a, T> {
  type Output = T;

  fn index(&self, index: usize) -> &Self::Output {
    &self.matrix[MatrixIndex { y: index, x: self.column_index }]
  }
}

impl<'a, T> Index<usize> for ColumnMut<'a, T> {
  type Output = T;

  fn index(&self, index: usize) -> &Self::Output {
    &self.matrix[MatrixIndex { y: index, x: self.column_index }]
  }
}

impl<'a, T> IndexMut<usize> for ColumnMut<'a, T> {
  fn index_mut(&mut self, index: usize) -> &mut Self::Output {
    &mut self.matrix[MatrixIndex { y: index, x: self.column_index }]
  }
}
