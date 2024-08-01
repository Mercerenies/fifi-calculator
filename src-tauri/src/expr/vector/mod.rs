
pub mod borrowed;
pub mod matrix;
pub mod tensor;

use matrix::Matrix;
use borrowed::BorrowedVector;
use super::Expr;
use super::number::Number;
use crate::util::uniq_element;
use crate::util::prism::Prism;

use thiserror::Error;

use std::ops::{Index, IndexMut};
use std::convert::TryFrom;

/// A `Vector` is simply a `Vec<Expr>` but with added functionality
/// for mathematical operations typical of vectors.
///
/// A `Vector` is represented in the expression language as a call to
/// the function called "vector", which is treated specially by many
/// parts of the calculation engine.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
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

pub fn vector_shape(expr: &Expr) -> Vec<usize> {
  let mut acc = Vec::new();
  vector_shape_impl([expr].into_iter(), &mut acc);
  acc
}

fn vector_shape_impl<'a, 'b>(exprs: impl Iterator<Item=&'a Expr>, acc: &'b mut Vec<usize>) {
  let Ok(vecs) = exprs.map(BorrowedVector::parse).collect::<Result<Vec<_>, _>>() else {
    // No more nested vectors, so stop recursing.
    return;
  };
  let Some(length) = uniq_element(vecs.iter().map(BorrowedVector::len)) else {
    // Inconsistent lengths, so don't treat it as a higher tensor.
    return;
  };
  acc.push(length);
  vector_shape_impl(vecs.into_iter().flatten(), acc);
}

impl Vector {
  pub const FUNCTION_NAME: &'static str = "vector";

  /// A new, empty `Vector`.
  pub fn empty() -> Self {
    Self { data: vec![] }
  }

  /// A new, empty vector with the given capacity.
  pub fn with_capacity(capacity: usize) -> Self {
    Self {
      data: Vec::with_capacity(capacity),
    }
  }

  pub fn append(mut self, mut other: Self) -> Self {
    self.data.append(&mut other.data);
    self
  }

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

  pub fn flatten_all_nested(self) -> Self {
    fn flatten_expr(expr: Expr) -> Vec<Expr> {
      match ExprToVector.narrow_type(expr) {
        Err(expr) => vec![expr],
        Ok(vec) => vec.flatten_all_nested().into(),
      }
    }

    self.into_iter()
      .flat_map(flatten_expr)
      .collect()
  }

  pub fn into_row_vector(self) -> Matrix {
    Matrix::new(vec![Vec::from(self)]).unwrap()
  }

