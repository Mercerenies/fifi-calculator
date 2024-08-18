
use super::{Matrix, MatrixIndex};
use crate::expr::Expr;
use crate::expr::vector::Vector;
use crate::expr::vector::borrowed::{BorrowedVector, ParseBorrowedVectorError};

use thiserror::Error;

/// A [`Matrix`] which borrows its rows as slices, rather than owning
/// the entire data structure.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BorrowedMatrix<'a> {
  body: Vec<&'a [Expr]>,
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("Expected a matrix, got {original_expr}")]
pub struct ParseBorrowedMatrixError<'a> {
  pub original_expr: &'a Expr,
  _priv: (),
}

impl<'a> BorrowedMatrix<'a> {
  pub fn empty() -> Self {
    Self { body: Vec::new() }
  }

  pub fn is_empty(&self) -> bool {
    self.height() == 0 || self.width() == 0
  }

  pub fn parse(expr: &'a Expr) -> Result<Self, ParseBorrowedMatrixError<'a>> {
    let Expr::Call(name, args) = expr else {
      return Err(ParseBorrowedMatrixError { original_expr: expr, _priv: () });
    };
    if name != Vector::FUNCTION_NAME {
      return Err(ParseBorrowedMatrixError { original_expr: expr, _priv: () });
    }
    if args.is_empty() {
      return Ok(Self::empty());
    }

    let args = args.iter()
      .map(|arg| {
        let vec = BorrowedVector::parse(arg)?;
        Ok(vec.as_slice())
      })
      .collect::<Result<Vec<_>, ParseBorrowedVectorError<'a>>>()
      .map_err(|_| ParseBorrowedMatrixError { original_expr: expr, _priv: () })?;
    if args.iter().any(|row| row.len() != args[0].len()) {
      return Err(ParseBorrowedMatrixError { original_expr: expr, _priv: () });
    }
    Ok(BorrowedMatrix { body: args })
  }

  pub fn as_row_major<'b>(&'b self) -> &'b [&'a [Expr]] {
    &self.body
  }

  pub fn into_row_major(self) -> Vec<&'a [Expr]> {
    self.body
  }

  pub fn row(&self, index: usize) -> Option<&'a [Expr]> {
    self.body.get(index).copied()
  }

  pub fn get(&self, index: MatrixIndex) -> Option<&'a Expr> {
    let MatrixIndex { y, x } = index;
    self.row(y)
      .and_then(|row| row.get(x))
  }

  pub fn rows(&self) -> impl Iterator<Item = &'a [Expr]> + '_ {
    self.body.iter().copied()
  }

  pub fn items(&self) -> impl Iterator<Item = &'a Expr> + '_ {
    self.body.iter().copied().flatten()
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

  pub fn to_owned(&self) -> Matrix {
    let body: Vec<_> = self.body.iter().map(|row| (*row).to_owned()).collect();
    Matrix::new(body).expect("Matrix::new should always succeed on a valid matrix")
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_parse_vector_successful() {
    let expr = Expr::call("vector", vec![
      Expr::call("vector", vec![Expr::from(10), Expr::from(20), Expr::from(30)]),
      Expr::call("vector", vec![Expr::from(40), Expr::from(50), Expr::from(60)]),
    ]);
    let matrix = BorrowedMatrix::parse(&expr).unwrap();
    assert_eq!(
      matrix.to_owned(),
      Matrix::new(vec![
        vec![Expr::from(10), Expr::from(20), Expr::from(30)],
        vec![Expr::from(40), Expr::from(50), Expr::from(60)],
      ]).unwrap(),
    );
  }
  #[test]
  fn test_parse_empty_vector_successful() {
    let expr = Expr::call("vector", vec![]);
    let matrix = BorrowedMatrix::parse(&expr).unwrap();
    assert_eq!(matrix, BorrowedMatrix::empty());
  }


  #[test]
  fn test_parse_vector_failure() {
    let expr = Expr::call("vector", vec![Expr::from(10)]);
    assert!(BorrowedMatrix::parse(&expr).is_err());
    let expr = Expr::call("vector", vec![
      Expr::call("vector", vec![Expr::from(10), Expr::from(20), Expr::from(30)]),
      Expr::call("vector", vec![Expr::from(40), Expr::from(50)]),
    ]);
    assert!(BorrowedMatrix::parse(&expr).is_err());
  }

  #[test]
  fn test_row_accessors() {
    let expr = Expr::call("vector", vec![
      Expr::call("vector", vec![Expr::from(10), Expr::from(20), Expr::from(30)]),
      Expr::call("vector", vec![Expr::from(40), Expr::from(50), Expr::from(60)]),
    ]);
    let matrix = BorrowedMatrix::parse(&expr).unwrap();
    assert_eq!(matrix.row(0), Some([Expr::from(10), Expr::from(20), Expr::from(30)].as_slice()));
    assert_eq!(matrix.row(1), Some([Expr::from(40), Expr::from(50), Expr::from(60)].as_slice()));
    assert_eq!(matrix.row(2), None);
  }

  #[test]
  fn test_element_accessors() {
    let expr = Expr::call("vector", vec![
      Expr::call("vector", vec![Expr::from(10), Expr::from(20), Expr::from(30)]),
      Expr::call("vector", vec![Expr::from(40), Expr::from(50), Expr::from(60)]),
    ]);
    let matrix = BorrowedMatrix::parse(&expr).unwrap();
    assert_eq!(matrix.get(MatrixIndex { x: 0, y: 0 }), Some(&Expr::from(10)));
    assert_eq!(matrix.get(MatrixIndex { x: 1, y: 0 }), Some(&Expr::from(20)));
    assert_eq!(matrix.get(MatrixIndex { x: 2, y: 0 }), Some(&Expr::from(30)));
    assert_eq!(matrix.get(MatrixIndex { x: 3, y: 0 }), None);
    assert_eq!(matrix.get(MatrixIndex { x: 0, y: 1 }), Some(&Expr::from(40)));
    assert_eq!(matrix.get(MatrixIndex { x: 1, y: 1 }), Some(&Expr::from(50)));
    assert_eq!(matrix.get(MatrixIndex { x: 2, y: 1 }), Some(&Expr::from(60)));
    assert_eq!(matrix.get(MatrixIndex { x: 3, y: 1 }), None);
    assert_eq!(matrix.get(MatrixIndex { x: 0, y: 2 }), None);
  }
}
