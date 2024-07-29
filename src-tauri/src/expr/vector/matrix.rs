
use crate::expr::Expr;
use crate::util::matrix::{Matrix as UtilMatrix, MatrixIndex, MatrixDimsError};

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
