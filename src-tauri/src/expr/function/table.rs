
use super::Function;

use std::collections::HashMap;

/// A table of known functions.
#[derive(Debug, Default)]
pub struct FunctionTable {
  known_functions: HashMap<String, Function>,
}

impl FunctionTable {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn with_capacity(capacity: usize) -> Self {
    Self {
      known_functions: HashMap::with_capacity(capacity),
    }
  }

  pub fn insert(&mut self, func: Function) {
    self.known_functions.insert(func.name().to_string(), func);
  }

  pub fn get(&self, name: &str) -> Option<&Function> {
    self.known_functions.get(name)
  }
}

impl FromIterator<Function> for FunctionTable {
  fn from_iter<I: IntoIterator<Item = Function>>(iter: I) -> Self {
    let iter = iter.into_iter();
    let (len_bound, _) = iter.size_hint();
    let mut table = Self::with_capacity(len_bound);
    for func in iter {
      table.insert(func);
    }
    table
  }
}
