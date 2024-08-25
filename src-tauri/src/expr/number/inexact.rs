
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
