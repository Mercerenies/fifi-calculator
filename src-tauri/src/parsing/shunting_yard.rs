
use super::operator::{OperWithFixity, TaggedToken,
                      PrefixProperties, PostfixProperties, InfixProperties, FixityType};
use super::source::Spanned;

use std::error::{Error as StdError};
use std::fmt::{self, Display, Formatter};

/// Internal type which tracks an output value together with the first
/// token that produced it. Used to produce better error messages.
#[derive(Debug, Clone)]
struct OutputWithToken<T, O> {
  output: O,
  token: Spanned<TaggedToken<T>>,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ShuntingYardError<T, E: StdError> {
  CustomError(E),
  UnexpectedEOF,
  UnexpectedToken(Spanned<TaggedToken<T>>),
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

impl<T: Display, E: StdError> Display for ShuntingYardError<T, E> {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
    match self {
      ShuntingYardError::CustomError(e) =>
        write!(f, "{}", e),
      ShuntingYardError::UnexpectedEOF =>
        write!(f, "unexpected end of file"),
      ShuntingYardError::UnexpectedToken(t) =>
        write!(f, "unexpected token {} at position {}", t.item, t.span),
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
      I: IntoIterator<Item = Spanned<TaggedToken<T>>> {
  let mut operator_stack: Vec<Spanned<OperWithFixity>> = Vec::new();
  let mut output_stack: Vec<OutputWithToken<T, D::Output>> = Vec::new();
  for token in input {
    // Handle the current token.
    match token.item {
      TaggedToken::Scalar(t) => {
        let output = driver.compile_scalar(t.clone())?;
        let token = Spanned::new(TaggedToken::Scalar(t), token.span);
        output_stack.push(OutputWithToken { output, token });
      }
      TaggedToken::Operator(oper) => {
        let current_value = Spanned::new(oper, token.span);
        pop_and_simplify_while(driver, &mut output_stack, &mut operator_stack, |stack_value| {
          compare_precedence(&stack_value.item, &current_value.item)
        })?;
        operator_stack.push(current_value);
      }
    }
  }

  // Pop and resolve remaining operators.
  pop_and_simplify_while(driver, &mut output_stack, &mut operator_stack, |_| true)?;
  assert!(operator_stack.is_empty(), "Expected no operators left on the stack at end of shunting yard algorithm");

  let final_result = output_stack.pop().ok_or(ShuntingYardError::UnexpectedEOF)?;
  if let Some(remaining_value) = output_stack.pop() {
    return Err(ShuntingYardError::UnexpectedToken(remaining_value.token));
  }
  Ok(final_result.output)
}

fn compare_precedence(stack_value: &OperWithFixity, current_value: &OperWithFixity) -> bool {
  stack_value.precedence() > current_value.precedence() ||
    (stack_value.precedence() == current_value.precedence() && current_value.is_left_assoc())
}

fn pop_and_simplify_while<F, T, D>(
  driver: &mut D,
  output_stack: &mut Vec<OutputWithToken<T, D::Output>>,
  operator_stack: &mut Vec<Spanned<OperWithFixity>>,
  mut continue_condition: F,
) -> Result<(), ShuntingYardError<T, D::Error>>
where T: Clone,
      D: ShuntingYardDriver<T>,
      F: FnMut(&Spanned<OperWithFixity>) -> bool {
  while let Some(stack_value) = operator_stack.pop() {
    if continue_condition(&stack_value) {
      let error = ShuntingYardError::UnexpectedToken(stack_value.clone().map(TaggedToken::from));
      simplify_operator(driver, output_stack, stack_value, error)?;
    } else {
      operator_stack.push(stack_value);
      break;
    }
  }
  Ok(())
}

fn simplify_operator<T, D>(
  driver: &mut D,
  output_stack: &mut Vec<OutputWithToken<T, D::Output>>,
  stack_value: Spanned<OperWithFixity>,
  error: ShuntingYardError<T, D::Error>,
) -> Result<(), ShuntingYardError<T, D::Error>>
where T: Clone,
      D: ShuntingYardDriver<T> {
  let fixity_type = stack_value.item.fixity_type();
  let operator = &stack_value.item.into_operator();
  match fixity_type {
    FixityType::Infix => {
      let (arg1, arg2) = output_stack.pop()
        .and_then(|arg2| output_stack.pop().map(|arg1| (arg1, arg2)))
        .ok_or(error)?;
      let infix_properties = operator.fixity().as_infix().unwrap();
      let output = driver.compile_infix_op(arg1.output, infix_properties, arg2.output)?;
      output_stack.push(OutputWithToken { output, token: arg1.token });
    }
    FixityType::Prefix => {
      let arg = output_stack.pop().ok_or(error)?;
      let prefix_properties = operator.fixity().as_prefix().unwrap();
      let output = driver.compile_prefix_op(prefix_properties, arg.output)?;
      output_stack.push(OutputWithToken { output, token: arg.token });
    }
    FixityType::Postfix => {
      let arg = output_stack.pop().ok_or(error)?;
      let postfix_properties = operator.fixity().as_postfix().unwrap();
      let output = driver.compile_postfix_op(arg.output, postfix_properties)?;
      output_stack.push(OutputWithToken { output, token: arg.token });
    }
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::parsing::source::{SourceOffset, Span};
  use crate::parsing::operator::{Operator, Precedence, Associativity, Fixity};

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

  fn scalar(value: i64, span: Span) -> Spanned<TaggedToken<i64>> {
    Spanned::new(TaggedToken::Scalar(value), span)
  }

  fn infix_operator(op: Operator, span: Span) -> Spanned<TaggedToken<i64>> {
    Spanned::new(TaggedToken::Operator(OperWithFixity::infix(op)), span)
  }

  #[test]
  fn test_full_assoc_op() {
    let tokens = vec![
      scalar(1, span(0, 1)),
      infix_operator(plus(), span(1, 2)),
      scalar(2, span(2, 3)),
      infix_operator(plus(), span(3, 4)),
      scalar(3, span(4, 5)),
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
      scalar(1, span(0, 1)),
      infix_operator(minus(), span(1, 2)),
      scalar(2, span(2, 3)),
      infix_operator(minus(), span(3, 4)),
      scalar(3, span(4, 5)),
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
      scalar(1, span(0, 1)),
      infix_operator(pow(), span(1, 2)),
      scalar(2, span(2, 3)),
      infix_operator(pow(), span(3, 4)),
      scalar(3, span(4, 5)),
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
      scalar(1, span(0, 1)),
      infix_operator(plus(), span(1, 2)),
      scalar(2, span(2, 3)),
      infix_operator(times(), span(3, 4)),
      scalar(3, span(4, 5)),
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
      scalar(1, span(0, 1)),
      infix_operator(times(), span(1, 2)),
      scalar(2, span(2, 3)),
      infix_operator(plus(), span(3, 4)),
      scalar(3, span(4, 5)),
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
