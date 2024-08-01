
use crate::util::prism::{Prism, PrismExt, ErrorWithPayload, Identity};
use crate::expr::Expr;
use crate::expr::prisms::expr_to_typed_vector;
use crate::util::matrix::{Matrix as UtilMatrix, MatrixIndex, MatrixDimsError, Column, ColumnMut};
use super::Vector;

/// A `Matrix` is a vector of vectors of expressions, where each
/// subvector has the same length.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Matrix {
  data: UtilMatrix<Expr>,
}

/// Prism which attempts to parse an expression as a matrix.
/// Specifically, this prism accepts vectors of vectors where each
/// subvector has the same length and the elements of the subvectors
/// satisfy the inner prism.
///
/// NOTE: This prism is NOT mutually exclusive with
/// [`ExprToTensor`](super::tensor::ExprToTensor) or [`ExprToVector`]
/// and should not be used in a disjunction with either of those.
#[derive(Debug, Clone)]
pub struct ExprToTypedMatrix<P> {
  element_prism: P,
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

  pub fn transpose(self) -> Self {
    Self { data: self.data.transpose() }
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

  pub fn row_mut(&mut self, index: usize) -> Option<&mut [Expr]> {
    self.data.row_mut(index)
  }

  pub fn column(&self, index: usize) -> Option<Column<'_, Expr>> {
    self.data.column(index)
  }

  pub fn column_mut(&mut self, index: usize) -> Option<ColumnMut<'_, Expr>> {
    self.data.column_mut(index)
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

  pub fn as_matrix(&self) -> &UtilMatrix<Expr> {
    &self.data
  }

  pub fn as_matrix_mut(&mut self) -> &mut UtilMatrix<Expr> {
    &mut self.data
  }

  pub fn into_matrix(self) -> UtilMatrix<Expr> {
    self.data
  }
}

pub fn expr_to_matrix() -> impl Prism<Expr, Matrix> + Clone {
  ExprToTypedMatrix::new(Identity).rmap(Matrix::from, Matrix::into_matrix)
}

impl<P> ExprToTypedMatrix<P> {
  fn new(element_prism: P) -> Self {
    ExprToTypedMatrix { element_prism }
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

impl<P, T> Prism<Expr, UtilMatrix<T>> for ExprToTypedMatrix<P>
where P: Prism<Expr, T> + Clone {
  fn narrow_type(&self, expr: Expr) -> Result<UtilMatrix<T>, Expr> {
    let inner_prism = expr_to_typed_vector(expr_to_typed_vector(self.element_prism.clone()));
    let vec_of_vecs = inner_prism.narrow_type(expr)?;
    UtilMatrix::new(vec_of_vecs).map_err(|err| {
      inner_prism.widen_type(err.recover_payload())
    })
  }

  fn widen_type(&self, matrix: UtilMatrix<T>) -> Expr {
    let inner_prism = expr_to_typed_vector(expr_to_typed_vector(self.element_prism.clone()));
    let vec_of_vecs = matrix.into_row_major();
    inner_prism.widen_type(vec_of_vecs)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_prism_widen_type() {
    let matrix = Matrix::from_generator(2, 2, |index| Expr::from((index.x * 10 + index.y) as i64));
    let expr = expr_to_matrix().widen_type(matrix);
    assert_eq!(expr, Expr::call("vector", vec![
      Expr::call("vector", vec![Expr::from(0), Expr::from(10)]),
      Expr::call("vector", vec![Expr::from(1), Expr::from(11)]),
    ]));

    let matrix = Matrix::from_generator(2, 1, |index| Expr::from((index.x * 10 + index.y) as i64));
    let expr = expr_to_matrix().widen_type(matrix);
    assert_eq!(expr, Expr::call("vector", vec![
      Expr::call("vector", vec![Expr::from(0)]),
      Expr::call("vector", vec![Expr::from(1)]),
    ]));

    let matrix = Matrix::from_generator(1, 2, |index| Expr::from((index.x * 10 + index.y) as i64));
    let expr = expr_to_matrix().widen_type(matrix);
    assert_eq!(expr, Expr::call("vector", vec![
      Expr::call("vector", vec![Expr::from(0), Expr::from(10)]),
    ]));
  }

  #[test]
  fn test_prism_widen_type_corner_cases() {
    let matrix = Matrix::from_generator(0, 0, |_| unreachable!());
    assert_eq!(expr_to_matrix().widen_type(matrix), Expr::call("vector", vec![]));

    let matrix = Matrix::from_generator(0, 99, |_| unreachable!());
    assert_eq!(expr_to_matrix().widen_type(matrix), Expr::call("vector", vec![]));

    let matrix = Matrix::from_generator(2, 0, |_| unreachable!());
    assert_eq!(expr_to_matrix().widen_type(matrix), Expr::call("vector", vec![
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
    let matrix = expr_to_matrix().narrow_type(expr).unwrap();
    assert_eq!(matrix, Matrix::from_generator(2, 2, |index| Expr::from((index.x * 10 + index.y) as i64)));
  }

  #[test]
  fn test_prism_narrow_type_non_square() {
    let expr = Expr::call("vector", vec![
      Expr::call("vector", vec![Expr::from(0), Expr::from(10), Expr::from(20)]),
      Expr::call("vector", vec![Expr::from(1), Expr::from(11), Expr::from(21)]),
    ]);
    let matrix = expr_to_matrix().narrow_type(expr).unwrap();
    assert_eq!(matrix, Matrix::from_generator(2, 3, |index| Expr::from((index.x * 10 + index.y) as i64)));
  }

  #[test]
  fn test_prism_narrow_type_corner_cases() {
    let expr = Expr::call("vector", vec![]);
    assert_eq!(expr_to_matrix().narrow_type(expr).unwrap(), Matrix::default());

    let expr = Expr::call("vector", vec![Expr::call("vector", vec![])]);
    assert_eq!(expr_to_matrix().narrow_type(expr).unwrap(), Matrix::from_generator(1, 0, |_| unreachable!()));
  }

  #[test]
  fn test_prism_narrow_type_failure() {
    expr_to_matrix().narrow_type(Expr::from(0)).unwrap_err();
    expr_to_matrix().narrow_type(Expr::call("*", vec![])).unwrap_err();
    expr_to_matrix().narrow_type(Expr::call("vector", vec![Expr::from(0)])).unwrap_err();
    expr_to_matrix().narrow_type(Expr::call("vector", vec![
      Expr::call("vector", vec![Expr::from(0), Expr::from(10), Expr::from(20)]),
      Expr::call("vector", vec![Expr::from(9)]),
    ])).unwrap_err();
  }
}
