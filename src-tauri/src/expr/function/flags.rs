
use bitflags::bitflags;

bitflags! {
  #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
  pub struct FunctionFlags: u32 {
    /// Functions with this flag set are permitted to treat nested
    /// invocations as arguments to the current function. That is, if
    /// `f` has this flag set, then `f(x, f(y, z), t)` can be
    /// simplified to `f(x, y, z, t)`.
    const PERMITS_FLATTENING = 0b0001;
  }
}
