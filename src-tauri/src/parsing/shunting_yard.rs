
use super::operator::{Operator, PrefixProperties, PostfixProperties, InfixProperties, FixityType};
use super::source::Span;

use std::error::{Error as StdError};
use std::fmt::{self, Display, Formatter};

/// A token, for the purposes of the shunting yard algorithm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token<T> {
  data: TokenData<T>,
  span: Span,
}

/// Internal type which tracks an output value together with the first
/// token that produced it. Used to produce better error messages.
#[derive(Debug, Clone)]
struct OutputWithToken<T, O> {
  output: O,
  token: Token<T>,
}

#[derive(Clone, Debug)]
struct OpStackValue {
  operator: Operator,
  fixity: FixityType,
  span: Span,
}

/// The contents of a token.
#[derive(Debug, Clone, PartialEq, Eq)]
enum TokenData<T> {
  /// A value in the target language.
  Scalar(T),
  /// An infix, binary operator.
  InfixOperator(Operator),
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ShuntingYardError<T, E: StdError> {
  CustomError(E),
  UnexpectedEOF,
  UnexpectedToken(Token<T>),
}

/// A type implementing this trait is capable of driving the shunting
/// yard algorithm and compiling tokens to a given target language.
pub trait ShuntingYardDriver<T> {
  type Output;
  type Error: StdError;

  fn compile_scalar(&mut self, scalar: T) -> Result<Self::Output, Self::Error>;
  fn compile_infix_op(
    &mut self,
    left: Self::Output,
    infix: &InfixProperties,
    right: Self::Output,
  ) -> Result<Self::Output, Self::Error>;
  fn compile_prefix_op(
    &mut self,
    prefix: &PrefixProperties,
    right: Self::Output,
  ) -> Result<Self::Output, Self::Error>;
  fn compile_postfix_op(
    &mut self,
    left: Self::Output,
    postfix: &PostfixProperties,
  ) -> Result<Self::Output, Self::Error>;
}

impl<T> Token<T> {

  pub fn scalar(data: T, span: Span) -> Self {
    Self { data: TokenData::Scalar(data), span }
  }

  /// Constructs a token representing an infix operator. Panics if
  /// `op` is not an infix operator.
  pub fn infix_operator(op: Operator, span: Span) -> Self {
    if !op.fixity().is_infix() {
      panic!("Token::infix_operator requires an infix operator, got {:?}", op);
    }
    Self { data: TokenData::InfixOperator(op), span }
  }

  pub fn span(&self) -> Span {
    self.span
  }
}

impl<T: Display> Display for TokenData<T> {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
    match self {
      TokenData::Scalar(s) => s.fmt(f),
      TokenData::InfixOperator(op) => op.operator_name().fmt(f),
    }
  }
}

impl<T: Display> Display for Token<T> {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
    write!(f, "{}", self.data)
  }
}

impl<T: Display, E: StdError> Display for ShuntingYardError<T, E> {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
    match self {
      ShuntingYardError::CustomError(e) =>
        write!(f, "{}", e),
      ShuntingYardError::UnexpectedEOF =>
        write!(f, "unexpected end of file"),
      ShuntingYardError::UnexpectedToken(t) =>
        write!(f, "unexpected token {} at position {}", t.data, t.span),
    }
  }
}

impl<T, E> StdError for ShuntingYardError<T, E>
where T: Display + fmt::Debug,
      E: StdError + 'static {
  fn source(&self) -> Option<&(dyn StdError + 'static)> {
    match self {
      ShuntingYardError::CustomError(e) => Some(e),
      ShuntingYardError::UnexpectedEOF => None,
      ShuntingYardError::UnexpectedToken(_) => None,
    }
  }
}

impl<T, E: StdError> From<E> for ShuntingYardError<T, E> {
  fn from(e: E) -> Self {
    Self::CustomError(e)
  }
}