  pub fn into_column_vector(self) -> Matrix {
    let elems = self.into_iter().map(|x| vec![x]).collect();
    Matrix::new(elems).unwrap()
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

  pub fn as_mut_vec(&mut self) -> &mut Vec<Expr> {
    &mut self.data
  }

  pub fn len(&self) -> usize {
    self.data.len()
  }

  pub fn is_empty(&self) -> bool {
    self.data.is_empty()
  }

  /// Produces an expression which computes the k-norm of the vector.
  pub fn norm(self, k: usize) -> Expr {
    fn abs(x: Expr) -> Expr {
      Expr::call("abs", vec![x])
    }
    fn pow(x: Expr, numer: usize, denom: usize) -> Expr {
      if numer == 1 && denom == 1 {
        x
      } else {
        Expr::call("^", vec![
          x,
          Expr::from(Number::ratio(numer, denom)),
        ])
      }
    }

    assert!(k > 0, "k-norm must be positive");
    if self.is_empty() {
      return Expr::zero();
    }
    let addends = self.into_iter().map(|x| pow(abs(x), k, 1)).collect();
    pow(Expr::call("+", addends), 1, k)
  }

  /// Produces and expression to compute the infinity-norm of the
  /// vector.
  pub fn infinity_norm(self) -> Expr {
    fn abs(x: Expr) -> Expr {
      Expr::call("abs", vec![x])
    }

    if self.is_empty() {
      return Expr::zero();
    }
    let args = self.into_iter().map(abs).collect();
    Expr::call("max", args)
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

impl<const C: usize> TryFrom<Vector> for [Expr; C] {
  type Error = <Vec<Expr> as TryInto<[Expr; C]>>::Error;

  fn try_from(v: Vector) -> Result<Self, Self::Error> {
    v.data.try_into()
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

  #[test]
  fn test_vector_shape_of_scalar() {
    let expr = Expr::from(9);
    assert_eq!(vector_shape(&expr), Vec::<usize>::new());
  }

  #[test]
  fn test_vector_shape_of_simple_vector() {
    let expr = Expr::call("vector", vec![Expr::from(10), Expr::from(20), Expr::from(30)]);
    assert_eq!(vector_shape(&expr), vec![3]);
  }

  #[test]
  fn test_vector_shape_of_matrix() {
    let expr = Expr::call("vector", vec![
      Expr::call("vector", vec![Expr::from(1), Expr::from(2)]),
      Expr::call("vector", vec![Expr::from(3), Expr::from(4)]),
      Expr::call("vector", vec![Expr::from(5), Expr::from(6)]),
    ]);
    assert_eq!(vector_shape(&expr), vec![3, 2]);
  }

  #[test]
  fn test_vector_shape_of_jagged_vector_of_vectors() {
    let expr = Expr::call("vector", vec![
      Expr::call("vector", vec![Expr::from(1), Expr::from(2)]),
      Expr::call("vector", vec![Expr::from(3), Expr::from(4), Expr::from(5)]),
      Expr::call("vector", vec![Expr::from(6), Expr::from(7)]),
    ]);
    assert_eq!(vector_shape(&expr), vec![3]);
  }

  #[test]
  fn test_flatten_all_nested() {
    let expr = Expr::call("vector", vec![
      Expr::call("vector", vec![Expr::from(1), Expr::from(2)]),
      Expr::call("vector", vec![Expr::from(3), Expr::from(4), Expr::from(5)]),
      Expr::call("vector", vec![Expr::from(6), Expr::from(7)]),
    ]);
    let vector = Vector::parse(expr).unwrap();
    assert_eq!(vector.flatten_all_nested(), Vector::from(vec![
      Expr::from(1), Expr::from(2), Expr::from(3),
      Expr::from(4), Expr::from(5), Expr::from(6),
      Expr::from(7),
    ]));
  }

  #[test]
  fn test_flatten_all_nested_empty() {
    let vector = Vector::empty();
    assert_eq!(vector.flatten_all_nested(), Vector::empty());
  }

  #[test]
  fn test_flatten_all_nested_of_different_depths() {
    let expr = Expr::call("vector", vec![
      Expr::call("vector", vec![Expr::from(1), Expr::from(2)]),
      Expr::call("vector", vec![
        Expr::from(3),
        Expr::call("vector", vec![Expr::from(4)]),
        Expr::from(5),
      ]),
      Expr::from(6),
      Expr::from(7),
    ]);
    let vector = Vector::parse(expr).unwrap();
    assert_eq!(vector.flatten_all_nested(), Vector::from(vec![
      Expr::from(1), Expr::from(2), Expr::from(3),
      Expr::from(4), Expr::from(5), Expr::from(6),
      Expr::from(7),
    ]));
  }

  #[test]
  fn test_norm() {
    let vector = Vector::from(vec![Expr::var("x").unwrap(), Expr::var("y").unwrap(), Expr::var("z").unwrap()]);
    assert_eq!(vector.clone().norm(1), Expr::call("+", vec![
      Expr::call("abs", vec![Expr::var("x").unwrap()]),
      Expr::call("abs", vec![Expr::var("y").unwrap()]),
      Expr::call("abs", vec![Expr::var("z").unwrap()]),
    ]));
    assert_eq!(vector.clone().norm(2), Expr::call("^", vec![
      Expr::call("+", vec![
        Expr::call("^", vec![Expr::call("abs", vec![Expr::var("x").unwrap()]), Expr::from(2)]),
        Expr::call("^", vec![Expr::call("abs", vec![Expr::var("y").unwrap()]), Expr::from(2)]),
        Expr::call("^", vec![Expr::call("abs", vec![Expr::var("z").unwrap()]), Expr::from(2)]),
      ]),
      Expr::from(Number::ratio(1, 2)),
    ]));
    assert_eq!(vector.norm(3), Expr::call("^", vec![
      Expr::call("+", vec![
        Expr::call("^", vec![Expr::call("abs", vec![Expr::var("x").unwrap()]), Expr::from(3)]),
        Expr::call("^", vec![Expr::call("abs", vec![Expr::var("y").unwrap()]), Expr::from(3)]),
        Expr::call("^", vec![Expr::call("abs", vec![Expr::var("z").unwrap()]), Expr::from(3)]),
      ]),
      Expr::from(Number::ratio(1, 3)),
    ]));
  }

  #[test]
  fn test_infinity_norm() {
    let vector = Vector::from(vec![Expr::var("x").unwrap(), Expr::var("y").unwrap(), Expr::var("z").unwrap()]);
    assert_eq!(vector.clone().infinity_norm(), Expr::call("max", vec![
      Expr::call("abs", vec![Expr::var("x").unwrap()]),
      Expr::call("abs", vec![Expr::var("y").unwrap()]),
      Expr::call("abs", vec![Expr::var("z").unwrap()]),
    ]));
  }

  #[test]
  fn test_norm_of_empty() {
    assert_eq!(Vector::default().norm(1), Expr::from(0));
    assert_eq!(Vector::default().norm(2), Expr::from(0));
    assert_eq!(Vector::default().norm(3), Expr::from(0));
    assert_eq!(Vector::default().infinity_norm(), Expr::from(0));
  }

  #[test]
  #[should_panic]
  fn test_norm_zero_panics() {
    Vector::default().norm(0);
  }
}
