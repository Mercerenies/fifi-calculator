
use thiserror::Error;

/// An error that occurred during the simplification process.
#[derive(Debug, Clone, Error)]
#[non_exhaustive]
pub enum SimplifierError {
  #[error("{function}: Expected {expected} argument(s) but got {actual}.")]
  ArityError { function: String, expected: usize, actual: usize },
}

