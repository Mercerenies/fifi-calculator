
use crate::stack::StackError;
use crate::command::arguments::UserFacingSchemaError;
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
}

impl Error {
  pub fn custom_error(err: impl StdError + Send + Sync + 'static) -> Self {
    Self::CustomError(Box::new(err))
  }
}

impl From<UserFacingSchemaError> for Error {
  fn from(err: UserFacingSchemaError) -> Self {
    Self::custom_error(err)
  }
}

impl From<NoSuchCommandError> for Error {
  fn from(err: NoSuchCommandError) -> Self {
    Self::custom_error(err)
  }
}

impl From<ParseNumberError> for Error {
  fn from(err: ParseNumberError) -> Self {
    Self::custom_error(err)
  }
}
