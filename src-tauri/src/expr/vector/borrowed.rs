
use super::Vector;
use crate::expr::Expr;

use thiserror::Error;

use std::ops::Index;

/// Like [`Vector`], but immutably borrows its values rather than
/// requiring exclusive ownership.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BorrowedVector<'a> {
  data: &'a [Expr],
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("Expected a vector, got {original_expr}")]
pub struct ParseBorrowedVectorError<'a> {
  pub original_expr: &'a Expr,
  _priv: (),
}

impl<'a> BorrowedVector<'a> {
  pub fn empty() -> Self {
    Self::default()
  }

  pub fn parse(expr: &'a Expr) -> Result<Self, ParseBorrowedVectorError<'a>> {
    if let Expr::Call(name, args) = expr {
      if name == Vector::FUNCTION_NAME {
        return Ok(BorrowedVector { data: args })
      }
    }
    Err(ParseBorrowedVectorError {
      original_expr: expr,
      _priv: (),
    })
  }

  pub fn iter(&self) -> std::slice::Iter<'_, Expr> {
    self.data.iter()
  }

  pub fn get(&self, i: usize) -> Option<&Expr> {
    self.data.get(i)
  }

  pub fn as_slice(&self) -> &[Expr] {
    self.data
  }

  pub fn len(&self) -> usize {
    self.data.len()
  }

  pub fn is_empty(&self) -> bool {
    self.data.is_empty()
  }

  pub fn to_owned(&self) -> Vector {
    Vector::from(self.data.to_owned())
  }
}

impl<'a> IntoIterator for BorrowedVector<'a> {
  type Item = &'a Expr;
  type IntoIter = std::slice::Iter<'a, Expr>;

  fn into_iter(self) -> Self::IntoIter {
    self.data.iter()
  }
}

impl Index<usize> for BorrowedVector<'_> {
  type Output = Expr;

  fn index(&self, i: usize) -> &Self::Output {
    &self.data[i]
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_vector() {
    let expr = Expr::call("vector", vec![Expr::from(10), Expr::from(20), Expr::from(30)]);
    let vec = BorrowedVector::parse(&expr).unwrap();
    assert_eq!(vec.as_slice(), &[Expr::from(10), Expr::from(20), Expr::from(30)]);
  }

  #[test]
  fn test_accessors() {
    let elems = [Expr::from(10), Expr::from(20), Expr::from(30)];
    let vec = BorrowedVector { data: &elems };

    assert_eq!(&vec[0], &Expr::from(10));
    assert_eq!(&vec[1], &Expr::from(20));
    assert_eq!(&vec[2], &Expr::from(30));

    assert_eq!(vec.get(0), Some(&Expr::from(10)));
    assert_eq!(vec.get(1), Some(&Expr::from(20)));
    assert_eq!(vec.get(2), Some(&Expr::from(30)));
    assert_eq!(vec.get(3), None);
  }

  #[test]
  fn test_iter() {
    let elems = [Expr::from(10), Expr::from(20), Expr::from(30)];
    let vec = BorrowedVector { data: &elems };

    let final_elems = vec.iter().cloned().collect::<Vec<_>>();
    assert_eq!(final_elems, elems);
  }

  #[test]
  fn test_into_iter() {
    let elems = [Expr::from(10), Expr::from(20), Expr::from(30)];
    let vec = BorrowedVector { data: &elems };

    let final_elems = vec.into_iter().cloned().collect::<Vec<_>>();
    assert_eq!(final_elems, elems);
  }
}
