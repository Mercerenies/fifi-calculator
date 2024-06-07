
use thiserror::Error;

// TODO: We might decide to convert this to anyhow::Error as our needs
// grow with more simplifier types.

/// An error that occurred during the simplification process.
#[derive(Debug, Clone, Error)]
#[non_exhaustive]
pub enum SimplifierError {
  #[error("{function}: Expected {expected} argument(s) but got {actual}.")]
  ArityError { function: String, expected: usize, actual: usize },
  #[error("{function}: Domain error: {explanation}")]
  DomainError { function: String, explanation: String },
}

pub const DIVISION_BY_ZERO: &str = "Division by zero";

pub const EXPECTED_REAL: &str = "Expected real number";

pub const ZERO_TO_ZERO_POWER: &str = "Indeterminate form 0^0";

impl SimplifierError {
  pub fn division_by_zero(function: impl Into<String>) -> SimplifierError {
    SimplifierError::DomainError {
      function: function.into(),
      explanation: DIVISION_BY_ZERO.to_owned(),
    }
  }

  pub fn expected_real(function: impl Into<String>) -> SimplifierError {
    SimplifierError::DomainError {
      function: function.into(),
      explanation: EXPECTED_REAL.to_owned(),
    }
  }

  pub fn zero_to_zero_power(function: impl Into<String>) -> SimplifierError {
    SimplifierError::DomainError {
      function: function.into(),
      explanation: ZERO_TO_ZERO_POWER.to_owned(),
    }
  }
}
