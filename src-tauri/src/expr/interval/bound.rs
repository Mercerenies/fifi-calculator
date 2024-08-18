
use crate::util::brackets::HtmlBracketsType;

use std::cmp::{Ordering, min};
use std::ops::{Add, Sub, Mul};

/// An interval bound together with its bound type.
///
/// Binary arithmetic operations on bounded numbers always take the
/// stricter bound of the two arguments.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bounded<T> {
  pub scalar: T,
  pub bound_type: BoundType,
}

/// Whether or not a bound is inclusive.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BoundType {
  Exclusive,
  Inclusive,
}

impl<T> Bounded<T> {
  pub fn new(scalar: T, bound_type: BoundType) -> Self {
    Self { scalar, bound_type }
  }

  pub fn bound_type(&self) -> BoundType {
    self.bound_type
  }

  pub fn scalar(&self) -> &T {
    &self.scalar
  }

  pub fn into_scalar(self) -> T {
    self.scalar
  }

  pub fn map<F, U>(self, f: F) -> Bounded<U>
  where F: FnOnce(T) -> U {
    Bounded {
      scalar: f(self.scalar),
      bound_type: self.bound_type,
    }
  }

  pub fn apply<F, S, U>(self, other: Bounded<S>, f: F) -> Bounded<U>
  where F: FnOnce(T, S) -> U {
    Bounded {
      scalar: f(self.scalar, other.scalar),
      bound_type: min(self.bound_type, other.bound_type), // Take the *stricter* bound
    }
  }

  pub fn apply_err<F, S, U, E>(self, other: Bounded<S>, f: F) -> Result<Bounded<U>, E>
  where F: FnOnce(T, S) -> Result<U, E> {
    Ok(
      Bounded {
        scalar: f(self.scalar, other.scalar)?,
        bound_type: min(self.bound_type, other.bound_type), // Take the *stricter* bound
      },
    )
  }

  pub fn min(self, other: Bounded<T>) -> Bounded<T> where T: Ord {
    match self.scalar.cmp(&other.scalar) {
      Ordering::Greater => other,
      Ordering::Less => self,
      Ordering::Equal => Bounded::new(self.scalar, self.bound_type.max(other.bound_type)),
    }
  }

  pub fn max(self, other: Bounded<T>) -> Bounded<T> where T: Ord {
    match self.scalar.cmp(&other.scalar) {
      Ordering::Greater => self,
      Ordering::Less => other,
      Ordering::Equal => Bounded::new(self.scalar, self.bound_type.max(other.bound_type)),
    }
  }
}

impl BoundType {
  pub fn html_bracket_type(self) -> HtmlBracketsType {
    match self {
      BoundType::Inclusive => HtmlBracketsType::SquareBrackets,
      BoundType::Exclusive => HtmlBracketsType::Parentheses,
    }
  }
}

impl<T: Add> Add for Bounded<T> {
  type Output = Bounded<T::Output>;

  fn add(self, other: Self) -> Bounded<T::Output> {
    self.apply(other, |x, y| x + y)
  }
}

impl<T: Sub> Sub for Bounded<T> {
  type Output = Bounded<T::Output>;

  fn sub(self, other: Self) -> Bounded<T::Output> {
    self.apply(other, |x, y| x - y)
  }
}

impl<T: Mul> Mul for Bounded<T> {
  type Output = Bounded<T::Output>;

  fn mul(self, other: Self) -> Bounded<T::Output> {
    self.apply(other, |x, y| x * y)
  }
}
