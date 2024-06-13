
pub mod broadcasting;

use super::Expr;
use crate::util::prism::Prism;

use thiserror::Error;

use std::ops::{Index, IndexMut};

/// A `Vector` is simply a `Vec<Expr>` but with added functionality
/// for mathematical operations typical of vectors.
///
/// A `Vector` is often represented in the expression language as a
/// call to the function called "vector", which is treated specially
/// by many parts of the calculation engine.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Vector {
  data: Vec<Expr>,
}

/// Prism which accepts only vectors (i.e. expressions which are calls
/// to a function called "vector").
///
/// Delegates to [`Vector::parse`] for narrowing.
#[derive(Debug, Clone, Copy, Default)]
pub struct ExprToVector;

#[derive(Debug, Error)]
#[error("Expected a vector, got {original_expr}")]
pub struct ParseVectorError {
  pub original_expr: Expr,
  _priv: (),
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("Length mismatch: expected {expected} but found {actual}")]
pub struct LengthError {
  expected: usize,
  actual: usize,
}

impl Vector {
  pub const FUNCTION_NAME: &'static str = "vector";

  /// If the expression is a function call of the form `vector(...)`,
  /// returns a [`Vector`] object representing the vector arguments.
  /// If the expression is of any other form, returns an appropriate
  /// error.
  pub fn parse(expr: Expr) -> Result<Vector, ParseVectorError> {
    if let Expr::Call(name, args) = expr {
      if name == Vector::FUNCTION_NAME {
        Ok(Vector { data: args })
      } else {
        Err(ParseVectorError {
          original_expr: Expr::Call(name, args),
          _priv: (),
        })
      }
    } else {
      Err(ParseVectorError {
        original_expr: expr,
        _priv: (),
      })
    }
  }

  pub fn zip_with<F>(self, other: Vector, mut f: F) -> Result<Vector, LengthError>
  where F: FnMut(Expr, Expr) -> Expr {
    if self.len() != other.len() {
      return Err(LengthError {
        expected: self.len(),
        actual: other.len(),
      });
    }
    Ok(
      self.into_iter()
        .zip(other)
        .map(|(a, b)| f(a, b))
        .collect()
    )
  }

  pub fn map<F>(self, f: F) -> Vector
  where F: FnMut(Expr) -> Expr {
    self.data.into_iter().map(f).collect()
  }

  /// Reifies the vector as an `Expr` in the expression language.
  /// Equivalent to `Expr::from(self)`.
  pub fn into_expr(self) -> Expr {
    Expr::call(Vector::FUNCTION_NAME, self.data)
  }

  pub fn iter(&self) -> std::slice::Iter<'_, Expr> {
    self.data.iter()
  }

  pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Expr> {
    self.data.iter_mut()
  }

  pub fn get(&self, i: usize) -> Option<&Expr> {
    self.data.get(i)
  }

  pub fn get_mut(&mut self, i: usize) -> Option<&mut Expr> {
    self.data.get_mut(i)
  }

  pub fn as_slice(&self) -> &[Expr] {
    &self.data
  }

  pub fn as_mut_slice(&mut self) -> &mut [Expr] {
    &mut self.data
  }

  pub fn len(&self) -> usize {
    self.data.len()
  }

  pub fn is_empty(&self) -> bool {
    self.data.is_empty()
  }
}

impl Prism<Expr, Vector> for ExprToVector {
  fn narrow_type(&self, input: Expr) -> Result<Vector, Expr> {
    Vector::parse(input).map_err(|err| err.original_expr)
  }
  fn widen_type(&self, input: Vector) -> Expr {
    input.into_expr()
  }
}

impl IntoIterator for Vector {
  type Item = Expr;
  type IntoIter = std::vec::IntoIter<Expr>;

  fn into_iter(self) -> Self::IntoIter {
    self.data.into_iter()
  }
}

impl FromIterator<Expr> for Vector {
  fn from_iter<I: IntoIterator<Item = Expr>>(iter: I) -> Self {
    Self {
      data: iter.into_iter().collect(),
    }
  }
}

impl From<Vec<Expr>> for Vector {
  fn from(data: Vec<Expr>) -> Self {
    Self { data }
  }
}

impl From<Vector> for Vec<Expr> {
  fn from(v: Vector) -> Self {
    v.data
  }
}

