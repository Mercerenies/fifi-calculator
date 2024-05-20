
use crate::stack::StackError;
use crate::command::dispatch::NoSuchCommandError;
use crate::expr::number::ParseNumberError;

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
  #[error("{0}")]
  ParseNumberError(#[from] ParseNumberError),
}