pub fn parse<T, D, I>(
  driver: &mut D,
  input: I
) -> Result<D::Output, ShuntingYardError<T, D::Error>>
where T: Clone,
      D: ShuntingYardDriver<T>,
      I: IntoIterator<Item = Token<T>> {
  let mut operator_stack: Vec<OpStackValue> = Vec::new();
  let mut output_stack: Vec<OutputWithToken<T, D::Output>> = Vec::new();
  for token in input {
    // Handle the current token.
    match token.data {
      TokenData::Scalar(t) => {
        let output = driver.compile_scalar(t.clone())?;
        let token = Token { data: TokenData::Scalar(t), span: token.span };
        output_stack.push(OutputWithToken { output, token });
      }
      TokenData::InfixOperator(op) => {
        // Pop operators until we hit one with higher precedence.
        while let Some(stack_value) = operator_stack.pop() {
          if compare_precedence(&stack_value.operator, &op) {
            let token = Token {
              data: TokenData::InfixOperator(stack_value.operator.clone()),
              span: stack_value.span,
            };
            let error = ShuntingYardError::UnexpectedToken(token);
            simplify_operator(driver, &mut output_stack, stack_value, error)?;
          } else {
            operator_stack.push(stack_value);
            break;
          }
        }
        operator_stack.push(OpStackValue { operator: op, span: token.span, fixity: FixityType::Infix });
      }
    }
  }

  // Pop and resolve remaining operators.
  while let Some(stack_value) = operator_stack.pop() {
    simplify_operator(driver, &mut output_stack, stack_value, ShuntingYardError::UnexpectedEOF)?;
  }

  let final_result = output_stack.pop().ok_or(ShuntingYardError::UnexpectedEOF)?;
  if let Some(remaining_value) = output_stack.pop() {
    return Err(ShuntingYardError::UnexpectedToken(remaining_value.token));
  }
  Ok(final_result.output)
}

fn compare_precedence(stack_op: &Operator, current_op: &Operator) -> bool {
  // TODO: Currently we assume infix here, but we'll have to work on that soon.
  let stack_op = stack_op.fixity().as_infix().unwrap();
  let current_op = current_op.fixity().as_infix().unwrap();
  stack_op.precedence() > current_op.precedence() ||
    (stack_op.precedence() == current_op.precedence() && current_op.associativity().is_left_assoc())
}

