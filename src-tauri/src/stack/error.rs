
use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum StackError {
  #[error("Not enough stack elements, expected at least {expected} but found {actual}.")]
  NotEnoughElements {
    expected: usize,
    actual: usize,
  },
}
