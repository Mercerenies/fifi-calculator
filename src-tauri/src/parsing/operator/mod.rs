
use std::collections::{hash_map, HashMap};

/// A table of operators, indexed by their name.
#[derive(Debug, Clone, Default)]
pub struct OperatorTable {
  by_function_name: HashMap<String, Operator>,
  by_display_name: HashMap<String, Operator>,
}

/// An operator has a precedence and an associativity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Operator {
  function_name: String,
  display_name: String,
  assoc: Associativity,
  prec: Precedence,
}

/// The precedence of an operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Precedence(u64);

/// The associativity of an operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Associativity {
  left_assoc: bool,
  right_assoc: bool,
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

impl Operator {
  /// Constructs a new operator with the given properties. By default,
  /// the operator's `display_name` _and_ `function_name` are both
  /// equal to `name`. If desired, the caller may override one or the
  /// other using the builder-style methods
  /// [`Operator::with_display_name`] or
  /// [`Operator::with_function_name`].
  pub fn new(name: impl Into<String>, assoc: Associativity, prec: Precedence) -> Operator {
    let name = name.into();
    Operator {
      function_name: name.clone(),
      display_name: name,
      assoc,
      prec,
    }
  }

  /// The name of the function used internally to represent this
  /// operator.
  pub fn function_name(&self) -> &str {
    &self.function_name
  }

  /// The name of the operator, as displayed to the user.
  pub fn display_name(&self) -> &str {
    &self.display_name
  }

  /// Operator identical to `self` but with a different
  /// `function_name`. This does not affect `display_name`.
  pub fn with_function_name(mut self, function_name: impl Into<String>) -> Self {
    self.function_name = function_name.into();
    self
  }

  /// Operator identical to `self` but with a different
  /// `display_name`. This does not affect `function_name`.
  pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
    self.display_name = display_name.into();
    self
  }

  pub fn associativity(&self) -> Associativity {
    self.assoc
  }

  pub fn precedence(&self) -> Precedence {
    self.prec
  }

  pub fn left_precedence(&self) -> Precedence {
    if self.assoc.is_left_assoc() {
      self.prec
    } else {
      self.prec.incremented()
    }
  }

  pub fn right_precedence(&self) -> Precedence {
    if self.assoc.is_right_assoc() {
      self.prec
    } else {
      self.prec.incremented()
    }
  }
}

impl Associativity {
  /// Indicates an operator which associates to the left.
  pub const LEFT: Associativity = Associativity {
    left_assoc: true,
    right_assoc: false,
  };
  /// Indicates an operator which associate to the right.
  pub const RIGHT: Associativity = Associativity {
    left_assoc: false,
    right_assoc: true,
  };
  /// Indicates a non-associative operator, which always requires
  /// parentheses for nested applications of itself.
  pub const NONE: Associativity = Associativity {
    left_assoc: false,
    right_assoc: false,
  };
  /// Indicates an associative operator for which the order of
  /// evaluation doesn't affect the result.
  pub const FULL: Associativity = Associativity {
    left_assoc: true,
    right_assoc: true,
  };
  pub const fn is_left_assoc(self) -> bool {
    self.left_assoc
  }
  pub const fn is_right_assoc(self) -> bool {
    self.right_assoc
  }
  pub const fn is_fully_assoc(self) -> bool {
    self.left_assoc && self.right_assoc
  }
}

impl Precedence {
  pub const MIN: Precedence = Precedence(0);
  pub const MAX: Precedence = Precedence(u64::MAX);

  /// Internally, we store an operator's precedence as ten times the
  /// input value, so that we can increment or decrement to represent
  /// associativity.
  ///
  /// For example, if `#` is a left-associative operator with
  /// (internal) precedence value `p`, then its left-hand side is also
  /// at precedence value `p`, while its right-hand side is at
  /// precedence value `p + 1`, indicating parentheses will be
  /// required if `#` is encountered again.
  ///
  /// Use [`from_raw`](Precedence::from_raw) to bypass the
  /// multiplication and construct a `Precedence` value directly.
  pub fn new(n: u64) -> Precedence {
    Precedence(n * 10)
  }

  pub fn from_raw(n: u64) -> Precedence {
    Precedence(n)
  }

  pub fn incremented(self) -> Precedence {
    Precedence(self.0 + 1)
  }
}

impl From<u64> for Precedence {
  fn from(n: u64) -> Precedence {
    Precedence::new(n)
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_left_assoc_precedence() {
    let op = Operator::new("#", Associativity::LEFT, Precedence::new(1));
    assert_eq!(op.left_precedence(), Precedence::from_raw(10));
    assert_eq!(op.right_precedence(), Precedence::from_raw(11));
  }

  #[test]
  fn test_right_assoc_precedence() {
    let op = Operator::new("#", Associativity::RIGHT, Precedence::new(1));
    assert_eq!(op.left_precedence(), Precedence::from_raw(11));
    assert_eq!(op.right_precedence(), Precedence::from_raw(10));
  }

  #[test]
  fn test_full_assoc_precedence() {
    let op = Operator::new("#", Associativity::FULL, Precedence::new(1));
    assert_eq!(op.left_precedence(), Precedence::from_raw(10));
    assert_eq!(op.right_precedence(), Precedence::from_raw(10));
  }

  #[test]
  fn test_none_assoc_precedence() {
    let op = Operator::new("#", Associativity::NONE, Precedence::new(1));
    assert_eq!(op.left_precedence(), Precedence::from_raw(11));
    assert_eq!(op.right_precedence(), Precedence::from_raw(11));
  }
}
