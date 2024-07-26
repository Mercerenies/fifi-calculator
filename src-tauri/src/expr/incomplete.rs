
use crate::expr::{Expr, TryFromExprError};
use crate::expr::atom::Atom;
use crate::stack::base::RandomAccessStackLike;

use thiserror::Error;

use std::fmt::{self, Display, Formatter};
use std::convert::TryFrom;

/// The calculator defines several singleton objects which are
/// referred to as "incomplete objects". These are transient,
/// temporary objects which exist on the stack for a few moments in
/// order to allow the user to input more complicated complete
/// objects.
///
/// For example, the [`ObjectType::LeftBracket`] incomplete object is
/// used on the stack to input the elements of a vector. Then a
/// command (bound to the `]` key) pops stack elements until
/// [`ObjectType::LeftBracket`] is encountered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncompleteObject {
  object_type: ObjectType,
}

/// The type of incomplete object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectType {
  /// An incomplete object used for inputting vectors.
  LeftBracket,
  /// An incomplete object used for inputting complex numbers, or (as
  /// a corner case) ordinary expressions, if given only one
  /// expression.
  LeftParen,
}

#[derive(Debug, Clone, Error)]
#[error("Invalid object type {input_string}")]
pub struct ObjectTypeParseError {
  input_string: String,
}

#[derive(Debug, Clone, Error)]
#[error("Error parsing Expr as IncompleteObject")]
pub struct TryFromExprRefError {
  _priv: (),
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum PopUntilDelimiterError {
  #[error("Reached end of stack, expecting '{expected}'")]
  UnexpectedEOF { expected: IncompleteObject },
  #[error("Found '{actual}' when expecting '{expected}'")]
  UnexpectedDelimiter { expected: IncompleteObject, actual: IncompleteObject },
}

impl IncompleteObject {
  pub const FUNCTION_NAME: &'static str = "incomplete";

  pub fn new(object_type: ObjectType) -> Self {
    Self { object_type }
  }

  pub fn object_type(&self) -> ObjectType {
    self.object_type
  }

  pub fn name(&self) -> &'static str {
    self.object_type.name()
  }
}

impl ObjectType {
  pub fn name(&self) -> &'static str {
    match self {
      ObjectType::LeftBracket => "[",
      ObjectType::LeftParen => "(",
    }
  }

  pub fn parse(input_string: &str) -> Result<ObjectType, ObjectTypeParseError> {
    match input_string {
      "[" => Ok(ObjectType::LeftBracket),
      "(" => Ok(ObjectType::LeftParen),
      _ => Err(ObjectTypeParseError { input_string: input_string.into() }),
    }
  }
}

pub fn is_incomplete_object(expr: &Expr) -> bool {
  IncompleteObject::try_from(expr).is_ok()
}

/// Pops elements from the stack until the given incomplete object is
/// encountered. Produces an error if there are no incomplete objects
/// on the stack or if an unexpected incomplete object is encountered.
/// In case of error, the stack will be left unmodified.
pub fn pop_until_delimiter<S>(
  target_object: &IncompleteObject,
  stack: &mut S,
) -> Result<Vec<Expr>, PopUntilDelimiterError>
where S: RandomAccessStackLike<Elem = Expr> {
  let first_delimiter_index = (0..stack.len() as i64).into_iter()
    .find(|i| is_incomplete_object(&stack.get(*i).unwrap()));
  match first_delimiter_index {
    None => Err(PopUntilDelimiterError::UnexpectedEOF { expected: target_object.clone() }),
    Some(first_delimiter_index) => {
      let first_delimiter = IncompleteObject::try_from(&*stack.get(first_delimiter_index).unwrap()).unwrap();
      if &first_delimiter == target_object {
        let mut elements = stack.pop_several((first_delimiter_index + 1) as usize).unwrap();
        elements.remove(0); // Do not include the incomplete object itself in the result.
        Ok(elements)
      } else {
        Err(PopUntilDelimiterError::UnexpectedDelimiter { expected: target_object.clone(), actual: first_delimiter })
      }
    }
  }
}

impl Display for ObjectType {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.name())
  }
}

impl Display for IncompleteObject {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "{} ...", self.object_type)
  }
}

impl From<IncompleteObject> for Expr {
  fn from(incomplete_object: IncompleteObject) -> Self {
    Expr::call(IncompleteObject::FUNCTION_NAME, vec![Expr::string(incomplete_object.name())])
  }
}

impl<'a> TryFrom<&'a Expr> for IncompleteObject {
  type Error = TryFromExprRefError;

  fn try_from(expr: &'a Expr) -> Result<Self, Self::Error> {
    let Expr::Call(function_name, args) = expr else {
      return Err(TryFromExprRefError { _priv: () });
    };
    if function_name == IncompleteObject::FUNCTION_NAME && args.len() == 1 && matches!(args[0], Expr::Atom(Atom::String(_))) {
      let Expr::Atom(Atom::String(name)) = &args[0] else { unreachable!() };
      match ObjectType::parse(name) {
        Ok(object_type) => Ok(IncompleteObject::new(object_type)),
        Err(_) => Err(TryFromExprRefError { _priv: () }),
      }
    } else {
      return Err(TryFromExprRefError { _priv: () });
    }
  }
}

impl TryFrom<Expr> for IncompleteObject {
  type Error = TryFromExprError;

