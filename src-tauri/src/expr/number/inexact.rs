
use num::{Zero, One};

use std::ops::{Add, Mul, Div, Neg};

/// Trait for inexact division. Inexact division should work like
/// ordinary division (via [`Div::div`]) except that the former shall
/// avoid creating rational numbers. That is, if the numerator and
/// denominator are integers and the denominator does not divide the
/// numerator, then normal division ([`Div::div`]) would produce an
/// exact rational number as result, but inexact division should fall
/// back to floating-point values instead.
pub trait DivInexact<Rhs = Self> {
  type Output;

  /// Division, but avoids producing (proper) rational values if none
  /// of the inputs are (proper) rationals. See
  /// [`Number::div_inexact`] for details on how this works. Note that
  /// [`ComplexNumber::div_inexact`] considers the real and imaginary
  /// components separately when determining whether to make a value
  /// inexact.
  fn div_inexact(&self, other: &Rhs) -> Self::Output;
}

/// Adapts a type which implements [`DivInexact`] to use the
/// [`div_inexact`](DivInexact::div_inexact) method for ordinary
/// [`Div::div`] division.
///
/// In addition to implementing [`Div::div`] with custom behavior,
/// this newtype wrapper also implements several other arithmetic
/// traits as simple delegators to the underlying type, mainly to
/// attain compatibility with the
/// [`MatrixElement`](crate::util::matrix::MatrixElement) and
/// [`MatrixFieldElement`](crate::util::matrix::MatrixFieldElement) APIs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WithInexactDiv<T>(pub T);

impl<T> Div<&WithInexactDiv<T>> for &WithInexactDiv<T>
where T: DivInexact {
  type Output = WithInexactDiv<<T as DivInexact>::Output>;

  /// Implements `div` by delegating to
  /// [`div_inexact`](DivInexact::div_inexact).
  fn div(self, other: &WithInexactDiv<T>) -> Self::Output {
    WithInexactDiv(self.0.div_inexact(&other.0))
  }
}

impl<T> Div<&WithInexactDiv<T>> for WithInexactDiv<T>
where T: DivInexact {
  type Output = WithInexactDiv<<T as DivInexact>::Output>;

  /// Implements `div` by delegating to
  /// [`div_inexact`](DivInexact::div_inexact).
  fn div(self, other: &WithInexactDiv<T>) -> Self::Output {
    WithInexactDiv(self.0.div_inexact(&other.0))
  }
}

impl<T: Add> Add for WithInexactDiv<T> {
  type Output = WithInexactDiv<<T as Add>::Output>;

  fn add(self, other: Self) -> Self::Output {
    WithInexactDiv(self.0 + other.0)
  }
}

impl<'a, T: Add<&'a T>> Add<&'a WithInexactDiv<T>> for WithInexactDiv<T> {
  type Output = WithInexactDiv<<T as Add<&'a T>>::Output>;

  fn add(self, other: &'a Self) -> Self::Output {
    WithInexactDiv(self.0 + &other.0)
  }
}

impl<T: Mul> Mul for WithInexactDiv<T> {
  type Output = WithInexactDiv<<T as Mul>::Output>;

  fn mul(self, other: Self) -> Self::Output {
    WithInexactDiv(self.0 * other.0)
  }
}

impl<'a, T: Mul<&'a T>> Mul<&'a WithInexactDiv<T>> for WithInexactDiv<T> {
  type Output = WithInexactDiv<<T as Mul<&'a T>>::Output>;

  fn mul(self, other: &'a Self) -> Self::Output {
    WithInexactDiv(self.0 * &other.0)
  }
}

impl<T: Zero> Zero for WithInexactDiv<T> {
  fn zero() -> Self {
    WithInexactDiv(T::zero())
  }

  fn is_zero(&self) -> bool {
    self.0.is_zero()
  }
}

impl<T: One> One for WithInexactDiv<T> {
  fn one() -> Self {
    WithInexactDiv(T::one())
  }
}

impl<T: Neg> Neg for WithInexactDiv<T> {
  type Output = WithInexactDiv<<T as Neg>::Output>;

  fn neg(self) -> Self::Output {
    WithInexactDiv(- self.0)
  }
}
