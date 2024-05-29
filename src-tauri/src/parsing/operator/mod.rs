
mod precedence;
mod associativity;
mod fixity;
mod table;

pub use precedence::Precedence;
pub use associativity::Associativity;
pub use fixity::{Fixity, FixityTypes, EmptyFixity, InfixProperties, PrefixProperties, PostfixProperties};
pub use table::OperatorTable;

/// An operator has a precedence and an associativity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Operator {
  operator_name: String,
  fixity: Fixity,
}

impl Operator {
  /// Constructs a new operator with the given properties.
  pub fn new(name: impl Into<String>, fixity: Fixity) -> Self {
    Operator {
      operator_name: name.into(),
      fixity,
    }
  }

  /// The name of the operator, as displayed to the user.
  pub fn operator_name(&self) -> &str {
    &self.operator_name
  }

  pub fn fixity(&self) -> &Fixity {
    &self.fixity
  }

  pub fn function_names(&self) -> impl Iterator<Item = &str> {
    vec![
      self.fixity.as_prefix().map(|props| props.function_name()),
      self.fixity.as_infix().map(|props| props.function_name()),
      self.fixity.as_postfix().map(|props| props.function_name()),
    ].into_iter().flatten()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_left_assoc_precedence() {
    let op = Operator::new("#", Fixity::new().with_infix("#", Associativity::LEFT, Precedence::new(1)));
    assert_eq!(op.fixity().as_infix().unwrap().left_precedence(), Precedence::from_raw(10));
    assert_eq!(op.fixity().as_infix().unwrap().right_precedence(), Precedence::from_raw(11));
  }

  #[test]
  fn test_right_assoc_precedence() {
    let op = Operator::new("#", Fixity::new().with_infix("#", Associativity::RIGHT, Precedence::new(1)));
    assert_eq!(op.fixity().as_infix().unwrap().left_precedence(), Precedence::from_raw(11));
    assert_eq!(op.fixity().as_infix().unwrap().right_precedence(), Precedence::from_raw(10));
  }

  #[test]
  fn test_full_assoc_precedence() {
    let op = Operator::new("#", Fixity::new().with_infix("#", Associativity::FULL, Precedence::new(1)));
    assert_eq!(op.fixity().as_infix().unwrap().left_precedence(), Precedence::from_raw(10));
    assert_eq!(op.fixity().as_infix().unwrap().right_precedence(), Precedence::from_raw(10));
  }

  #[test]
  fn test_none_assoc_precedence() {
    let op = Operator::new("#", Fixity::new().with_infix("#", Associativity::NONE, Precedence::new(1)));
    assert_eq!(op.fixity().as_infix().unwrap().left_precedence(), Precedence::from_raw(11));
    assert_eq!(op.fixity().as_infix().unwrap().right_precedence(), Precedence::from_raw(11));
  }
}