  fn try_from(expr: Expr) -> Result<Self, Self::Error> {
    IncompleteObject::try_from(&expr)
      .map_err(|_| TryFromExprError::new("IncompleteObject", expr))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::stack::Stack;

  #[test]
  fn test_to_string() {
    assert_eq!(IncompleteObject::new(ObjectType::LeftBracket).to_string(), "[ ...");
    assert_eq!(IncompleteObject::new(ObjectType::LeftParen).to_string(), "( ...");
  }

  #[test]
  fn test_parse_object_type() {
    assert_eq!(ObjectType::parse("[").unwrap(), ObjectType::LeftBracket);
    assert_eq!(ObjectType::parse("(").unwrap(), ObjectType::LeftParen);
    ObjectType::parse("]").unwrap_err();
    ObjectType::parse("e").unwrap_err();
    ObjectType::parse("").unwrap_err();
    ObjectType::parse("((").unwrap_err();
  }

  #[test]
  fn test_incomplete_object_into_expr() {
    let incomplete_object = IncompleteObject::new(ObjectType::LeftBracket);
    assert_eq!(Expr::from(incomplete_object), Expr::call(IncompleteObject::FUNCTION_NAME, vec![Expr::string("[")]));
    let incomplete_object = IncompleteObject::new(ObjectType::LeftParen);
    assert_eq!(Expr::from(incomplete_object), Expr::call(IncompleteObject::FUNCTION_NAME, vec![Expr::string("(")]));
  }

  #[test]
  fn test_parse_expr_into_incomplete_object() {
    let expr = Expr::call("incomplete", vec![Expr::string("[")]);
    assert_eq!(IncompleteObject::try_from(expr), Ok(IncompleteObject::new(ObjectType::LeftBracket)));
    let expr = Expr::call("incomplete", vec![Expr::string("(")]);
    assert_eq!(IncompleteObject::try_from(expr), Ok(IncompleteObject::new(ObjectType::LeftParen)));
    let expr = Expr::call("incomplete", vec![Expr::string("]")]);
    assert_eq!(IncompleteObject::try_from(expr), Err(TryFromExprError::new("IncompleteObject", Expr::call("incomplete", vec![Expr::string("]")]))));
    let expr = Expr::call("incomplete", vec![Expr::string("e")]);
    assert_eq!(IncompleteObject::try_from(expr), Err(TryFromExprError::new("IncompleteObject", Expr::call("incomplete", vec![Expr::string("e")]))));
    let expr = Expr::call("incomplete", vec![]);
    assert_eq!(IncompleteObject::try_from(expr), Err(TryFromExprError::new("IncompleteObject", Expr::call("incomplete", vec![]))));
    let expr = Expr::string("[");
    assert_eq!(IncompleteObject::try_from(expr), Err(TryFromExprError::new("IncompleteObject", Expr::string("["))));
    let expr = Expr::call("wrong_function_name", vec![Expr::string("(")]);
    assert_eq!(IncompleteObject::try_from(expr), Err(TryFromExprError::new("IncompleteObject", Expr::call("wrong_function_name", vec![Expr::string("(")]))));
    let expr = Expr::call("incomplete", vec![Expr::string("("), Expr::string("(")]);
    assert_eq!(IncompleteObject::try_from(expr), Err(TryFromExprError::new("IncompleteObject", Expr::call("incomplete", vec![Expr::string("("), Expr::string("(")]))));
  }

  #[test]
  fn test_pop_until_delimiter() {
    let mut stack = Stack::from(vec![
      Expr::from(10),
      IncompleteObject::new(ObjectType::LeftBracket).into(),
      Expr::from(20),
      Expr::from(30),
      Expr::from(40),
    ]);
    let elements = pop_until_delimiter(&IncompleteObject::new(ObjectType::LeftBracket), &mut stack).unwrap();
    assert_eq!(elements, vec![Expr::from(20), Expr::from(30), Expr::from(40)]);
    assert_eq!(stack.into_iter().collect::<Vec<_>>(), vec![Expr::from(10)]);
  }

  #[test]
  fn test_pop_until_delimiter_with_no_incomplete_objects_on_stack() {
    let mut stack = Stack::from(vec![
      Expr::from(10),
      Expr::from(20),
      Expr::from(30),
      Expr::from(40),
    ]);
    let original_stack = stack.clone();
    let err = pop_until_delimiter(&IncompleteObject::new(ObjectType::LeftBracket), &mut stack).unwrap_err();
    assert_eq!(err, PopUntilDelimiterError::UnexpectedEOF { expected: IncompleteObject::new(ObjectType::LeftBracket) });
    assert_eq!(stack, original_stack);
  }

  #[test]
  fn test_pop_until_delimiter_with_wrong_delimiter_on_stack() {
    let mut stack = Stack::from(vec![
      Expr::from(10),
      IncompleteObject::new(ObjectType::LeftBracket).into(),
      Expr::from(20),
      IncompleteObject::new(ObjectType::LeftParen).into(),
      Expr::from(30),
      Expr::from(40),
    ]);
    let original_stack = stack.clone();
    let err = pop_until_delimiter(&IncompleteObject::new(ObjectType::LeftBracket), &mut stack).unwrap_err();
    assert_eq!(err, PopUntilDelimiterError::UnexpectedDelimiter { expected: IncompleteObject::new(ObjectType::LeftBracket), actual: IncompleteObject::new(ObjectType::LeftParen) });
    assert_eq!(stack, original_stack);
  }
}
