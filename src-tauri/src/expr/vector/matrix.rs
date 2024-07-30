
use crate::util::prism::{Prism, ErrorWithPayload, Identity};
use crate::expr::Expr;
use crate::expr::prisms::expr_to_typed_vector;
use crate::util::matrix::{Matrix as UtilMatrix, MatrixIndex, MatrixDimsError};
use super::Vector;

/// A `Matrix` is a vector of vectors of expressions, where each
/// subvector has the same length.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Matrix {
  data: UtilMatrix<Expr>,
}

/// Prism which attempts to parse an expression as a matrix.
/// Specifically, this prism accepts vectors of vectors where each
/// subvector has the same length.
///
/// NOTE: This prism is NOT mutually exclusive with
/// [`ExprToTensor`](super::tensor::ExprToTensor) or [`ExprToVector`]
/// and should not be used in a disjunction with either of those.
#[derive(Debug, Clone)]
pub struct ExprToMatrix;

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

impl ExprToMatrix {
  fn inner_prism() -> impl Prism<Expr, Vec<Vec<Expr>>> {
    expr_to_typed_vector(expr_to_typed_vector(Identity))
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

impl Prism<Expr, Matrix> for ExprToMatrix {
  fn narrow_type(&self, expr: Expr) -> Result<Matrix, Expr> {
    let inner_prism = Self::inner_prism();
    let vec_of_vecs = inner_prism.narrow_type(expr)?;
    Matrix::new(vec_of_vecs).map_err(|err| {
      inner_prism.widen_type(err.recover_payload())
    })
  }

  fn widen_type(&self, matrix: Matrix) -> Expr {
    matrix.into()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_prism_widen_type() {
    let matrix = Matrix::from_generator(2, 2, |index| Expr::from((index.x * 10 + index.y) as i64));
    let expr = ExprToMatrix.widen_type(matrix);
    assert_eq!(expr, Expr::call("vector", vec![
      Expr::call("vector", vec![Expr::from(0), Expr::from(10)]),
      Expr::call("vector", vec![Expr::from(1), Expr::from(11)]),
    ]));

    let matrix = Matrix::from_generator(2, 1, |index| Expr::from((index.x * 10 + index.y) as i64));
    let expr = ExprToMatrix.widen_type(matrix);
    assert_eq!(expr, Expr::call("vector", vec![
      Expr::call("vector", vec![Expr::from(0)]),
      Expr::call("vector", vec![Expr::from(1)]),
    ]));

    let matrix = Matrix::from_generator(1, 2, |index| Expr::from((index.x * 10 + index.y) as i64));
    let expr = ExprToMatrix.widen_type(matrix);
    assert_eq!(expr, Expr::call("vector", vec![
      Expr::call("vector", vec![Expr::from(0), Expr::from(10)]),
    ]));
  }

  #[test]
  fn test_prism_widen_type_corner_cases() {
    let matrix = Matrix::from_generator(0, 0, |_| unreachable!());
    assert_eq!(ExprToMatrix.widen_type(matrix), Expr::call("vector", vec![]));

    let matrix = Matrix::from_generator(0, 99, |_| unreachable!());
    assert_eq!(ExprToMatrix.widen_type(matrix), Expr::call("vector", vec![]));

    let matrix = Matrix::from_generator(2, 0, |_| unreachable!());
    assert_eq!(ExprToMatrix.widen_type(matrix), Expr::call("vector", vec![
      Expr::call("vector", vec![]),
      Expr::call("vector", vec![]),
    ]));
  }

  #[test]
  fn test_prism_narrow_type_square() {
    let expr = Expr::call("vector", vec![
      Expr::call("vector", vec![Expr::from(0), Expr::from(10)]),
      Expr::call("vector", vec![Expr::from(1), Expr::from(11)]),
    ]);
    let matrix = ExprToMatrix.narrow_type(expr).unwrap();
    assert_eq!(matrix, Matrix::from_generator(2, 2, |index| Expr::from((index.x * 10 + index.y) as i64)));
  }

  #[test]
  fn test_prism_narrow_type_non_square() {
    let expr = Expr::call("vector", vec![
      Expr::call("vector", vec![Expr::from(0), Expr::from(10), Expr::from(20)]),
      Expr::call("vector", vec![Expr::from(1), Expr::from(11), Expr::from(21)]),
    ]);
    let matrix = ExprToMatrix.narrow_type(expr).unwrap();
    assert_eq!(matrix, Matrix::from_generator(2, 3, |index| Expr::from((index.x * 10 + index.y) as i64)));
  }

  #[test]
  fn test_prism_narrow_type_corner_cases() {
    let expr = Expr::call("vector", vec![]);
    assert_eq!(ExprToMatrix.narrow_type(expr).unwrap(), Matrix::default());

    let expr = Expr::call("vector", vec![Expr::call("vector", vec![])]);
    assert_eq!(ExprToMatrix.narrow_type(expr).unwrap(), Matrix::from_generator(1, 0, |_| unreachable!()));
  }

  #[test]
  fn test_prism_narrow_type_failure() {
    ExprToMatrix.narrow_type(Expr::from(0)).unwrap_err();
    ExprToMatrix.narrow_type(Expr::call("*", vec![])).unwrap_err();
    ExprToMatrix.narrow_type(Expr::call("vector", vec![Expr::from(0)])).unwrap_err();
    ExprToMatrix.narrow_type(Expr::call("vector", vec![
      Expr::call("vector", vec![Expr::from(0), Expr::from(10), Expr::from(20)]),
      Expr::call("vector", vec![Expr::from(9)]),
    ])).unwrap_err();
  }
}
