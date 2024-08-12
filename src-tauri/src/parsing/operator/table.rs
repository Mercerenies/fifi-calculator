
use super::Operator;
use super::fixity::{Fixity, FixityType, FixityTypes};
use super::precedence::Precedence;
use super::associativity::Associativity;

use std::collections::{hash_map, HashMap};

/// A table of operators, indexed by their name.
#[derive(Debug, Clone, Default)]
pub struct OperatorTable {
  by_function_name_prefix: HashMap<String, Operator>,
  by_function_name_infix: HashMap<String, Operator>,
  by_function_name_postfix: HashMap<String, Operator>,
  by_operator_name: HashMap<String, Operator>,
}

#[derive(Debug, Clone)]
pub struct OperatorAmbiguity<'a> {
  left: Vec<&'a Operator>,
  right: Vec<&'a Operator>,
}

/// Precedence used for the caret `^` infix operator. This is provided
/// as a top-level constant, since some language modes render
/// exponents in a special way (such as using superscripts).
pub const EXPONENT_PRECEDENCE: Precedence = Precedence::new(200);

impl OperatorTable {
  pub fn new() -> OperatorTable {
    OperatorTable::default()
  }

  pub fn with_capacity(capacity: usize) -> OperatorTable {
    OperatorTable {
      by_function_name_prefix: HashMap::with_capacity(capacity),
      by_function_name_infix: HashMap::with_capacity(capacity),
      by_function_name_postfix: HashMap::with_capacity(capacity),
      by_operator_name: HashMap::with_capacity(capacity),
    }
  }

  pub fn get_by_operator_name(&self, name: &str) -> Option<&Operator> {
    self.by_operator_name.get(name)
  }

  pub fn get_by_function_name(&self, name: &str, fixity: FixityType) -> Option<&Operator> {
    match fixity {
      FixityType::Prefix => self.by_function_name_prefix.get(name),
      FixityType::Infix => self.by_function_name_infix.get(name),
      FixityType::Postfix => self.by_function_name_postfix.get(name),
    }
  }

  pub fn insert(&mut self, op: Operator) {
    self.by_operator_name.insert(op.operator_name().to_owned(), op.clone());
    if let Some(prefix_props) = op.fixity().as_prefix() {
      self.by_function_name_prefix.insert(prefix_props.function_name().to_owned(), op.clone());
    }
    if let Some(infix_props) = op.fixity().as_infix() {
      self.by_function_name_infix.insert(infix_props.function_name().to_owned(), op.clone());
    }
    if let Some(postfix_props) = op.fixity().as_postfix() {
      self.by_function_name_postfix.insert(postfix_props.function_name().to_owned(), op.clone());
    }
  }

  pub fn common_operators() -> OperatorTable {
    // Note: We borrow the Emacs Calc operator precedence values here
    // when it makes sense to do so. See
    // https://www.gnu.org/software/emacs/manual/html_mono/calc.html#Composition-Basics
    vec![
      Operator::new("..", Fixity::new().with_infix("..", Associativity::NONE, Precedence::new(196))),
      Operator::new("..^", Fixity::new().with_infix("..^", Associativity::NONE, Precedence::new(196))),
      Operator::new("^..", Fixity::new().with_infix("^..", Associativity::NONE, Precedence::new(196))),
      Operator::new("^..^", Fixity::new().with_infix("^..^", Associativity::NONE, Precedence::new(196))),
      Operator::new("^", Fixity::new().with_infix("^", Associativity::RIGHT, EXPONENT_PRECEDENCE)),
      multiplication_operator(),
      Operator::new("@", Fixity::new().with_infix("@", Associativity::LEFT, Precedence::new(195))), // Matrix mul
      Operator::new("/", Fixity::new().with_infix("/", Associativity::LEFT, Precedence::new(190))),
      Operator::new("%", Fixity::new().with_infix("%", Associativity::NONE, Precedence::new(190))),
      Operator::new("+", Fixity::new()
                           .with_infix("+", Associativity::FULL, Precedence::new(180))
                           .with_prefix("identity", Precedence::new(197))),
      Operator::new("-", Fixity::new()
                           .with_infix("-", Associativity::LEFT, Precedence::new(180))
                           .with_prefix("negate", Precedence::new(197))),
      Operator::new("<", Fixity::new().with_infix("<", Associativity::NONE, Precedence::new(160))),
      Operator::new(">", Fixity::new().with_infix(">", Associativity::NONE, Precedence::new(160))),
      Operator::new("=", Fixity::new().with_infix("=", Associativity::NONE, Precedence::new(160))),
      Operator::new("!=", Fixity::new().with_infix("!=", Associativity::NONE, Precedence::new(160))),
      Operator::new("<=", Fixity::new().with_infix("<=", Associativity::NONE, Precedence::new(160))),
      Operator::new(">=", Fixity::new().with_infix(">=", Associativity::NONE, Precedence::new(160))),
      Operator::new("&&", Fixity::new().with_infix("&&", Associativity::FULL, Precedence::new(110))),
      Operator::new("||", Fixity::new().with_infix("||", Associativity::FULL, Precedence::new(100))),

      // Also admit Unicode equivalents to several operators.
      Operator::new("≠", Fixity::new().with_infix("≠", Associativity::NONE, Precedence::new(160))),
      Operator::new("≤", Fixity::new().with_infix("≤", Associativity::NONE, Precedence::new(160))),
      Operator::new("≥", Fixity::new().with_infix("≥", Associativity::NONE, Precedence::new(160))),
    ].into_iter().collect()
  }

