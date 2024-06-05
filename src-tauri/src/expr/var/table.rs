
use super::Var;

use std::collections::HashMap;
use std::iter::FromIterator;

/// A table of variable bindings.
#[derive(Debug, Clone)]
pub struct VarTable<T> {
  data: HashMap<Var, T>,
}

impl<T> VarTable<T> {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn with_capacity(capacity: usize) -> Self {
    Self { data: HashMap::with_capacity(capacity) }
  }

  pub fn get(&self, var: &Var) -> Option<&T> {
    self.data.get(var)
  }

  pub fn contains_key(&self, var: &Var) -> bool {
    self.data.contains_key(var)
  }

  pub fn insert(&mut self, var: Var, value: T) -> Option<T> {
    self.data.insert(var, value)
  }
}

impl<T> Default for VarTable<T> {
  fn default() -> Self {
    Self { data: HashMap::new() }
  }
}

impl<T> FromIterator<(Var, T)> for VarTable<T> {
  fn from_iter<I: IntoIterator<Item = (Var, T)>>(iter: I) -> Self {
    Self { data: HashMap::from_iter(iter) }
  }
}
