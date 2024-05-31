
mod associativity;
pub mod chain;
pub mod fixity;
mod precedence;
pub mod table;

pub use precedence::Precedence;
pub use associativity::Associativity;
use fixity::{Fixity, FixityType};
pub use table::OperatorTable;

use std::fmt::{self, Formatter, Display};
use std::error::{Error as StdError};
use std::sync::Arc;

/// An operator has a precedence and an associativity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Operator {
  // Fixity is actually quite a large structure, and we store
  // Operators in all kinds of places, including error objects, token
  // objects, etc. So I want Operator to be relatively cheap-to-copy
  // and not take up a ton of space in its enclosing structure. Hide
  // it behind an Arc. We have to use Arc (not Rc) so that
  // OperatorTable can be passed in the Tauri global application
  // state.
  data: Arc<OperatorImpl>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OperatorImpl {
  operator_name: String,
  fixity: Fixity,
}

/// An operator, together with the fixity it's currently being used
/// as. An `TaggedOperator` is always guaranteed to be coherent, in
/// the sense that the operator will always support the used fixity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaggedOperator {
  operator: Operator,
  fixity_type: FixityType,
}

#[derive(Debug, Clone)]
pub struct TaggedOperatorError {
  expected_fixity: FixityType,
  operator: Operator,
}

impl Operator {
  /// Constructs a new operator with the given properties.
  pub fn new(name: impl Into<String>, fixity: Fixity) -> Self {
    let data = OperatorImpl {
      operator_name: name.into(),
      fixity,
    };
    Operator { data: Arc::new(data) }
  }

  /// The name of the operator, as displayed to the user.
  pub fn operator_name(&self) -> &str {
    &self.data.operator_name
  }

  pub fn fixity(&self) -> &Fixity {
    &self.data.fixity
  }

  pub fn function_names(&self) -> impl Iterator<Item = &str> {
    vec![
      self.data.fixity.as_prefix().map(|props| props.function_name()),
      self.data.fixity.as_infix().map(|props| props.function_name()),
      self.data.fixity.as_postfix().map(|props| props.function_name()),
    ].into_iter().flatten()
  }
}

impl TaggedOperator {
  /// Constructs an `TaggedOperator` for the given operator and fixity
  /// type. Returns `None` if the operator cannot be used with the
  /// given fixity.
  pub fn try_new(operator: Operator, fixity_type: FixityType) -> Result<Self, TaggedOperatorError> {
    if !operator.fixity().supports(fixity_type) {
      return Err(TaggedOperatorError { expected_fixity: fixity_type, operator });
    }
    Ok(TaggedOperator { operator, fixity_type })
  }

  /// Constructs an `TaggedOperator` for the given operator and fixity
  /// type. Panics if the operator cannot be used with the given
  /// fixity. See [`try_new`](TaggedOperator::try_new) for a
  /// non-panicking variant.
  pub fn new(operator: Operator, fixity_type: FixityType) -> Self {
    Self::try_new(operator, fixity_type)
      .unwrap_or_else(|err| panic!("{}", err))
  }

  /// Returns an operator being used in infix form. Panics if the
  /// operator is not infix.
  pub fn infix(operator: Operator) -> Self {
    Self::new(operator, FixityType::Infix)
  }

  /// Returns an operator being used in postfix form. Panics if the
  /// operator is not postfix.
  pub fn postfix(operator: Operator) -> Self {
    Self::new(operator, FixityType::Postfix)
  }

  /// Returns an operator being used in prefix form. Panics if the
  /// operator is not prefix.
  pub fn prefix(operator: Operator) -> Self {
    Self::new(operator, FixityType::Prefix)
  }

  pub fn operator(&self) -> &Operator {
    &self.operator
  }

  pub fn fixity_type(&self) -> FixityType {
    self.fixity_type
  }

  pub fn into_operator(self) -> Operator {
    self.operator
  }

