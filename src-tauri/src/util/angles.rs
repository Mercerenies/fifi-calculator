
//! Helpers for keeping track of which angle format is currently in
//! use.

use num::{FromPrimitive, Float, Zero};
use num::traits::FloatConst;

use std::ops::{Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Div, DivAssign};

/// A number representing degrees.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq,
         PartialOrd, Ord)]
pub struct Degrees<T>(pub T);

/// A number representing radians.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq,
         PartialOrd, Ord)]
pub struct Radians<T>(pub T);

impl<T> Degrees<T> {
  pub fn new(value: T) -> Degrees<T> {
    Degrees(value)
  }

  pub fn into_radians(self) -> Radians<T>
  where T: FloatConst + Mul<Output=T> + Div<Output=T> + FromPrimitive {
    Radians(self.0 * T::PI() / T::from_i32(180).unwrap())
  }
}

impl<T> Radians<T> {
  pub fn new(value: T) -> Radians<T> {
    Radians(value)
  }

  pub fn into_degrees(self) -> Degrees<T>
  where T: FloatConst + Mul<Output=T> + Div<Output=T> + FromPrimitive {
    Degrees(self.0 * T::from_i32(180).unwrap() / T::PI())
  }
}

impl<T: Float> Radians<T> {
  pub fn cos(self) -> T {
    self.0.cos()
  }

  pub fn sin(self) -> T {
    self.0.sin()
  }

  pub fn tan(self) -> T {
    self.0.tan()
  }

  pub fn atan(value: T) -> Self {
    Radians(T::atan(value))
  }

  pub fn atan2(y: T, x: T) -> Self {
    Radians(T::atan2(y, x))
  }
}

macro_rules! newtype_impl {
  (impl $trait_: ident for $type_: ident { fn $method: ident };) => {
    impl<T: $trait_> $trait_ for $type_<T> {
      type Output = $type_<T::Output>;

      fn $method(self, rhs: Self) -> $type_<<T as $trait_>::Output> {
        let result = self.0.$method(rhs.0);
        $type_(result)
      }
    }
  };

  (impl $trait_: ident <T> for $type_: ident { fn $method: ident };) => {
    impl<T: $trait_> $trait_<T> for $type_<T> {
      type Output = $type_<T::Output>;

      fn $method(self, rhs: T) -> $type_<<T as $trait_>::Output> {
        let result = self.0.$method(rhs);
        $type_(result)
      }
    }
  };
}

macro_rules! newtype_impl_assign {
  (impl $trait_: ident for $type_: ident { fn $method: ident };) => {
    impl<T: $trait_> $trait_ for $type_<T> {
      fn $method(&mut self, rhs: Self) {
        self.0.$method(rhs.0);
      }
    }
  };

  (impl $trait_: ident <T> for $type_: ident { fn $method: ident };) => {
    impl<T: $trait_> $trait_<T> for $type_<T> {
      fn $method(&mut self, rhs: T) {
        self.0.$method(rhs);
      }
    }
  };
}

newtype_impl! { impl Add for Degrees { fn add }; }
newtype_impl! { impl Add for Radians { fn add }; }
newtype_impl! { impl Sub for Degrees { fn sub }; }
newtype_impl! { impl Sub for Radians { fn sub }; }

newtype_impl! { impl Mul<T> for Degrees { fn mul }; }
newtype_impl! { impl Mul<T> for Radians { fn mul }; }
newtype_impl! { impl Div<T> for Degrees { fn div }; }
newtype_impl! { impl Div<T> for Radians { fn div }; }

newtype_impl_assign! { impl AddAssign for Degrees { fn add_assign }; }
newtype_impl_assign! { impl AddAssign for Radians { fn add_assign }; }
newtype_impl_assign! { impl SubAssign for Degrees { fn sub_assign }; }
newtype_impl_assign! { impl SubAssign for Radians { fn sub_assign }; }

newtype_impl_assign! { impl MulAssign<T> for Degrees { fn mul_assign }; }
newtype_impl_assign! { impl MulAssign<T> for Radians { fn mul_assign }; }
newtype_impl_assign! { impl DivAssign<T> for Degrees { fn div_assign }; }
newtype_impl_assign! { impl DivAssign<T> for Radians { fn div_assign }; }

impl<T: Zero> Zero for Degrees<T> {
  fn zero() -> Self {
    Degrees(T::zero())
  }
  fn is_zero(&self) -> bool {
    self.0.is_zero()
  }
}

impl<T: Zero> Zero for Radians<T> {
  fn zero() -> Self {
    Radians(T::zero())
  }
  fn is_zero(&self) -> bool {
    self.0.is_zero()
  }
}
