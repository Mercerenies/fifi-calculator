
//! Dispatch function to produce the correct command for a given name.

use super::base::Command;
use super::functional::BinaryFunctionCommand;

use thiserror::Error;

use std::fmt::{self, Formatter, Display};

#[derive(Debug, Error, Clone)]
pub struct NoSuchCommandError {
  command: String,
}

pub fn dispatch(name: &str) -> Result<Box<dyn Command + Send + Sync>, NoSuchCommandError> {
  match name {
    "+" => Ok(Box::new(BinaryFunctionCommand::new("+"))),
    "-" => Ok(Box::new(BinaryFunctionCommand::new("-"))),
    "*" => Ok(Box::new(BinaryFunctionCommand::new("*"))),
    "/" => Ok(Box::new(BinaryFunctionCommand::new("/"))),
    _ => Err(NoSuchCommandError { command: name.to_owned() }),
  }
}

impl Display for NoSuchCommandError {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "No such command {}", self.command)
  }
}