  pub fn precedence(&self) -> Precedence {
    // unwrap safety: Our constructors guarantee that the operator is
    // always good for the given fixity type.
    match self.fixity_type {
      FixityType::Prefix => self.operator.fixity().as_prefix().unwrap().precedence(),
      FixityType::Infix => self.operator.fixity().as_infix().unwrap().precedence(),
      FixityType::Postfix => self.operator.fixity().as_postfix().unwrap().precedence(),
    }
  }

  pub fn is_left_assoc(&self) -> bool {
    // unwrap safety: Our constructors guarantee that the operator is
    // always good for the given fixity type.
    match self.fixity_type {
      FixityType::Infix => self.operator.fixity().as_infix().unwrap().associativity().is_left_assoc(),
      FixityType::Prefix => false,
      FixityType::Postfix => true,
    }
  }

  pub fn is_right_assoc(&self) -> bool {
    // unwrap safety: Our constructors guarantee that the operator is
    // always good for the given fixity type.
    match self.fixity_type {
      FixityType::Infix => self.operator.fixity().as_infix().unwrap().associativity().is_right_assoc(),
      FixityType::Prefix => true,
      FixityType::Postfix => false,
    }
  }
}

impl Display for Operator {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.operator_name())
  }
}

impl Display for TaggedOperator {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self.fixity_type {
      FixityType::Infix => write!(f, "_{}_", self.operator),
      FixityType::Prefix => write!(f, "{}_", self.operator),
      FixityType::Postfix => write!(f, "_{}", self.operator),
    }
  }
}

impl Display for TaggedOperatorError {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "Expected fixity {:?} for operator {:?}", self.expected_fixity, self.operator)
  }
}

impl StdError for TaggedOperatorError {}

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

  #[test]
  fn test_construct_operator_with_fixity() {
    let op = Operator::new(
      "#",
      Fixity::new()
        .with_prefix("#", Precedence::new(1))
        .with_infix("#", Associativity::LEFT, Precedence::new(2)),
    );

    let op_with_fixity = TaggedOperator::try_new(op.clone(), FixityType::Prefix).unwrap();
    assert_eq!(op_with_fixity.fixity_type(), FixityType::Prefix);
    assert_eq!(op_with_fixity.operator(), &op);

    let op_with_fixity = TaggedOperator::try_new(op.clone(), FixityType::Infix).unwrap();
    assert_eq!(op_with_fixity.fixity_type(), FixityType::Infix);
    assert_eq!(op_with_fixity.operator(), &op);

    let err = TaggedOperator::try_new(op.clone(), FixityType::Postfix).unwrap_err();
    assert_eq!(&err.operator, &op);
    assert_eq!(err.expected_fixity, FixityType::Postfix);
  }

  #[test]
  fn test_construct_operator_with_fixity_panicking_variant() {
    let op = Operator::new(
      "#",
      Fixity::new()
        .with_prefix("#", Precedence::new(1))
        .with_infix("#", Associativity::LEFT, Precedence::new(2)),
    );

    let op_with_fixity = TaggedOperator::new(op.clone(), FixityType::Prefix);
    assert_eq!(op_with_fixity.fixity_type(), FixityType::Prefix);
    assert_eq!(op_with_fixity.operator(), &op);

    let op_with_fixity = TaggedOperator::new(op.clone(), FixityType::Infix);
    assert_eq!(op_with_fixity.fixity_type(), FixityType::Infix);
    assert_eq!(op_with_fixity.operator(), &op);
  }

  #[test]
  #[should_panic]
  fn test_construct_operator_with_fixity_panicking_variant_with_invalid_fixity() {
    let op = Operator::new(
      "#",
      Fixity::new()
        .with_prefix("#", Precedence::new(1))
        .with_infix("#", Associativity::LEFT, Precedence::new(2)),
    );
    TaggedOperator::new(op.clone(), FixityType::Postfix);
  }
}
