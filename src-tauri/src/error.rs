
use crate::stack::StackError;
use crate::command::dispatch::NoSuchCommandError;
use crate::expr::number::ParseNumberError;

use thiserror::Error;

use std::error::{Error as StdError};

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
  #[error("{0}")]
  CustomError(Box<dyn StdError + Send + Sync + 'static>),
  #[error("{0}")]
  TauriError(#[from] tauri::Error),
  #[error("{0}")]
  StackError(#[from] StackError),
  #[error("{0}")]
  NoSuchCommandError(#[from] NoSuchCommandError),
  #[error("{0}")]
  ParseNumberError(#[from] ParseNumberError),
}

impl Error {
  pub fn custom_error(err: impl StdError + Send + Sync + 'static) -> Self {
    Self::CustomError(Box::new(err))
  }
}
