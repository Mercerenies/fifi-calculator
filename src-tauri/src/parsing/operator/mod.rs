
mod precedence;
mod associativity;
mod fixity;
mod table;

pub use precedence::Precedence;
pub use associativity::Associativity;
pub use fixity::{Fixity, FixityTypes, FixityType, EmptyFixity,
                 InfixProperties, PrefixProperties, PostfixProperties};
pub use table::OperatorTable;

use std::fmt::{self, Formatter, Display};
use std::error::{Error as StdError};

/// An operator has a precedence and an associativity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Operator {
  operator_name: String,
  fixity: Fixity,
}

/// An operator, together with the fixity it's currently being used
/// as. An `OperWithFixity` is always guaranteed to be coherent, in
/// the sense that the operator will always support the used fixity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperWithFixity {
  operator: Operator,
  fixity_type: FixityType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperWithFixityError {
  expected_fixity: FixityType,
  operator: Operator,
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

impl OperWithFixity {
  /// Constructs an `OperWithFixity` for the given operator and fixity
  /// type. Returns `None` if the operator cannot be used with the
  /// given fixity.
  pub fn try_new(operator: Operator, fixity_type: FixityType) -> Result<Self, OperWithFixityError> {
    match fixity_type {
      FixityType::Prefix => {
        if !operator.fixity().is_prefix() {
          return Err(OperWithFixityError { expected_fixity: FixityType::Prefix, operator });
        }
        Ok(OperWithFixity { operator, fixity_type })
      }
      FixityType::Infix => {
        if !operator.fixity().is_infix() {
          return Err(OperWithFixityError { expected_fixity: FixityType::Infix, operator });
        }
        Ok(OperWithFixity { operator, fixity_type })
      }
      FixityType::Postfix => {
        if !operator.fixity().is_postfix() {
          return Err(OperWithFixityError { expected_fixity: FixityType::Postfix, operator });
        }
        Ok(OperWithFixity { operator, fixity_type })
      }
    }
  }

  /// Constructs an `OperWithFixity` for the given operator and fixity
  /// type. Panics if the operator cannot be used with the given
  /// fixity. See [`try_new`](OperWithFixity::try_new) for a
  /// non-panicking variant.
  pub fn new(operator: Operator, fixity_type: FixityType) -> Self {
    Self::try_new(operator, fixity_type)
      .unwrap_or_else(|err| panic!("{}", err))
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
}

impl Display for OperWithFixityError {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "Expected fixity {:?} for operator {:?}", self.expected_fixity, self.operator)
  }
}

impl StdError for OperWithFixityError {}

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

    let op_with_fixity = OperWithFixity::try_new(op.clone(), FixityType::Prefix).unwrap();
    assert_eq!(op_with_fixity.fixity_type(), FixityType::Prefix);
    assert_eq!(op_with_fixity.operator(), &op);

    let op_with_fixity = OperWithFixity::try_new(op.clone(), FixityType::Infix).unwrap();
    assert_eq!(op_with_fixity.fixity_type(), FixityType::Infix);
    assert_eq!(op_with_fixity.operator(), &op);

    let err = OperWithFixity::try_new(op.clone(), FixityType::Postfix).unwrap_err();
    assert_eq!(
      err,
      OperWithFixityError { operator: op.clone(), expected_fixity: FixityType::Postfix },
    );
  }

  #[test]
  fn test_construct_operator_with_fixity_panicking_variant() {
    let op = Operator::new(
      "#",
      Fixity::new()
        .with_prefix("#", Precedence::new(1))
        .with_infix("#", Associativity::LEFT, Precedence::new(2)),
    );

    let op_with_fixity = OperWithFixity::new(op.clone(), FixityType::Prefix);
    assert_eq!(op_with_fixity.fixity_type(), FixityType::Prefix);
    assert_eq!(op_with_fixity.operator(), &op);

    let op_with_fixity = OperWithFixity::new(op.clone(), FixityType::Infix);
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
    OperWithFixity::new(op.clone(), FixityType::Postfix);
  }
}
