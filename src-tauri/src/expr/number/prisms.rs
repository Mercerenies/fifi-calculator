
use super::real::Number;
use crate::util::prism::Prism;

use num::{ToPrimitive, FromPrimitive, BigInt};

use std::convert::TryFrom;

/// Prism which converts a [`Number`] to a primitive Rust integer
/// type, either signed or unsigned.
#[derive(Debug, Copy, Clone)]
pub struct NumberToPrimInt<T> {
  down: fn(&BigInt) -> Option<T>,
  // Invariant: This function must never actually return `None`. We
  // control all of the constructors for this type, so we can assume
  // this invariant holds.
  up: fn(T) -> Option<BigInt>,
}

pub const fn number_to_usize() -> impl Prism<Number, usize> + Clone {
  NumberToPrimInt {
    down: BigInt::to_usize,
    up: BigInt::from_usize,
  }
}

pub const fn number_to_i64() -> impl Prism<Number, i64> + Clone {
  NumberToPrimInt {
    down: BigInt::to_i64,
    up: BigInt::from_i64,
  }
}

pub const fn number_to_i32() -> impl Prism<Number, i32> + Clone {
  NumberToPrimInt {
    down: BigInt::to_i32,
    up: BigInt::from_i32,
  }
}

pub const fn number_to_u32() -> impl Prism<Number, u32> + Clone {
  NumberToPrimInt {
    down: BigInt::to_u32,
    up: BigInt::from_u32,
  }
}

pub const fn number_to_u8() -> impl Prism<Number, u8> + Clone {
  NumberToPrimInt {
    down: BigInt::to_u8,
    up: BigInt::from_u8,
  }
}

impl<T> Prism<Number, T> for NumberToPrimInt<T> {
  fn narrow_type(&self, number: Number) -> Result<T, Number> {
    let bigint = BigInt::try_from(number).map_err(|err| err.number)?;
    (self.down)(&bigint).ok_or_else(|| bigint.into())
  }

  fn widen_type(&self, number: T) -> Number {
    // unwrap() safety: By the invariant on self.up, we never actually
    // get `None` from the call.
    (self.up)(number).unwrap().into()
  }
}
