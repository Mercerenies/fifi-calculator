
//! Common errors that can occur during simplification.

use thiserror::Error;

use std::error::{Error as StdError};

use std::fmt::{self, Display, Formatter};

/// An error that occurred during the simplification process.
/// Simplifier errors consist of the function that caused the error,
/// as well as an arbitrary [`Error`](std::error::Error) object.
#[derive(Debug)]
pub struct SimplifierError {
  function: String,
  error: anyhow::Error,
}

/// Error indicating that the function's arity was not correct.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("Expected {expected} argument(s) but got {actual}.")]
pub struct ArityError {
  pub expected: usize,
  pub actual: usize,
}

/// Error indicating that one or more of the arguments for a function
/// was out of the supported domain of the function.
#[derive(Debug, Clone, Error)]
#[error("Domain error: {explanation}")]
pub struct DomainError {
  pub explanation: String,
}

pub const DIVISION_BY_ZERO: &str = "Division by zero";

pub const EXPECTED_REAL: &str = "Expected real number";

pub const ZERO_TO_ZERO_POWER: &str = "Indeterminate form 0^0";

impl SimplifierError {
  pub fn new<E, S>(function: S, error: E) -> Self
  where S: Into<String>,
        E: Into<anyhow::Error> {
    Self {
      function: function.into(),
      error: error.into(),
    }
  }

  pub fn custom_error<S>(function: S, error_message: &'static str) -> Self
  where S: Into<String> {
    Self {
      function: function.into(),
      error: anyhow::Error::msg(error_message),
    }
  }

  pub fn function(&self) -> &str {
    &self.function
  }

  pub fn division_by_zero(function: impl Into<String>) -> SimplifierError {
    SimplifierError::new(function, DomainError { explanation: DIVISION_BY_ZERO.to_owned() })
  }

  pub fn expected_real(function: impl Into<String>) -> SimplifierError {
    SimplifierError::new(function, DomainError { explanation: EXPECTED_REAL.to_owned() })
  }

  pub fn zero_to_zero_power(function: impl Into<String>) -> SimplifierError {
    SimplifierError::new(function, DomainError { explanation: ZERO_TO_ZERO_POWER.to_owned() })
  }
}

impl DomainError {
  pub fn new(explanation: impl Into<String>) -> Self {
    Self { explanation: explanation.into() }
  }
}

impl Display for SimplifierError {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}: {}", self.function, self.error)
  }
}

impl StdError for SimplifierError {
  fn source(&self) -> Option<&(dyn StdError + 'static)> {
    Some(self.error.as_ref())
  }
}
