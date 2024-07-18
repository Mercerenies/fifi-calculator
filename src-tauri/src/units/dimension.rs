
use crate::util::zip_with;

use num::One;
use num::pow::Pow;

use std::ops::{Mul, Div};
use std::fmt::{self, Formatter, Display};

/// A dimension is a formal product and quotient of zero or more
/// [`BaseDimension`] values.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Dimension {
  dims: [i64; NDIMS],
}

/// Dimensions available for units to represent. Every unit represents
/// a formal product or quotient of zero or more dimensions.
///
/// These are simply the seven base SI units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseDimension {
  Length,
  Time,
  Mass,
  Temperature,
  Current,
  LuminousIntensity,
  AmountOfSubstance,
}

pub const NDIMS: usize = 7;

impl Dimension {
  pub fn singleton(base: BaseDimension) -> Self {
    let mut dims = [0; NDIMS];
    dims[base.dimension_index()] = 1;
    Self { dims }
  }

  pub fn get(&self, base: BaseDimension) -> i64 {
    self.dims[base.dimension_index()]
  }

  pub fn get_mut(&mut self, base: BaseDimension) -> &mut i64 {
    &mut self.dims[base.dimension_index()]
  }

  pub fn components(&self) -> impl Iterator<Item = (BaseDimension, i64)> + '_ {
    BaseDimension::ALL.iter()
      .copied()
      .zip(self.dims.iter().copied())
      .filter(|(_, x)| *x != 0)
  }

  pub fn into_components(self) -> impl Iterator<Item = (BaseDimension, i64)> {
    BaseDimension::ALL.iter()
      .copied()
      .zip(self.dims.into_iter())
      .filter(|(_, x)| *x != 0)
  }

  /// Minimum of `self` and `other`, according to the point-wise
  /// lattice on dimensions. That is, the power of each base dimension
  /// is considered in isolation and the smaller power is chosen in
  /// each case.
  pub fn min(&self, other: &Self) -> Self {
    let mut result = Dimension::one();
    for index in 0..NDIMS {
      result.dims[index] = self.dims[index].min(other.dims[index]);
    }
    result
  }

  /// Maximum of `self` and `other`, according to the point-wise
  /// lattice on dimensions. That is, the power of each base dimension
  /// is considered in isolation and the larger power is chosen in
  /// each case.
  pub fn max(&self, other: &Self) -> Self {
    let mut result = Dimension::one();
    for index in 0..NDIMS {
      result.dims[index] = self.dims[index].max(other.dims[index]);
    }
    result
  }
}

impl BaseDimension {
  pub const ALL: [BaseDimension; NDIMS] = [
    BaseDimension::Length,
    BaseDimension::Time,
    BaseDimension::Mass,
    BaseDimension::Temperature,
    BaseDimension::Current,
    BaseDimension::LuminousIntensity,
    BaseDimension::AmountOfSubstance,
  ];

  fn dimension_index(self) -> usize {
    match self {
      BaseDimension::Length => 0,
      BaseDimension::Time => 1,
      BaseDimension::Mass => 2,
      BaseDimension::Temperature => 3,
      BaseDimension::Current => 4,
      BaseDimension::LuminousIntensity => 5,
      BaseDimension::AmountOfSubstance => 6,
    }
  }
}

impl From<BaseDimension> for Dimension {
  fn from(base: BaseDimension) -> Self {
    Dimension::singleton(base)
  }
}

impl Pow<i64> for &Dimension {
  type Output = Dimension;

  fn pow(self, power: i64) -> Dimension {
    Dimension {
      dims: self.dims.map(|x| x * power),
    }
  }
}

impl Pow<i64> for BaseDimension {
  type Output = Dimension;

  fn pow(self, power: i64) -> Dimension {
    Dimension::singleton(self).pow(power)
  }
}

impl Mul for Dimension {
  type Output = Self;

  fn mul(self, rhs: Self) -> Self {
    Dimension {
      dims: zip_with(self.dims, rhs.dims, |a, b| a + b),
    }
  }
}

impl Mul<BaseDimension> for Dimension {
  type Output = Self;

  fn mul(self, rhs: BaseDimension) -> Self {
    self * Dimension::singleton(rhs)
  }
}

impl Div for Dimension {
  type Output = Self;

  fn div(self, rhs: Self) -> Self {
    Dimension {
      dims: zip_with(self.dims, rhs.dims, |a, b| a - b),
    }
  }
}

impl Div<BaseDimension> for Dimension {
  type Output = Self;

  fn div(self, rhs: BaseDimension) -> Self {
    self / Dimension::singleton(rhs)
  }
}

impl Mul for BaseDimension {
  type Output = Dimension;

  fn mul(self, rhs: Self) -> Dimension {
    Dimension::singleton(self) * Dimension::singleton(rhs)
  }
}

impl Mul<Dimension> for BaseDimension {
  type Output = Dimension;

  fn mul(self, rhs: Dimension) -> Dimension {
    Dimension::singleton(self) * rhs
  }
}

impl Div for BaseDimension {
  type Output = Dimension;

  fn div(self, rhs: Self) -> Dimension {
    Dimension::singleton(self) / Dimension::singleton(rhs)
  }
}

impl Div<Dimension> for BaseDimension {
  type Output = Dimension;

  fn div(self, rhs: Dimension) -> Dimension {
    Dimension::singleton(self) / rhs
  }
}

