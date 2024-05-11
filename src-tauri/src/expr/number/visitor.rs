
//! Private module used to implement promotion and visitor semantics
//! for our numerical representation. None of this functionality is
//! directly exposed outside of `crate::expr::number`.

use super::{Number, NumberImpl};

use num::{BigInt, BigRational, ToPrimitive};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum NumberPair {
  Integers(BigInt, BigInt),
  Ratios(BigRational, BigRational),
  Floats(f64, f64),
}

impl NumberPair {
  /// Promote two numbers to a common representation, so we can do
  /// arithmetic on them.
  pub fn promote(left: Number, right: Number) -> NumberPair {
    use NumberImpl::*;
    use NumberPair::*;
    match (left.inner, right.inner) {
      // Coerce both to integers
      (Integer(left), Integer(right)) => Integers(left, right),
      // Coerce both to rational
      (Integer(left), Ratio(right)) => Ratios(int_to_rational(left), right),
      (Ratio(left), Integer(right)) => Ratios(left, int_to_rational(right)),
      (Ratio(left), Ratio(right)) => Ratios(left, right),
      // Coerce both to floats
      (Integer(left), Float(right)) => Floats(int_to_float(left), right),
      (Ratio(left), Float(right)) => Floats(rational_to_float(left), right),
      (Float(left), Integer(right)) => Floats(left, int_to_float(right)),
      (Float(left), Ratio(right)) => Floats(left, rational_to_float(right)),
      (Float(left), Float(right)) => Floats(left, right),
    }
  }
}

fn int_to_rational(i: BigInt) -> BigRational {
  BigRational::from_integer(i)
}

fn int_to_float(i: BigInt) -> f64 {
  i.to_f64().unwrap_or(f64::NAN)
}

fn rational_to_float(r: BigRational) -> f64 {
  r.to_f64().unwrap_or(f64::NAN)
}
