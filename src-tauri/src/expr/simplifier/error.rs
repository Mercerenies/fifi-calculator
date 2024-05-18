
use thiserror::Error;

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
}
