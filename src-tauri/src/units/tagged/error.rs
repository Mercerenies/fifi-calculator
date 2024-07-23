
use crate::util::prism::ErrorWithPayload;
use crate::units::composite::CompositeUnit;
use crate::units::tagged::Tagged;

use thiserror::Error;

use std::fmt::Debug;

#[derive(Clone, Debug, Error)]
#[error("Failed to convert units")]
pub struct TryConvertError<S, U> {
  pub tagged_value: Tagged<S, U>,
  pub attempted_target: CompositeUnit<U>,
}

impl<S: Debug, U: Debug> ErrorWithPayload<Tagged<S, U>> for TryConvertError<S, U> {
  fn recover_payload(self) -> Tagged<S, U> {
    self.tagged_value
  }
}
