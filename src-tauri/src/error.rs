
use crate::stack::error::StackError;
use crate::command::dispatch::NoSuchCommandError;

use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
  #[error("{0}")]
  TauriError(#[from] tauri::Error),
  #[error("{0}")]
  StackError(#[from] StackError),
  #[error("{0}")]
  NoSuchCommandError(#[from] NoSuchCommandError),
}