  /// Checks the table for operators which can be parsed ambiguously.
  ///
  /// If the operator table contains an operator (#) which is
  /// overloaded as infix and postfix, and another operator (@) which
  /// is overloaded as infix and prefix, then the sequence `t # @ s`
  /// is ambiguous and can be treated in two different ways. In fact,
  /// all operator chain ambiguities can be represented in this way.
  ///
  /// This function performs a check for operators which can cause
  /// this parse error. If such operators are found, the matches are
  /// returned, as an [`OperatorAmbiguity`].
  pub fn check_for_ambiguities(&self) -> Result<(), OperatorAmbiguity> {
    let mut left = Vec::new();
    let mut right = Vec::new();
    let left_conflict = FixityTypes::POSTFIX | FixityTypes::INFIX;
    let right_conflict = FixityTypes::PREFIX | FixityTypes::INFIX;
    for op in self.iter() {
      let fixity_types = op.fixity().fixity_types();
      if fixity_types.contains(left_conflict) {
        left.push(op);
      }
      if fixity_types.contains(right_conflict) {
        right.push(op);
      }
    }
    if !left.is_empty() && !right.is_empty() {
      Err(OperatorAmbiguity { left, right })
    } else {
      Ok(())
    }
  }

  pub fn iter(&self) -> impl Iterator<Item = &Operator> {
    self.by_operator_name.values()
  }
}

impl<'a> OperatorAmbiguity<'a> {
  /// Returns the operators in the table which can be treated as both
  /// postfix and infix. These can appear on the left-hand side of an
  /// ambiguous parse. The returned slice is always non-empty.
  pub fn left(&self) -> &[&'a Operator] {
    &self.left
  }
  /// Returns the operators in the table which can be treated as both
  /// prefix and infix. These can appear on the right-hand side of an
  /// ambiguous parse. The returned slice is always non-empty.
  pub fn right(&self) -> &[&'a Operator] {
    &self.right
  }
}

/// The operator used for multiplication and juxtaposition.
pub fn multiplication_operator() -> Operator {
  Operator::new("*", Fixity::new().with_infix("*", Associativity::FULL, Precedence::new(195)))
}

impl IntoIterator for OperatorTable {
  type Item = Operator;
  type IntoIter = hash_map::IntoValues<String, Operator>;

