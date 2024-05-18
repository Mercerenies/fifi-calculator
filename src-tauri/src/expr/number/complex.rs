
use super::{Number, NumberRepr};
use crate::util::stricteq::StrictEq;

use std::fmt::{self, Formatter, Display};
use std::ops;

/// A complex number has a real part and an imaginary part.
///
/// Note that each component of a `ComplexNumber` independently has a
/// representation and may or may not be exact. So it is possible to
/// use this one type to do both engineering approximations (with
/// representation [`NumberRepr::Float`]) and exact Gaussian integer
/// mathematics (with representation [`NumberRepr::Integer`]).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ComplexNumber {
  real: Number,
  imag: Number,
}

impl ComplexNumber {
  pub fn new(real: Number, imag: Number) -> Self {
    Self { real, imag }
  }

  pub fn real(&self) -> &Number {
    &self.real
  }

  pub fn imag(&self) -> &Number {
    &self.imag
  }

  pub fn real_repr(&self) -> NumberRepr {
    self.real.repr()
  }

  pub fn imag_repr(&self) -> NumberRepr {
    self.imag.repr()
  }
}

impl StrictEq for ComplexNumber {
  /// Compares the inner components using [`Number::strict_eq`].
  fn strict_eq(&self, other: &ComplexNumber) -> bool {
    self.real.strict_eq(&other.real) && self.imag.strict_eq(&other.imag)
  }
}

impl Display for ComplexNumber {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "({}, {})", self.real, self.imag)
  }
}

impl ops::Add for ComplexNumber {
  type Output = ComplexNumber;

  fn add(self, other: ComplexNumber) -> ComplexNumber {
    ComplexNumber {
      real: self.real + other.real,
      imag: self.imag + other.imag,
    }
  }
}

impl ops::Add for &ComplexNumber {
  type Output = ComplexNumber;

  fn add(self, other: &ComplexNumber) -> ComplexNumber {
    self.to_owned() + other.to_owned()
  }
}

impl ops::Sub for ComplexNumber {
  type Output = ComplexNumber;

  fn sub(self, other: ComplexNumber) -> ComplexNumber {
    ComplexNumber {
      real: self.real - other.real,
      imag: self.imag - other.imag,
    }
  }
}

impl ops::Sub for &ComplexNumber {
  type Output = ComplexNumber;

  fn sub(self, other: &ComplexNumber) -> ComplexNumber {
    self.to_owned() - other.to_owned()
  }
}

impl ops::Mul for ComplexNumber {
  type Output = ComplexNumber;

  fn mul(self, other: ComplexNumber) -> ComplexNumber {
    ComplexNumber {
      real: &self.real * &other.real - &self.imag * &other.imag,
      imag: &self.imag * &other.real + &self.real * &other.imag,
    }
  }
}

impl ops::Mul for &ComplexNumber {
  type Output = ComplexNumber;

  fn mul(self, other: &ComplexNumber) -> ComplexNumber {
    self.to_owned() * other.to_owned()
  }
}

/// Exactness-preserving division.
impl ops::Div for ComplexNumber {
  type Output = ComplexNumber;

  fn div(self, other: ComplexNumber) -> ComplexNumber {
    let denominator = &other.real * &other.real + &other.imag * &other.imag;
    ComplexNumber {
      real: (&self.real * &other.real + &self.imag * &other.imag) / denominator.clone(),
      imag: (&self.imag * &other.real - &self.real * &other.imag) / denominator,
    }
  }
}

impl ops::Div for &ComplexNumber {
  type Output = ComplexNumber;

  fn div(self, other: &ComplexNumber) -> ComplexNumber {
    self.to_owned() / other.to_owned()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::assert_strict_eq;

  // TODO The rest of these tests

  #[test]
  fn test_add() {
    assert_strict_eq!(
      ComplexNumber::new(Number::from(1), Number::from(2)) +
        ComplexNumber::new(Number::from(3), Number::from(4)),
      ComplexNumber::new(Number::from(4), Number::from(6))
    );
    assert_strict_eq!(
      ComplexNumber::new(Number::from(1.0), Number::from(2)) +
        ComplexNumber::new(Number::from(3), Number::from(4)),
      ComplexNumber::new(Number::from(4.0), Number::from(6))
    );
  }
}
