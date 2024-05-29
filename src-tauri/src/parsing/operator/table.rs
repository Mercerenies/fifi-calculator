
use super::Operator;
use super::precedence::Precedence;
use super::associativity::Associativity;

use std::collections::{hash_map, HashMap};

/// A table of operators, indexed by their name.
#[derive(Debug, Clone, Default)]
pub struct OperatorTable {
  by_function_name: HashMap<String, Operator>,
  by_display_name: HashMap<String, Operator>,
}

impl OperatorTable {
  pub fn new() -> OperatorTable {
    OperatorTable::default()
  }

  pub fn with_capacity(capacity: usize) -> OperatorTable {
    OperatorTable {
      by_function_name: HashMap::with_capacity(capacity),
      by_display_name: HashMap::with_capacity(capacity),
    }
  }

  pub fn get_by_display_name(&self, name: &str) -> Option<&Operator> {
    self.by_display_name.get(name)
  }

  pub fn get_by_function_name(&self, name: &str) -> Option<&Operator> {
    self.by_function_name.get(name)
  }

  pub fn insert(&mut self, op: Operator) {
    self.by_display_name.insert(op.display_name().to_owned(), op.clone());
    self.by_function_name.insert(op.function_name().to_owned(), op);
  }

  pub fn common_operators() -> OperatorTable {
    // Note: We borrow the Emacs Calc operator precedence values here
    // when it makes sense to do so. See
    // https://www.gnu.org/software/emacs/manual/html_mono/calc.html#Composition-Basics
    vec![
      Operator::new("^", Associativity::RIGHT, Precedence::new(200)),
      Operator::new("*", Associativity::FULL, Precedence::new(195)),
      Operator::new("/", Associativity::LEFT, Precedence::new(190)),
      Operator::new("%", Associativity::NONE, Precedence::new(190)),
      Operator::new("+", Associativity::FULL, Precedence::new(180)),
      Operator::new("-", Associativity::LEFT, Precedence::new(180)),
    ].into_iter().collect()
  }

  pub fn iter(&self) -> impl Iterator<Item = &Operator> {
    self.by_display_name.values()
  }
}

impl IntoIterator for OperatorTable {
  type Item = Operator;
  type IntoIter = hash_map::IntoValues<String, Operator>;

  fn into_iter(self) -> Self::IntoIter {
    self.by_display_name.into_values()
  }
}

impl FromIterator<Operator> for OperatorTable {
  fn from_iter<I>(iter: I) -> Self
  where I : IntoIterator<Item = Operator> {
    let iter = iter.into_iter();
    let (len_bound, _) = iter.size_hint();
    let mut table = OperatorTable::with_capacity(len_bound);
    for op in iter {
      table.insert(op);
    }
    table
  }
}