fn simplify_operator<T, D>(
  driver: &mut D,
  output_stack: &mut Vec<OutputWithToken<T, D::Output>>,
  stack_value: OpStackValue,
  error: ShuntingYardError<T, D::Error>,
) -> Result<(), ShuntingYardError<T, D::Error>>
where T: Clone,
      D: ShuntingYardDriver<T> {
  let operator = &stack_value.operator;
  match stack_value.fixity {
    FixityType::Infix => {
      let (arg1, arg2) = output_stack.pop()
        .and_then(|arg2| output_stack.pop().map(|arg1| (arg1, arg2)))
        .ok_or(error)?;
      let infix_properties = operator.fixity().as_infix().unwrap();
      let output = driver.compile_infix_op(arg1.output, infix_properties, arg2.output)?;
      output_stack.push(OutputWithToken { output, token: arg1.token });
    }
    FixityType::Prefix => {
      todo!()
    }
    FixityType::Postfix => {
      todo!()
    }
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::parsing::source::SourceOffset;
  use crate::parsing::operator::{Precedence, Associativity, Fixity};

  use std::convert::Infallible;

  /// Basic test "expression" type for our unit tests.
  #[derive(Debug, Clone, PartialEq, Eq)]
  enum TestExpr {
    Scalar(i64),
    InfixOp(Box<TestExpr>, String, Box<TestExpr>),
  }

  #[derive(Clone, Debug)]
  struct TestDriver;

  impl TestExpr {
    fn infix_op(left: TestExpr, op: impl Into<String>, right: TestExpr) -> Self {
      Self::InfixOp(Box::new(left), op.into(), Box::new(right))
    }
  }

  impl ShuntingYardDriver<i64> for TestDriver {
    type Output = TestExpr;
    type Error = Infallible;

    fn compile_scalar(&mut self, scalar: i64) -> Result<Self::Output, Self::Error> {
      Ok(TestExpr::Scalar(scalar))
    }

    fn compile_infix_op(
      &mut self,
      left: Self::Output,
      op: &InfixProperties,
      right: Self::Output,
    ) -> Result<Self::Output, Self::Error> {
      Ok(TestExpr::infix_op(left, op.function_name(), right))
    }

    fn compile_prefix_op(
      &mut self,
      _op: &PrefixProperties,
      _right: Self::Output,
    ) -> Result<Self::Output, Self::Error> {
      panic!("Unused in tests, currently");
    }

    fn compile_postfix_op(
      &mut self,
      _left: Self::Output,
      _op: &PostfixProperties,
    ) -> Result<Self::Output, Self::Error> {
      panic!("Unused in tests, currently");
    }
  }

  fn plus() -> Operator {
    Operator::new("+", Fixity::new().with_infix("plus", Associativity::FULL, Precedence::new(10)))
  }

  fn minus() -> Operator {
    Operator::new("-", Fixity::new().with_infix("minus", Associativity::LEFT, Precedence::new(10)))
  }

  fn times() -> Operator {
    Operator::new("*", Fixity::new().with_infix("times", Associativity::FULL, Precedence::new(20)))
  }

  fn pow() -> Operator {
    Operator::new("^", Fixity::new().with_infix("pow", Associativity::RIGHT, Precedence::new(30)))
  }

  fn span(start: usize, end: usize) -> Span {
    Span::new(SourceOffset(start), SourceOffset(end))
  }

  #[test]
  fn test_full_assoc_op() {
    let tokens = vec![
      Token::scalar(1, span(0, 1)),
      Token::infix_operator(plus(), span(1, 2)),
      Token::scalar(2, span(2, 3)),
      Token::infix_operator(plus(), span(3, 4)),
      Token::scalar(3, span(4, 5)),
    ];
    let result = parse(&mut TestDriver, tokens).unwrap();
    assert_eq!(
      TestExpr::infix_op(
        TestExpr::infix_op(
          TestExpr::Scalar(1),
          "plus",
          TestExpr::Scalar(2),
        ),
        "plus",
        TestExpr::Scalar(3),
      ),
      result,
    );
  }

  #[test]
  fn test_left_assoc_op() {
    let tokens = vec![
      Token::scalar(1, span(0, 1)),
      Token::infix_operator(minus(), span(1, 2)),
      Token::scalar(2, span(2, 3)),
      Token::infix_operator(minus(), span(3, 4)),
      Token::scalar(3, span(4, 5)),
    ];
    let result = parse(&mut TestDriver, tokens).unwrap();
    assert_eq!(
      TestExpr::infix_op(
        TestExpr::infix_op(
          TestExpr::Scalar(1),
          "minus",
          TestExpr::Scalar(2),
        ),
        "minus",
        TestExpr::Scalar(3),
      ),
      result,
    );
  }

  #[test]
  fn test_right_assoc_op() {
    let tokens = vec![
      Token::scalar(1, span(0, 1)),
      Token::infix_operator(pow(), span(1, 2)),
      Token::scalar(2, span(2, 3)),
      Token::infix_operator(pow(), span(3, 4)),
      Token::scalar(3, span(4, 5)),
    ];
    let result = parse(&mut TestDriver, tokens).unwrap();
    assert_eq!(
      TestExpr::infix_op(
        TestExpr::Scalar(1),
        "pow",
        TestExpr::infix_op(
          TestExpr::Scalar(2),
          "pow",
          TestExpr::Scalar(3),
        ),
      ),
      result,
    );
  }

  #[test]
  fn test_differing_assoc_op_higher_on_right() {
    let tokens = vec![
      Token::scalar(1, span(0, 1)),
      Token::infix_operator(plus(), span(1, 2)),
      Token::scalar(2, span(2, 3)),
      Token::infix_operator(times(), span(3, 4)),
      Token::scalar(3, span(4, 5)),
    ];
    let result = parse(&mut TestDriver, tokens).unwrap();
    assert_eq!(
      TestExpr::infix_op(
        TestExpr::Scalar(1),
        "plus",
        TestExpr::infix_op(
          TestExpr::Scalar(2),
          "times",
          TestExpr::Scalar(3),
        ),
      ),
      result,
    );
  }

  #[test]
  fn test_differing_assoc_op_higher_on_left() {
    let tokens = vec![
      Token::scalar(1, span(0, 1)),
      Token::infix_operator(times(), span(1, 2)),
      Token::scalar(2, span(2, 3)),
      Token::infix_operator(plus(), span(3, 4)),
      Token::scalar(3, span(4, 5)),
    ];
    let result = parse(&mut TestDriver, tokens).unwrap();
    assert_eq!(
      TestExpr::infix_op(
        TestExpr::infix_op(
          TestExpr::Scalar(1),
          "times",
          TestExpr::Scalar(2),
        ),
        "plus",
        TestExpr::Scalar(3),
      ),
      result,
    );
  }
}