impl From<Vector> for Expr {
  fn from(v: Vector) -> Self {
    v.into_expr()
  }
}

impl Index<usize> for Vector {
  type Output = Expr;

  fn index(&self, i: usize) -> &Expr {
    &self.data[i]
  }
}

impl IndexMut<usize> for Vector {
  fn index_mut(&mut self, i: usize) -> &mut Expr {
    &mut self.data[i]
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_success() {
    let expr = Expr::call("vector", vec![Expr::from(1), Expr::from(2), Expr::from(3)]);
    let vec = Vector::parse(expr).unwrap();
    assert_eq!(vec, Vector::from(vec![Expr::from(1), Expr::from(2), Expr::from(3)]));
  }

  #[test]
  fn test_parse_success_on_empty_vec() {
    let expr = Expr::call("vector", vec![]);
    let vec = Vector::parse(expr).unwrap();
    assert_eq!(vec, Vector::default());
  }

  #[test]
  fn test_parse_failure_on_atom() {
    let expr = Expr::from(10);
    let err = Vector::parse(expr).unwrap_err();
    assert_eq!(err.original_expr, Expr::from(10));
  }

  #[test]
  fn test_parse_failure_on_call() {
    let expr = Expr::call("xyz", vec![Expr::from(1)]);
    let err = Vector::parse(expr).unwrap_err();
    assert_eq!(err.original_expr, Expr::call("xyz", vec![Expr::from(1)]));
  }

  #[test]
  fn test_iter() {
    let vec = Vector::from(vec![Expr::from(1), Expr::from(2), Expr::from(3)]);
    assert_eq!(vec.iter().collect::<Vec<_>>(), vec![&Expr::from(1), &Expr::from(2), &Expr::from(3)]);
  }

  #[test]
  fn test_prism_widen() {
    let vec = Vector::from(vec![Expr::from(1), Expr::from(2), Expr::from(3)]);
    let expr = ExprToVector.widen_type(vec);
    assert_eq!(expr, Expr::call("vector", vec![Expr::from(1), Expr::from(2), Expr::from(3)]));
  }

  #[test]
  fn test_prism_narrow_success() {
    let expr = Expr::call("vector", vec![Expr::from(1), Expr::from(2), Expr::from(3)]);
    let vec = ExprToVector.narrow_type(expr).unwrap();
    assert_eq!(vec, Vector::from(vec![Expr::from(1), Expr::from(2), Expr::from(3)]));
  }

  #[test]
  fn test_prism_narrow_failure() {
    let expr = Expr::from(10);
    let err = ExprToVector.narrow_type(expr).unwrap_err();
    assert_eq!(err, Expr::from(10));
  }

  #[test]
  fn test_map() {
    let vec = Vector::from(vec![Expr::from(1), Expr::from(2), Expr::from(3)]);
    let mapped = vec.map(|e| Expr::call("foo", vec![e]));
    assert_eq!(mapped, Vector::from(vec![
      Expr::call("foo", vec![Expr::from(1)]),
      Expr::call("foo", vec![Expr::from(2)]),
      Expr::call("foo", vec![Expr::from(3)]),
    ]));
  }

  #[test]
  fn test_zip_with() {
    let vec1 = Vector::from(vec![Expr::from(1), Expr::from(2), Expr::from(3)]);
    let vec2 = Vector::from(vec![Expr::from(10), Expr::from(20), Expr::from(30)]);
    let zipped = vec1.zip_with(vec2, |a, b| Expr::call("add", vec![a, b])).unwrap();
    assert_eq!(zipped, Vector::from(vec![
      Expr::call("add", vec![Expr::from(1), Expr::from(10)]),
      Expr::call("add", vec![Expr::from(2), Expr::from(20)]),
      Expr::call("add", vec![Expr::from(3), Expr::from(30)]),
    ]));
  }

  #[test]
  fn test_zip_with_on_different_size_vectors() {
    let vec1 = Vector::from(vec![Expr::from(1), Expr::from(2), Expr::from(3)]);
    let vec2 = Vector::from(vec![Expr::from(10), Expr::from(20)]);
    let err = vec1.zip_with(vec2, |a, b| Expr::call("add", vec![a, b])).unwrap_err();
    assert_eq!(err, LengthError { expected: 3, actual: 2 });
  }
}
