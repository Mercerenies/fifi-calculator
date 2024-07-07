
//! Helpers for validating text from the frontend against various
//! conditions.

use anyhow::Context;

use crate::expr::var::Var;

use serde::{Serialize, Deserialize};

/// Types of validations that can be requested of the backend.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Validator {
  /// Validator that checks whether its input is a valid variable
  /// name. Invokes [`validate_var`].
  Variable,
}

pub fn validate(validator: Validator, payload: String) -> anyhow::Result<()> {
  match validator {
    Validator::Variable => validate_var(payload).map(|_| ())
  }
}

/// Validates that the given string is a valid variable name.
pub fn validate_var(name: String) -> Result<Var, anyhow::Error> {
  Var::try_from(name).context("Validation failed: invalid variable name")
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
