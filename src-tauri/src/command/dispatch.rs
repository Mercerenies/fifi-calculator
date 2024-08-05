
//! Dispatch function to produce the correct command for a given name.

use super::base::Command;

use thiserror::Error;

use std::collections::HashMap;

#[derive(Default)]
pub struct CommandDispatchTable {
  map: HashMap<String, Box<dyn Command + Send + Sync>>,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("No such command {command}")]
pub struct NoSuchCommandError {
  command: String,
}

impl CommandDispatchTable {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn from_hash_map(map: HashMap<String, Box<dyn Command + Send + Sync>>) -> CommandDispatchTable {
    CommandDispatchTable { map }
  }

  pub fn get(&self, name: &str) -> Result<&(dyn Command + Send + Sync), NoSuchCommandError> {
    match self.map.get(name) {
      Some(cmd) => Ok(cmd.as_ref()),
      None => Err(NoSuchCommandError { command: name.to_owned() }),
    }
  }
}