impl One for Dimension {
  fn one() -> Self {
    Self { dims: [0; NDIMS] }
  }

  fn is_one(&self) -> bool {
    self.dims.iter().all(|x| *x == 0)
  }
}

impl Display for BaseDimension {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      BaseDimension::Length => write!(f, "length"),
      BaseDimension::Time => write!(f, "time"),
      BaseDimension::Mass => write!(f, "mass"),
      BaseDimension::Temperature => write!(f, "temperature"),
      BaseDimension::Current => write!(f, "current"),
      BaseDimension::LuminousIntensity => write!(f, "intensity"),
      BaseDimension::AmountOfSubstance => write!(f, "amount"),
    }
  }
}

impl Display for Dimension {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    let mut numerator: Vec<String> = Vec::new();
    let mut denominator: Vec<String> = Vec::new();
    for dim in BaseDimension::ALL {
      match self.get(dim) {
        0 => {
          // Do not include this value in the output.
        }
        1 => {
          numerator.push(dim.to_string());
        }
        -1 => {
          denominator.push(dim.to_string());
        }
        power if power > 0 => {
          numerator.push(format!("{}^{}", dim, power));
        }
        power if power < 0 => {
          denominator.push(format!("{}^{}", dim, -power));
        }
        _ => {
          unreachable!();
        }
      }
    }
    if numerator.is_empty() {
      write!(f, "1")?;
    } else {
      write!(f, "{}", numerator.join(" "))?;
    }
    if !denominator.is_empty() {
      write!(f, " / {}", denominator.join(" "))?;
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_singleton() {
    let value = Dimension::singleton(BaseDimension::Time);
    assert_eq!(value.dims, [0, 1, 0, 0, 0, 0, 0]);
  }

  #[test]
  fn test_pow() {
    let value = Dimension { dims: [1, 2, 3, 4, 5, 6, 7] }.pow(2);
    assert_eq!(value.dims, [2, 4, 6, 8, 10, 12, 14]);
    let value = Dimension { dims: [1, -1, 2, 2, -3, 3, 10] }.pow(-2);
    assert_eq!(value.dims, [-2, 2, -4, -4, 6, -6, -20]);
    let value = Dimension { dims: [1, -1, 2, 2, -3, 3, 10] }.pow(0);
    assert_eq!(value, Dimension::one());
  }

  #[test]
  fn test_get() {
    let value = Dimension { dims: [1, 2, 3, 4, 5, 6, 7] };
    assert_eq!(value.get(BaseDimension::LuminousIntensity), 6);
    assert_eq!(value.get(BaseDimension::Mass), 3);
  }

  #[test]
  fn test_get_mut() {
    let mut value = Dimension { dims: [1, 2, 3, 4, 5, 6, 7] };
    *value.get_mut(BaseDimension::LuminousIntensity) = 99;
    assert_eq!(value.dims, [1, 2, 3, 4, 5, 99, 7]);
  }

  #[test]
  fn test_min_dimension() {
    let a = Dimension { dims: [1, 2, 3, 4, 5, 6, 7] };
    let b = Dimension { dims: [7, 6, 5, 4, 3, 2, 1] };
    let result = a.min(&b);
    assert_eq!(result.dims, [1, 2, 3, 4, 3, 2, 1]);
  }

  #[test]
  fn test_max_dimension() {
    let a = Dimension { dims: [1, 2, 3, 4, 5, 6, 7] };
    let b = Dimension { dims: [7, 6, 5, 4, 3, 2, 1] };
    let result = a.max(&b);
    assert_eq!(result.dims, [7, 6, 5, 4, 5, 6, 7]);
  }

  #[test]
  fn test_mul() {
    let a = Dimension { dims: [1, 2, 3, 4, 5, 6, 7] };
    let b = Dimension { dims: [-1, 2, 2, 2, 10, 10, 10] };
    assert_eq!(
      a * b,
      Dimension { dims: [0, 4, 5, 6, 15, 16, 17] },
    );
  }

  #[test]
  fn test_div() {
    let a = Dimension { dims: [1, 2, 3, 4, 5, 6, 7] };
    let b = Dimension { dims: [-1, 2, 2, 2, 10, 10, 10] };
    assert_eq!(
      a / b,
      Dimension { dims: [2, 0, 1, 2, -5, -4, -3] },
    );
  }

  #[test]
  fn test_display_on_singleton() {
    let dim = Dimension { dims: [0, 0, 1, 0, 0, 0, 0] };
    assert_eq!(dim.to_string(), "mass");
  }

  #[test]
  fn test_display_on_power() {
    let dim = Dimension { dims: [0, 0, 3, 0, 0, 0, 0] };
    assert_eq!(dim.to_string(), "mass^3");
    let dim = Dimension { dims: [0, 0, 0, -3, 0, 0, 0] };
    assert_eq!(dim.to_string(), "1 / temperature^3");
    let dim = Dimension { dims: [0, 0, 0, -1, 0, 0, 0] };
    assert_eq!(dim.to_string(), "1 / temperature");
  }

  #[test]
  fn test_display_on_one() {
    assert_eq!(Dimension::one().to_string(), "1");
  }

  #[test]
  fn test_display_on_composite() {
    let dim = Dimension { dims: [0, 1, 3, 0, -1, 1, -2] };
    assert_eq!(dim.to_string(), "time mass^3 intensity / current amount^2");
  }
}
