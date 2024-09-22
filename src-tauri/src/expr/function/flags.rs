
use bitflags::bitflags;

bitflags! {
  #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
  pub struct FunctionFlags: u32 {
    /// Functions with this flag set are permitted to treat nested
    /// invocations as arguments to the current function and vice
    /// versa. That is, if `f` has this flag set, then `f(x, f(y, z),
    /// t)` can be simplified to `f(x, y, z, t)` and vice versa.
    ///
    /// This is similar to the binary associativity property, but this
    /// flag applies to functions of variable arity.
    const PERMITS_FLATTENING = 0b0001;
    /// Functions with this flag set are permitted to reorder their
    /// arguments without changing the result. That is, if `f` has
    /// this flag set, then `f(x, y, z)`, `f(y, z, x)`, `f(z, y, x)`,
    /// and any other argument order are guaranteed to produce the
    /// same result (possibly up to floating-point precision
    /// limitations).
    ///
    /// This is a generalization of the binary commutativity property,
    /// but this flag applies to functions of any arity.
    const PERMITS_REORDERING = 0b0010;
    /// Functions with this flag are known to be involutions.
    /// Specifically, if `f` is a function with this flag, then
    /// `f(f(x))` can simplify to `x`, regardless of the value of `x`.
    /// This flag will only be used to simplify unary applications of
    /// the function.
    const IS_INVOLUTION = 0b0100;
  }
}
