
//! Utilities for working with numbers of different radixes.

use super::Sign;

use std::fmt::{self, Display, Formatter};

/// A numerical radix. Supported radixes are from 2 up to 36 inclusive
/// and will use decimal digits first, followed by the uppercase Latin
/// alphabet A-Z.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Radix {
  value: u8,
}

/// The digits of a number. All digits are stored with the most
/// significant digit first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Digits {
  /// The sign of the number. Unsigned zeroes should be represented as
  /// having a positive sign.
  pub sign: Sign,
  /// The digits to the left of the decimal point.
  pub whole: Vec<u8>,
  /// The digits to the right of the decimal point.
  pub fraction: Vec<u8>,
}

/// An implementor of this trait is a number-like type that can be
/// converted into its digits.
pub trait ToDigits {
  fn to_digits(&self, radix: Radix) -> Digits;

  fn to_string_radix(&self, radix: Radix) -> String {
    self.to_digits(radix).to_string()
  }
}

fn digit_into_char(digit: u8) -> char {
  if digit < 10 {
    (b'0' + digit) as char
  } else if digit < 36 {
    (b'A' + digit - 10) as char
  } else {
    panic!("Invalid digit {} in radix", digit)
  }
}

impl Radix {
  pub const BINARY: Radix = Radix { value: 2 };
  pub const OCTAL: Radix = Radix { value: 8 };
  pub const DECIMAL: Radix = Radix { value: 10 };
  pub const HEXADECIMAL: Radix = Radix { value: 16 };

  /// Constructs a new radix, performing a bounds check first.
  pub fn try_new(value: u8) -> Option<Self> {
    if value < 2 || value > 36 {
      None
    } else {
      Some(Radix { value })
    }
  }

  /// Constructs a new radix. Panics if the provided value is out of
  /// bounds.
  pub fn new(value: u8) -> Self {
    Self::try_new(value).expect("Radix out of bounds")
  }
}

impl Digits {
  /// Creates a new `Digits` from a whole and fraction part.
  pub fn new(sign: Sign, whole: Vec<u8>, fraction: Vec<u8>) -> Self {
    Self { sign, whole, fraction }
  }
}

impl Display for Digits {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    if self.sign == Sign::Negative || f.sign_plus() {
      write!(f, "{}", self.sign)?;
    }
    if self.whole.is_empty() {
      write!(f, "0")?
    } else {
      for digit in self.whole.iter() {
        write!(f, "{}", digit_into_char(*digit))?;
      }
    }
    if !self.fraction.is_empty() {
      write!(f, ".")?;
      for digit in self.fraction.iter() {
        write!(f, "{}", digit_into_char(*digit))?;
      }
    }
    Ok(())
  }
}

impl From<Radix> for u8 {
  fn from(radix: Radix) -> Self {
    radix.value
  }
}

macro_rules! impl_to_digits_signed {
  (impl ToDigits for $signed_type: ident by $unsigned_type: ident) => {
    impl ToDigits for $signed_type {
      fn to_digits(&self, radix: Radix) -> Digits {
        let sign = if *self < 0 { Sign::Negative } else { Sign::Positive };
        let mut digits = (self.abs() as $unsigned_type).to_digits(radix);
        digits.sign = sign;
        digits
      }
    }
  }
}

macro_rules! impl_to_digits_unsigned {
  (impl ToDigits for $unsigned_type: ident) => {
    impl ToDigits for $unsigned_type {
      fn to_digits(&self, radix: Radix) -> Digits {
        let mut digits = Vec::new();
        let mut n = *self;
        while n != 0 {
          let digit = n % (radix.value as $unsigned_type);
          digits.push(digit as u8);
          n /= radix.value as $unsigned_type;
        }
        digits.reverse();
        Digits {
          sign: Sign::Positive,
          whole: digits,
          fraction: Vec::new(),
        }
      }
    }
  }
}

impl_to_digits_signed!(impl ToDigits for i8 by u8);
impl_to_digits_signed!(impl ToDigits for i16 by u16);
impl_to_digits_signed!(impl ToDigits for i32 by u32);
impl_to_digits_signed!(impl ToDigits for i64 by u64);

impl_to_digits_unsigned!(impl ToDigits for u8);
impl_to_digits_unsigned!(impl ToDigits for u16);
impl_to_digits_unsigned!(impl ToDigits for u32);
impl_to_digits_unsigned!(impl ToDigits for u64);
