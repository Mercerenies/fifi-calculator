
//! Helpers for validating text from the frontend against various
//! conditions.

use crate::expr::var::Var;

use serde::{Serialize, Deserialize};

use std::fmt::{self, Display, Formatter};
use std::error::{Error as StdError};

/// Types of validations that can be requested of the backend.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Validator {
  /// Validator that checks whether its input is a valid variable
  /// name. Invokes [`validate_var`].
  Variable,
}

#[derive(Debug)]
pub struct ValidationError {
  inner: Box<dyn StdError + Send + Sync + 'static>
}

impl ValidationError {
  pub fn from_error(err: impl StdError + Send + Sync + 'static) -> Self {
    Self {
      inner: Box::new(err),
    }
  }
}

impl Display for ValidationError {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    self.inner.fmt(f)
  }
}

impl StdError for ValidationError {
  fn source(&self) -> Option<&(dyn StdError + 'static)> {
    Some(&*self.inner)
  }
}

pub fn validate(validator: Validator, payload: String) -> Result<(), ValidationError> {
  match validator {
    Validator::Variable => validate_var(payload).map(|_| ())
  }
}

/// Validates that the given string is a valid variable name.
pub fn validate_var(name: String) -> Result<Var, ValidationError> {
  Var::try_from(name).map_err(ValidationError::from_error)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_validate_var_valid() {
    validate_var("abc".to_owned()).unwrap();
    validate_var("abc'".to_owned()).unwrap();
    validate_var("abc123".to_owned()).unwrap();
  }

  #[test]
  fn test_validate_var_invalid() {
    validate_var("3".to_owned()).unwrap_err();
    validate_var("''".to_owned()).unwrap_err();
    validate_var("a_b".to_owned()).unwrap_err();
  }
}
