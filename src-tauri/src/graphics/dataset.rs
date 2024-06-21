
//! Structures for generating points for a graph.

use crate::expr::Expr;
use crate::expr::number::Number;
use crate::expr::interval::Interval;
use crate::expr::prisms;
use crate::util::prism::{Prism, DisjPrism};

use thiserror::Error;
use either::Either;

/// A dataset of input (independent) values for some graph.
///
/// Datasets can be enumerated explicitly or provided implicitly as
/// either an interval (indicating a minimum and a maxmum) or a unit
/// step (indicating a starting value and an implicit step size of 1).
#[derive(Debug, Clone, PartialEq)]
pub struct XDataSet {
  data: XDataSetImpl,
}

#[derive(Debug, Clone, PartialEq)]
enum XDataSetImpl {
  Vector(Vec<Number>),
  Interval(Number, Number),
  Number(Number),
}

/// An [`Expr`] which can be reasonably interpreted as an
/// [`XDataSet`]. `XDataSet` implements `From<XDataSetExpr>`, and this
/// type is mainly used as the target for a prism, since
/// `XDataSetExpr` stores enough information to recover the original
/// expression while `XDataSet` does not.
#[derive(Debug, Clone, PartialEq)]
pub struct XDataSetExpr {
  data: Either<Vec<Number>, Either<Interval, Number>>,
}

/// Prism which attempts to parse an `Expr` as a `XDataSetExpr`.
///
/// Specifically, this prism accepts any of the following expression
/// types:
///
/// * Vectors of literal real numbers.
///
/// * Intervals of literal real numbers (with any interval type).
///
/// * Literal real numbers.
#[derive(Debug, Clone)]
pub struct ExprToXDataSet {
  _priv: (),
}

#[derive(Debug, Clone, PartialEq, Error)]
#[error("Data set length mismatch (expected {expected}, got {actual})")]
pub struct LengthError {
  expected: usize,
  actual: usize,
}

impl XDataSet {
  pub const STEP_DATA_POINTS: usize = 30;
  pub const INTERVAL_DATA_POINTS: usize = 200;

  /// An enumerated data set, consisting of exactly the indicated
  /// points.
  pub fn enumerated(vec: Vec<Number>) -> Self {
    Self { data: XDataSetImpl::Vector(vec) }
  }

  /// An interval data set, consisting of an arbitrary, unspecified
  /// number of points from `min` to `max`. Both bounds are inclusive,
  /// except in corner cases where the length of the requested data
  /// set is less than 2.
  ///
  /// Panics if `min > max`.
  pub fn interval(min: Number, max: Number) -> Self {
    assert!(min <= max, "Invalid interval ({min} .. {max})");
    Self { data: XDataSetImpl::Interval(min, max) }
  }

  /// A data set which starts at `starting` and increments by 1 for
  /// each subsequent point. Usually used to generate sets of integers
  /// for integer-valued functions.
  pub fn step_from(starting: Number) -> Self {
    Self { data: XDataSetImpl::Number(starting) }
  }

  pub fn required_len(&self) -> Option<usize> {
    match &self.data {
      XDataSetImpl::Vector(v) => Some(v.len()),
      _ => None,
    }
  }

  pub fn preferred_len(&self) -> usize {
    match &self.data {
      XDataSetImpl::Vector(v) => v.len(),
      XDataSetImpl::Interval(_, _) => Self::INTERVAL_DATA_POINTS,
      XDataSetImpl::Number(_) => Self::STEP_DATA_POINTS,
    }
  }

  pub fn has_required_len(&self) -> bool {
    self.required_len().is_some()
  }

  pub fn gen_points(&self, requested_size: Option<usize>) -> Result<Vec<Number>, LengthError> {
    if requested_size.is_some() && self.required_len().is_some() && requested_size != self.required_len() {
      // unwrap: We just checked that both values are Some, not None.
      return Err(LengthError {
        expected: self.required_len().unwrap(),
        actual: requested_size.unwrap(),
      });
    }

    let size = requested_size.unwrap_or_else(|| self.preferred_len());
    Ok(
      match &self.data {
        XDataSetImpl::Vector(v) => v.clone(),
        XDataSetImpl::Interval(min, max) => XDataSet::gen_points_from_interval(min.to_owned(), max.to_owned(), size),
        XDataSetImpl::Number(starting) => XDataSet::gen_points_from_step(starting.to_owned(), size),
      },
    )
  }

  fn gen_points_from_interval(min: Number, max: Number, size: usize) -> Vec<Number> {
    // Corner cases for small sizes :)
    if size == 0 {
      return vec![];
    } else if size == 1 {
      return vec![min];
    }

    let step = (max - min.clone()) / Number::from(size - 1);
    (0..size)
      .map(Number::from)
      .map(|i| i * step.clone() + min.clone())
      .collect()
  }

  fn gen_points_from_step(starting: Number, size: usize) -> Vec<Number> {
    (0..size)
      .map(Number::from)
      .map(|i| starting.clone() + i)
      .collect()
  }
}

impl ExprToXDataSet {
  pub fn new() -> Self {
    Self { _priv: () }
  }

  fn inner_prism() -> impl Prism<Expr, Either<Vec<Number>, Either<Interval, Number>>> {
    DisjPrism::new(
      prisms::expr_to_typed_vector(prisms::ExprToNumber),
      DisjPrism::new(
        prisms::expr_to_interval(),
        prisms::ExprToNumber,
      ),
    )
  }
}

impl Prism<Expr, XDataSetExpr> for ExprToXDataSet {
  fn narrow_type(&self, input: Expr) -> Result<XDataSetExpr, Expr> {
    ExprToXDataSet::inner_prism().narrow_type(input)
      .map(|data| XDataSetExpr { data })
  }

  fn widen_type(&self, input: XDataSetExpr) -> Expr {
    ExprToXDataSet::inner_prism().widen_type(input.data)
  }
}

impl From<XDataSetExpr> for XDataSet {
  fn from(data: XDataSetExpr) -> Self {
    let data = match data.data {
      Either::Left(vec) => XDataSetImpl::Vector(vec),
      Either::Right(Either::Left(interval)) => {
        let (min, max) = interval.into_bounds();
        XDataSetImpl::Interval(min.into_number(), max.into_number())
      }
      Either::Right(Either::Right(number)) => XDataSetImpl::Number(number),
    };
    Self { data }
  }
}
