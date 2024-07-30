
use crate::expr::Expr;
use crate::util::matrix::{Matrix as UtilMatrix, MatrixIndex, MatrixDimsError};
use super::Vector;

/// A `Matrix` is a vector of vectors of expressions, where each
/// subvector has the same length.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Matrix {
  data: UtilMatrix<Expr>,
}

impl Matrix {
  pub fn new(body: Vec<Vec<Expr>>) -> Result<Matrix, MatrixDimsError<Expr>> {
    Ok(Matrix {
      data: UtilMatrix::new(body)?,
    })
  }

  pub fn from_generator<F>(height: usize, width: usize, generator: F) -> Self
  where F: FnMut(MatrixIndex) -> Expr {
    Matrix::from(UtilMatrix::from_generator(height, width, generator))
  }

  pub fn diagonal(elements: Vec<Expr>) -> Self {
    // This algorithm takes advantage of the fact that
    // `from_generator` calls its function in row-major order, so we
    // know we're generating the diagonals of the matrix in order.
    // Thus, we can just pull from the iterator each time we need a
    // value.
    let len = elements.len();
    let mut iter = elements.into_iter();
    Self::from_generator(len, len, move |index| {
      if index.x == index.y {
        // unwrap: We know we have `len` elements in the iterator.
        iter.next().unwrap()
      } else {
        Expr::zero()
      }
    })
  }

  pub fn of_value(height: usize, width: usize, value: Expr) -> Self {
    Matrix::from(UtilMatrix::of_value(height, width, value))
  }

  pub fn empty() -> Self {
    Matrix::from(UtilMatrix::empty())
  }

  pub fn into_row_major(self) -> Vec<Vec<Expr>> {
    self.data.into_row_major()
  }

  pub fn row(&self, index: usize) -> Option<&[Expr]> {
    self.data.row(index)
  }

  pub fn get(&self, index: MatrixIndex) -> Option<&Expr> {
    self.data.get(index)
  }

  pub fn get_mut(&mut self, index: MatrixIndex) -> Option<&mut Expr> {
    self.data.get_mut(index)
  }

  pub fn rows(&self) -> impl Iterator<Item = &[Expr]> + '_ {
    self.data.rows()
  }

  pub fn items(&self) -> impl Iterator<Item = &Expr> + '_ {
    self.data.items()
  }

  pub fn into_items(self) -> impl Iterator<Item = Expr> {
    self.data.into_items()
  }

  pub fn width(&self) -> usize {
    self.data.width()
  }

  pub fn height(&self) -> usize {
    self.data.height()
  }
}

impl From<UtilMatrix<Expr>> for Matrix {
  fn from(data: UtilMatrix<Expr>) -> Self {
    Matrix { data }
  }
}

impl TryFrom<Vec<Vec<Expr>>> for Matrix {
  type Error = MatrixDimsError<Expr>;

  fn try_from(body: Vec<Vec<Expr>>) -> Result<Self, Self::Error> {
    Self::new(body)
  }
}

impl From<Matrix> for Expr {
  fn from(matrix: Matrix) -> Expr {
    let outer_vector: Vector = matrix.into_row_major()
      .into_iter()
      .map(|row| Expr::from(Vector::from(row)))
      .collect();
    Expr::from(outer_vector)
  }
}
