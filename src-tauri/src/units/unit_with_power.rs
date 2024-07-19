
use super::unit::Unit;
use super::composite::CompositeUnit;
use super::dimension::Dimension;
use num::pow::Pow;

use std::fmt::{self, Formatter, Display};
use std::ops::{Mul, Div};
use std::cmp::Ordering;

/// A named unit raised to an integer power.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitWithPower<T> {
  pub unit: Unit<T>,
  pub exponent: i64,
}

impl<T> UnitWithPower<T> {
  pub fn dimension(&self) -> Dimension {
    self.unit.dimension().pow(self.exponent)
  }

  pub fn to_base<'a, U>(&'a self, mut amount: U) -> U
  where U: Mul<&'a T, Output = U>,
        U: Div<&'a T, Output = U> {
    match self.exponent.cmp(&0) {
      Ordering::Greater => {
        for _ in 0..self.exponent {
          amount = self.unit.to_base(amount);
        }
      }
      Ordering::Less => {
        for _ in 0..(-self.exponent) {
          amount = self.unit.from_base(amount);
        }
      }
      Ordering::Equal => {},
    }
    amount
  }

  pub fn from_base<'a, U>(&'a self, mut amount: U) -> U
  where U: Mul<&'a T, Output = U>,
        U: Div<&'a T, Output = U> {
    match self.exponent.cmp(&0) {
      Ordering::Greater => {
        for _ in 0..self.exponent {
          amount = self.unit.from_base(amount);
        }
      }
      Ordering::Less => {
        for _ in 0..(-self.exponent) {
          amount = self.unit.to_base(amount);
        }
      }
      Ordering::Equal => {},
    }
    amount
  }
}

impl<T> Display for UnitWithPower<T> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    if self.exponent == 1 {
      write!(f, "{}", self.unit)
    } else {
      write!(f, "{}^{}", self.unit, self.exponent)
    }
  }
}

impl<T, S> Mul<S> for UnitWithPower<T>
where S: Into<CompositeUnit<T>> {
  type Output = CompositeUnit<T>;

  fn mul(self, rhs: S) -> Self::Output {
    CompositeUnit::from(self) * rhs
  }
}

impl<T, S> Div<S> for UnitWithPower<T>
where S: Into<CompositeUnit<T>> {
  type Output = CompositeUnit<T>;

  fn div(self, rhs: S) -> Self::Output {
    CompositeUnit::from(self) / rhs
  }
}

impl<T> Pow<i64> for UnitWithPower<T> {
  type Output = UnitWithPower<T>;

  fn pow(self, rhs: i64) -> Self::Output {
    UnitWithPower { unit: self.unit, exponent: self.exponent * rhs }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::units::dimension::BaseDimension;
  use crate::units::test_utils::{kilometers, seconds};

  use num::One;

  #[test]
  fn test_unit_with_power_dimension() {
    let unit = UnitWithPower { unit: kilometers(), exponent: 3 };
    assert_eq!(unit.dimension(), Dimension::singleton(BaseDimension::Length).pow(3));
    let unit = UnitWithPower { unit: seconds(), exponent: -2 };
    assert_eq!(unit.dimension(), Dimension::singleton(BaseDimension::Time).pow(-2));
    let unit = UnitWithPower { unit: kilometers(), exponent: 0 };
    assert_eq!(unit.dimension(), Dimension::one());
  }

  #[test]
  fn test_unit_with_power_to_base() {
    let unit = UnitWithPower { unit: kilometers(), exponent: 3 };
    assert_eq!(unit.to_base(2.0), 2_000_000_000.0);
    let unit = UnitWithPower { unit: kilometers(), exponent: -1 };
    assert_eq!(unit.to_base(2_000.0), 2.0);
    let unit = UnitWithPower { unit: kilometers(), exponent: 0 };
    assert_eq!(unit.to_base(199.0), 199.0);
  }

  #[test]
  fn test_unit_with_power_from_base() {
    let unit = UnitWithPower { unit: kilometers(), exponent: 3 };
    assert_eq!(unit.from_base(2_000_000_000.0), 2.0);
    let unit = UnitWithPower { unit: kilometers(), exponent: -1 };
    assert_eq!(unit.from_base(2.0), 2_000.0);
    let unit = UnitWithPower { unit: kilometers(), exponent: 0 };
    assert_eq!(unit.from_base(199.0), 199.0);
  }
}
