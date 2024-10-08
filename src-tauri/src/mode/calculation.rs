
use bitflags::bitflags;

/// The calculator's current calculation mode includes several bitwise
/// flags indicating how to evaluate expressions.
///
/// This structure is designed to be cheap to clone, but its exact
/// implementation is private.
#[derive(Clone, Debug, Default)]
pub struct CalculationMode {
  inner: CalculationModeBits,
}

bitflags! {
  #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
  struct CalculationModeBits: u8 {
    /// See [`CalculationMode::has_infinity_flag`].
    const INFINITY = 0b0001;
    /// See [`CalculationMode::has_fractional_flag`].
    const FRACTIONAL = 0b0010;
  }
}

impl CalculationMode {
  pub fn new() -> Self {
    Self::default()
  }

  /// A reasonable default calculation mode for local evaluations
  /// while performing computer algebra or numerical approximations.
  pub fn for_algebra() -> Self {
    Self::default()
  }

  /// The infinity flag is off by default. If the infinity flag is
  /// off, then calculations which would produce infinity, such as
  /// `ln(0)` or `1 / 0`, will produce an error. If the infinity flag
  /// is on, then those calculations will produce one of the infinite
  /// constants: `inf`, `-inf`, `uinf`, or `nan`.
  ///
  /// Expressions which already contain an infinity constant can still
  /// produce infinite results, regardless of this flag's value.
  pub fn has_infinity_flag(&self) -> bool {
    self.inner.contains(CalculationModeBits::INFINITY)
  }

  /// The fractional flag is off by default. If this flag is set, then
  /// expressions on integers which do NOT result in integer results
  /// will attempt to produce an exact rational number as a result. If
  /// this flag is not set, then such calculations will produce a
  /// floating-point result.
  ///
  /// This flag has no effect on expressions which already contain
  /// rational numbers, nor on those which contain floating-point
  /// values.
  pub fn has_fractional_flag(&self) -> bool {
    self.inner.contains(CalculationModeBits::FRACTIONAL)
  }

  /// Sets the infinity flag. See
  /// [`CalculationMode::has_infinity_flag`].
  pub fn set_infinity_flag(&mut self, mode: bool) {
    self.inner.set(CalculationModeBits::INFINITY, mode);
  }

  /// Sets the fractional flag. See
  /// [`CalculationMode::has_fractional_flag`].
  pub fn set_fractional_flag(&mut self, mode: bool) {
    self.inner.set(CalculationModeBits::FRACTIONAL, mode);
  }
}