  fn into_iter(self) -> Self::IntoIter {
    self.by_operator_name.into_values()
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

  fn plus() -> Operator {
    Operator::new(
      "+",
      Fixity::new()
        .with_infix("infix_plus", Associativity::LEFT, Precedence::new(0))
        .with_prefix("prefix_plus", Precedence::new(0))
        .with_postfix("postfix_plus", Precedence::new(0)),
    )
  }

  fn minus() -> Operator {
    Operator::new(
      "-",
      Fixity::new()
        .with_infix("infix_minus", Associativity::LEFT, Precedence::new(0))
        .with_prefix("prefix_minus", Precedence::new(0))
        .with_postfix("postfix_minus", Precedence::new(0)),
    )
  }

  fn sample_table() -> OperatorTable {
    vec![plus(), minus()].into_iter().collect()
  }

  #[test]
  fn test_get_by_operator_name() {
    let table = sample_table();
    assert_eq!(table.get_by_operator_name("+"), Some(&plus()));
    assert_eq!(table.get_by_operator_name("-"), Some(&minus()));
    assert_eq!(table.get_by_operator_name("*"), None);
    assert_eq!(table.get_by_operator_name("infix_plus"), None);
    assert_eq!(table.get_by_operator_name("postfix_minus"), None);
  }

  #[test]
  fn test_get_by_function_name() {
    let table = sample_table();
    assert_eq!(table.get_by_function_name("infix_plus", FixityType::Infix), Some(&plus()));
    assert_eq!(table.get_by_function_name("prefix_plus", FixityType::Prefix), Some(&plus()));
    assert_eq!(table.get_by_function_name("postfix_plus", FixityType::Postfix), Some(&plus()));
    assert_eq!(table.get_by_function_name("infix_minus", FixityType::Infix), Some(&minus()));
    assert_eq!(table.get_by_function_name("prefix_minus", FixityType::Prefix), Some(&minus()));
    assert_eq!(table.get_by_function_name("postfix_minus", FixityType::Postfix), Some(&minus()));
    assert_eq!(table.get_by_function_name("x", FixityType::Infix), None);
    assert_eq!(table.get_by_function_name("", FixityType::Infix), None);
    assert_eq!(table.get_by_function_name("plus", FixityType::Infix), None);
    assert_eq!(table.get_by_function_name("+", FixityType::Infix), None);
    assert_eq!(table.get_by_function_name("*", FixityType::Infix), None);
  }

  #[test]
  fn test_insert() {
    let mut table = sample_table();
    let new_op = Operator::new(
      "&&",
      Fixity::new()
        .with_infix("infix_minus", Associativity::LEFT, Precedence::new(0))
        .with_prefix("XXX", Precedence::new(0)),
    );

    assert_eq!(table.get_by_function_name("infix_plus", FixityType::Infix), Some(&plus()));
    assert_eq!(table.get_by_function_name("infix_minus", FixityType::Infix), Some(&minus()));
    assert_eq!(table.get_by_function_name("XXX", FixityType::Prefix), None);
    assert_eq!(table.get_by_operator_name("&&"), None);

    table.insert(new_op.clone());

    assert_eq!(table.get_by_function_name("infix_plus", FixityType::Infix), Some(&plus()));
    assert_eq!(table.get_by_function_name("infix_minus", FixityType::Infix), Some(&new_op));
    assert_eq!(table.get_by_function_name("XXX", FixityType::Prefix), Some(&new_op));
    assert_eq!(table.get_by_operator_name("&&"), Some(&new_op));
  }

  #[test]
  fn test_ambiguity_check_on_empty_table() {
    let table = OperatorTable::default();
    assert!(table.check_for_ambiguities().is_ok());
  }

  #[test]
  fn test_ambiguity_check_with_no_overloaded_ops() {
    let table: OperatorTable = vec![
      Operator::new("+", Fixity::new().with_infix("+", Associativity::FULL, Precedence::new(0))),
      Operator::new("-", Fixity::new().with_prefix("-", Precedence::new(0))),
      Operator::new("!", Fixity::new().with_postfix("!", Precedence::new(0))),
    ].into_iter().collect();
    assert!(table.check_for_ambiguities().is_ok());
  }

  #[test]
  fn test_ambiguity_check_with_overloaded_pre_and_post_ops() {
    let table: OperatorTable = vec![
      Operator::new("+", Fixity::new().with_infix("+", Associativity::FULL, Precedence::new(0))),
      Operator::new("-", Fixity::new().with_prefix("-", Precedence::new(0)).with_postfix("-", Precedence::new(0))),
      Operator::new("!", Fixity::new().with_postfix("!", Precedence::new(0))),
    ].into_iter().collect();
    assert!(table.check_for_ambiguities().is_ok());
  }

  #[test]
  fn test_ambiguity_check_with_overloaded_pre_and_infix_ops() {
    let table: OperatorTable = vec![
      Operator::new("+", Fixity::new().with_infix("+", Associativity::FULL, Precedence::new(0))),
      Operator::new("-", Fixity::new().with_prefix("-", Precedence::new(0)).with_infix("-", Associativity::FULL, Precedence::new(0))),
      Operator::new("!", Fixity::new().with_postfix("!", Precedence::new(0))),
    ].into_iter().collect();
    assert!(table.check_for_ambiguities().is_ok());
  }

  #[test]
  fn test_ambiguity_check_with_overloaded_post_and_infix_ops() {
    let table: OperatorTable = vec![
      Operator::new("+", Fixity::new().with_infix("+", Associativity::FULL, Precedence::new(0))),
      Operator::new("-", Fixity::new().with_postfix("-", Precedence::new(0)).with_infix("-", Associativity::FULL, Precedence::new(0))),
      Operator::new("!", Fixity::new().with_postfix("!", Precedence::new(0))),
    ].into_iter().collect();
    assert!(table.check_for_ambiguities().is_ok());
  }

  #[test]
  fn test_ambiguity_check_with_conflict() {
    let table: OperatorTable = vec![
      Operator::new("+", Fixity::new().with_infix("+", Associativity::FULL, Precedence::new(0))),
      Operator::new("-", Fixity::new().with_postfix("-", Precedence::new(0)).with_infix("-", Associativity::FULL, Precedence::new(0))),
      Operator::new("!", Fixity::new().with_prefix("!", Precedence::new(0)).with_infix("!", Associativity::FULL, Precedence::new(0))),
    ].into_iter().collect();
    let err = table.check_for_ambiguities().unwrap_err();
    assert_eq!(err.left(), &[&Operator::new("-", Fixity::new().with_postfix("-", Precedence::new(0)).with_infix("-", Associativity::FULL, Precedence::new(0)))]);
    assert_eq!(err.right(), &[&Operator::new("!", Fixity::new().with_prefix("!", Precedence::new(0)).with_infix("!", Associativity::FULL, Precedence::new(0)))]);
  }

  #[test]
  fn test_ambiguity_check_with_triple_overloaded_opertor() {
    let triple = Operator::new("-", Fixity::new().with_postfix("-", Precedence::new(0)).with_infix("-", Associativity::FULL, Precedence::new(0)).with_prefix("-", Precedence::new(0)));
    let table: OperatorTable = vec![
      Operator::new("+", Fixity::new().with_infix("+", Associativity::FULL, Precedence::new(0))),
      triple.clone(),
    ].into_iter().collect();
    let err = table.check_for_ambiguities().unwrap_err();
    assert_eq!(err.left(), &[&triple]);
    assert_eq!(err.right(), &[&triple]);
  }
}
