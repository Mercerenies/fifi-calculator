
use thiserror::Error;

#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum StackError {
  #[error("Not enough stack elements, expected at least {expected} but found {actual}.")]
  NotEnoughElements {
    expected: usize,
    actual: usize,
  },
}
