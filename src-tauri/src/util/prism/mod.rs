
//! Functional-style prisms for checked downcasts.

mod instances;
mod ext;

pub use instances::{Identity, Composed, Only, OnVec, OnTuple2,
                    DisjPrism, InOption, Conversion, LosslessConversion,
                    Iso, VecToArray};
pub use ext::PrismExt;

use std::error::{Error as StdError};

/// A prism from `Up` to `Down` is an assertion of a subtype
/// relationship between `Up` and `Down`. Specifically, it asserts
/// that every `Down` can be seen as an `Up` in a well-defined way,
/// and that some `Up`s can be safely downcast to type `Down`.
///
/// This prism implementation is based loosely on [the Haskell lens
/// library](https://hackage.haskell.org/package/lens-5.3.2/docs/Control-Lens-Prism.html),
/// and prisms implementing this trait should satisfy similar laws.
///
/// * A widen followed by a narrow should reproduce the original
/// value. That is, for all `d: Down`,
/// `prism.narrow_type(prism.widen_type(d)) === Some(d)`.
///
/// * A successful narrow, followed by a widen, should reproduce the
/// original value completely. That is, for all `u: Up`, if
/// `prism.narrow_type(u) = Ok(d)`, then `prism.widen_type(d) === u`.
///
/// * A failed narrow shall return the original value. That is, for
/// all `u: Up`, if `prism.narrow_type(u) = Err(u1)`, then `u === u1`.
///
/// where `===` is taken to mean "conceptually equal". You can think
/// of this as `==` for types that implement `PartialEq`.
pub trait Prism<Up, Down> {
  /// Attempts to downcast `input` to the type `Down`. This method
  /// shall either return the result of successfully downcasting (as
  /// an `Ok`) or the original input value (as an `Err`).
  fn narrow_type(&self, input: Up) -> Result<Down, Up>;

  /// Widens a `Down` value to its parent type. This must always
  /// succeed.
  fn widen_type(&self, input: Down) -> Up;
}

/// An `ErrorWithPayload<T>` is an `Error` which also contains the
/// original value (of type `T`) on which an operation was attempted.
///
/// This is most commonly used with `TryFrom` implementations, where
/// the original value can be recovered from the error type.
pub trait ErrorWithPayload<T>: StdError {
  fn recover_payload(self) -> T;
}
