
//! Utilities for working with numbers of different radixes.

use super::Sign;
use crate::util::remove_suffix;
use crate::util::prism::Prism;

use num::{BigInt, Zero, Signed, ToPrimitive};

use std::str::FromStr;
use std::fmt::{self, Display, Formatter};

/// A numerical radix. Supported radixes are from 2 up to 36 inclusive
/// and will use decimal digits first, followed by the uppercase Latin
/// alphabet A-Z.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Radix {
  value: u8,
}

/// Prism which parses a string as a valid numerical radix.
#[derive(Debug, Clone)]
pub struct StringToRadix;

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

/// Options for [`ToDigits::to_digits_opts`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToDigitsOptions {
  /// Maximum number of digits after the decimal point. Anything else
  /// will be truncated toward zero.
  ///
  /// Integer types which implement `ToDigits` should ignore this
  /// field.
  pub max_fractional_digits: usize,
}

/// An implementor of this trait is a number-like type that can be
/// converted into its digits.
pub trait ToDigits {
  fn to_digits_opts(&self, radix: Radix, opts: ToDigitsOptions) -> Digits;

  fn to_digits(&self, radix: Radix) -> Digits {
    self.to_digits_opts(radix, ToDigitsOptions::default())
  }

  fn to_string_radix(&self, radix: Radix) -> String {
    self.to_digits(radix).to_string()
  }
}

