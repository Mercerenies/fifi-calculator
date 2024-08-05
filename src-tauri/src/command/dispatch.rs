
//! Dispatch function to produce the correct command for a given name.

use super::base::Command;

use thiserror::Error;

use std::collections::{hash_map, HashMap};

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

  pub fn iter(&self) -> impl Iterator<Item=(&str, &(dyn Command + Send + Sync))> {
    self.map.iter().map(|(k, v)| (k.as_str(), v.as_ref()))
  }

  pub fn iter_mut(&mut self) -> impl Iterator<Item=(&str, &mut (dyn Command + Send + Sync + 'static))> {
    self.map.iter_mut().map(|(k, v)| (k.as_str(), v.as_mut()))
  }
}

impl IntoIterator for CommandDispatchTable {
  type Item = (String, Box<dyn Command + Send + Sync>);
  type IntoIter = hash_map::IntoIter<String, Box<dyn Command + Send + Sync>>;

  fn into_iter(self) -> Self::IntoIter {
    self.map.into_iter()
  }
}

impl FromIterator<(String, Box<dyn Command + Send + Sync>)> for CommandDispatchTable {
  fn from_iter<T>(iter: T) -> Self
  where T: IntoIterator<Item = (String, Box<dyn Command + Send + Sync>)> {
    CommandDispatchTable { map: HashMap::from_iter(iter) }
  }
}

impl Extend<(String, Box<dyn Command + Send + Sync>)> for CommandDispatchTable {
  fn extend<T>(&mut self, iter: T)
  where T: IntoIterator<Item = (String, Box<dyn Command + Send + Sync>)> {
    self.map.extend(iter)
  }
}
