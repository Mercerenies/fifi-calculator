
//! Dispatch function to produce the correct command for a given name.

use super::base::Command;

use thiserror::Error;

use std::fmt::{self, Formatter, Display};
use std::collections::HashMap;

pub struct CommandDispatchTable {
  map: HashMap<String, Box<dyn Command + Send + Sync>>,
}

#[derive(Debug, Error, Clone)]
pub struct NoSuchCommandError {
  command: String,
}

impl CommandDispatchTable {
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

impl Display for NoSuchCommandError {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "No such command {}", self.command)
  }
}