pub fn digit_into_char(digit: u8) -> char {
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
    if (2..=36).contains(&value) {
      Some(Radix { value })
    } else {
      None
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

impl Prism<String, Radix> for StringToRadix {
  fn narrow_type(&self, input: String) -> Result<Radix, String> {
    let Ok(n) = u8::from_str(&input) else { return Err(input); };
    Radix::try_new(n).ok_or(input)
  }

  fn widen_type(&self, radix: Radix) -> String {
    u8::from(radix).to_string()
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

impl Default for ToDigitsOptions {
  fn default() -> Self {
    Self {
      max_fractional_digits: 10,
    }
  }
}

impl ToDigits for BigInt {
  fn to_digits_opts(&self, radix: Radix, _: ToDigitsOptions) -> Digits {
    let sign = if *self < BigInt::zero() { Sign::Negative } else { Sign::Positive };
    let mut digits = Vec::new();
    let mut n = self.abs();
    while !n.is_zero() {
      let digit = &n % radix.value;
      digits.push(digit.to_u8().unwrap()); // unwrap: radix.value is at most 36, which fits in a u8
      n /= radix.value;
    }
    digits.reverse();
    Digits {
      sign,
      whole: digits,
      fraction: Vec::new(),
    }
  }
}

macro_rules! impl_to_digits_signed {
  (impl ToDigits for $signed_type: ident by $_unsigned_type: ident) => {
    impl ToDigits for $signed_type {
      fn to_digits_opts(&self, radix: Radix, _: ToDigitsOptions) -> Digits {
        let sign = if *self < 0 { Sign::Negative } else { Sign::Positive };
        let mut digits = self.unsigned_abs().to_digits(radix);
        digits.sign = sign;
        digits
      }
    }
  }
}

macro_rules! impl_to_digits_unsigned {
  (impl ToDigits for $unsigned_type: ident) => {
    impl ToDigits for $unsigned_type {
      fn to_digits_opts(&self, radix: Radix, _: ToDigitsOptions) -> Digits {
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

macro_rules! impl_to_digits_floating {
  (impl ToDigits for $type_: ident) => {
    impl ToDigits for $type_ {
      fn to_digits_opts(&self, radix: Radix, opts: ToDigitsOptions) -> Digits {
        const EPSILON: $type_ = 0.000_001;

        let sign = if *self < 0.0 { Sign::Negative } else { Sign::Positive };
        let n = self.abs();

        // Whole digits
        let mut whole_digits = Vec::new();
        let mut whole_part = n.floor();
        while whole_part.abs() > EPSILON {
          let digit = (whole_part % radix.value as $type_) as u8;
          whole_digits.push(digit);
          whole_part /= radix.value as $type_;
          whole_part = whole_part.floor();
        }
        whole_digits.reverse();

        // Fractional digits
        let mut fraction_digits = Vec::new();
        let mut fraction_part = n - whole_part;
        let mut i = 0;
        while fraction_part != 0.0 && i < opts.max_fractional_digits {
          fraction_part *= radix.value as $type_;
          let digit = (fraction_part.floor() % radix.value as $type_) as u8;
          fraction_digits.push(digit);
          i += 1;
        }
        remove_suffix(&mut fraction_digits, |x| *x == 0);

        Digits::new(sign, whole_digits, fraction_digits)
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

impl_to_digits_floating!(impl ToDigits for f32);
impl_to_digits_floating!(impl ToDigits for f64);

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_radix_constructor() {
    Radix::try_new(2).unwrap();
    Radix::try_new(8).unwrap();
    Radix::try_new(11).unwrap();
    Radix::try_new(35).unwrap();
    Radix::try_new(36).unwrap();
    assert_eq!(Radix::try_new(37), None);
    assert_eq!(Radix::try_new(1), None);
    assert_eq!(Radix::try_new(0), None);
    assert_eq!(Radix::try_new(99), None);
  }

  #[test]
  fn test_radix_panicking_constructor() {
    Radix::new(2);
    Radix::new(36);
  }

  #[test]
  #[should_panic]
  fn test_radix_panicking_constructor_on_out_of_bounds() {
    Radix::new(38);
  }

  #[test]
  fn test_radix_to_u8() {
    let radix = Radix::new(17);
    assert_eq!(u8::from(radix), 17);
  }

  #[test]
  fn test_unsigned_to_binary() {
    assert_eq!(5u64.to_string_radix(Radix::BINARY), "101");
    assert_eq!(99u64.to_string_radix(Radix::BINARY), "1100011");
    assert_eq!(100u64.to_string_radix(Radix::BINARY), "1100100");
    assert_eq!(0u64.to_string_radix(Radix::BINARY), "0");
  }

  #[test]
  fn test_signed_to_binary() {
    assert_eq!(5i64.to_string_radix(Radix::BINARY), "101");
    assert_eq!((-6i64).to_string_radix(Radix::BINARY), "-110");
    assert_eq!(0i64.to_string_radix(Radix::BINARY), "0");
  }

  #[test]
  fn test_signed_to_hexadecimal() {
    assert_eq!(108i64.to_string_radix(Radix::HEXADECIMAL), "6C");
    assert_eq!((-108i64).to_string_radix(Radix::HEXADECIMAL), "-6C");
  }

  #[test]
  fn test_signed_to_base36() {
    assert_eq!(24_236_467i64.to_string_radix(Radix::new(36)), "EFGZ7");
    assert_eq!((-24_236_467i64).to_string_radix(Radix::new(36)), "-EFGZ7");
  }

  #[test]
  fn test_bigint_to_hexadecimal() {
    assert_eq!(BigInt::from(108i64).to_string_radix(Radix::HEXADECIMAL), "6C");
    assert_eq!(BigInt::from(0i64).to_string_radix(Radix::HEXADECIMAL), "0");
    assert_eq!(BigInt::from(-108i64).to_string_radix(Radix::HEXADECIMAL), "-6C");
  }

  #[test]
  fn test_floating_to_binary() {
    assert_eq!(35f64.to_string_radix(Radix::BINARY), "100011");
    assert_eq!(0.0f64.to_string_radix(Radix::BINARY), "0");
    assert_eq!(0.5f64.to_string_radix(Radix::BINARY), "0.1");
    assert_eq!(0.25f64.to_string_radix(Radix::BINARY), "0.01");
    assert_eq!(0.75f64.to_string_radix(Radix::BINARY), "0.11");
  }

  #[test]
  fn test_floating_to_hexadecimal() {
    assert_eq!(3.6875f64.to_string_radix(Radix::HEXADECIMAL), "3.B");
    assert_eq!((-3.6875f64).to_string_radix(Radix::HEXADECIMAL), "-3.B");
  }

  #[test]
  fn test_floating_to_digits() {
    let opts = ToDigitsOptions {
      max_fractional_digits: 4,
    };
    assert_eq!(
      0.5f64.to_digits_opts(Radix::new(3), opts),
      Digits {
        sign: Sign::Positive,
        whole: vec![],
        fraction: vec![1, 1, 1, 1],
      },
    );
  }

  #[test]
  fn test_prism_widen() {
    assert_eq!(StringToRadix.widen_type(Radix::new(32)), "32");
  }

  #[test]
  fn test_prism_narrow() {
    assert_eq!(StringToRadix.narrow_type(String::from("3")), Ok(Radix::new(3)));
    assert_eq!(StringToRadix.narrow_type(String::from("19")), Ok(Radix::new(19)));
    assert_eq!(StringToRadix.narrow_type(String::from("")), Err(String::from("")));
    assert_eq!(StringToRadix.narrow_type(String::from("e")), Err(String::from("e")));
    assert_eq!(StringToRadix.narrow_type(String::from("-1")), Err(String::from("-1")));
    assert_eq!(StringToRadix.narrow_type(String::from("37")), Err(String::from("37")));
  }
}
