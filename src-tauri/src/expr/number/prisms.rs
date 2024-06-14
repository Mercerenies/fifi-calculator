
use super::real::Number;
use crate::util::prism::Prism;

use num::{ToPrimitive, BigInt};

use std::convert::TryFrom;

/// Prism which converts a [`Number`] to a `usize`. Fails if the
/// `Number` is not an integer, is a non-positive value, or is too
/// large to fit into a `usize`.
#[derive(Debug, Copy, Clone, Default)]
pub struct NumberToUsize;

/// Prism which converts a [`Number`] to an `i64`. Fails if the
/// `Number` is not an integer or is too large to fit into an `i64`.
#[derive(Debug, Copy, Clone, Default)]
pub struct NumberToI64;

impl Prism<Number, usize> for NumberToUsize {
  fn narrow_type(&self, number: Number) -> Result<usize, Number> {
    let bigint = BigInt::try_from(number).map_err(|err| err.number)?;
    bigint.to_usize().ok_or_else(|| bigint.into())
  }
  fn widen_type(&self, number: usize) -> Number {
    Number::from(number)
  }
}

impl Prism<Number, i64> for NumberToI64 {
  fn narrow_type(&self, number: Number) -> Result<i64, Number> {
    let bigint = BigInt::try_from(number).map_err(|err| err.number)?;
    bigint.to_i64().ok_or_else(|| bigint.into())
  }
  fn widen_type(&self, number: i64) -> Number {
    Number::from(number)
  }
}
