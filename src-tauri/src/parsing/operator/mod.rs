
mod precedence;
mod associativity;
mod fixity;
mod table;

pub use precedence::Precedence;
pub use associativity::Associativity;
pub use fixity::{Fixity, EmptyFixity};
pub use table::OperatorTable;

/// An operator has a precedence and an associativity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Operator {
  function_name: String,
  display_name: String,
  assoc: Associativity,
  prec: Precedence,
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
