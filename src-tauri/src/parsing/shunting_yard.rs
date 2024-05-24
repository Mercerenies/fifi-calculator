
use super::operator::Operator;
use super::source::Span;

use std::error::{Error as StdError};
use std::fmt::{self, Display, Formatter};

/// A token, for the purposes of the shunting yard algorithm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token<T> {
  pub data: TokenData<T>,
  pub span: Span,
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
  span: Span,
}

/// The content of a token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenData<T> {
  /// A value in the target language.
  Scalar(T),
  /// An infix, binary operator.
  Operator(Operator),
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
  fn compile_bin_op(
    &mut self,
    left: Self::Output,
    operator: Operator,
    right: Self::Output,
  ) -> Result<Self::Output, Self::Error>;
}

impl<T: Display> Display for TokenData<T> {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
    match self {
      TokenData::Scalar(s) => s.fmt(f),
      TokenData::Operator(op) => op.name().fmt(f),
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
      TokenData::Operator(op) => {
        // Pop operators until we hit one with higher precedence.
        while let Some(OpStackValue { operator: stack_op, span }) = operator_stack.pop() {
          if compare_precedence(&stack_op, &op) {
            let token = Token { data: TokenData::Operator(op.clone()), span };
            let error = ShuntingYardError::UnexpectedToken(token);
            simplify_operator(driver, &mut output_stack, stack_op, error)?;
          } else {
            operator_stack.push(OpStackValue { operator: stack_op, span });
            break;
          }
        }
        operator_stack.push(OpStackValue { operator: op, span: token.span });
      }
    }
  }

  // Pop and resolve remaining operators.
  while let Some(op) = operator_stack.pop() {
    simplify_operator(driver, &mut output_stack, op.operator, ShuntingYardError::UnexpectedEOF)?;
  }

  let final_result = output_stack.pop().ok_or(ShuntingYardError::UnexpectedEOF)?;
  if let Some(remaining_value) = output_stack.pop() {
    return Err(ShuntingYardError::UnexpectedToken(remaining_value.token));
  }
  Ok(final_result.output)
}

fn compare_precedence(stack_op: &Operator, current_op: &Operator) -> bool {
  stack_op.precedence() > current_op.precedence() ||
    (stack_op.precedence() == current_op.precedence() && current_op.associativity().is_left_assoc())
}

fn simplify_operator<T, D>(
  driver: &mut D,
  output_stack: &mut Vec<OutputWithToken<T, D::Output>>,
  operator: Operator,
  error: ShuntingYardError<T, D::Error>,
) -> Result<(), ShuntingYardError<T, D::Error>>
where T: Clone,
      D: ShuntingYardDriver<T> {
  let (arg1, arg2) = output_stack.pop()
    .and_then(|arg2| output_stack.pop().map(|arg1| (arg1, arg2)))
    .ok_or(error)?;
  let output = driver.compile_bin_op(arg1.output, operator, arg2.output)?;
  output_stack.push(OutputWithToken { output, token: arg1.token });
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::parsing::source::SourceOffset;
  use crate::parsing::operator::{Precedence, Associativity};

  use std::convert::Infallible;

  /// Basic test "expression" type for our unit tests.
  #[derive(Debug, Clone, PartialEq, Eq)]
  enum TestExpr {
    Scalar(i64),
    BinOp(Box<TestExpr>, String, Box<TestExpr>),
  }

  #[derive(Clone, Debug)]
  struct TestDriver;

  impl TestExpr {
    fn bin_op(left: TestExpr, op: impl Into<String>, right: TestExpr) -> Self {
      Self::BinOp(Box::new(left), op.into(), Box::new(right))
    }
  }

  impl ShuntingYardDriver<i64> for TestDriver {
    type Output = TestExpr;
    type Error = Infallible;

    fn compile_scalar(&mut self, scalar: i64) -> Result<Self::Output, Self::Error> {
      Ok(TestExpr::Scalar(scalar))
    }

    fn compile_bin_op(
      &mut self,
      left: Self::Output,
      op: Operator,
      right: Self::Output,
    ) -> Result<Self::Output, Self::Error> {
      Ok(TestExpr::bin_op(left, op.name(), right))
    }
  }

  fn plus() -> Operator {
    Operator::new("+", Associativity::FULL, Precedence::new(10))
  }

  fn minus() -> Operator {
    Operator::new("-", Associativity::LEFT, Precedence::new(10))
  }

  fn times() -> Operator {
    Operator::new("*", Associativity::FULL, Precedence::new(20))
  }

  fn pow() -> Operator {
    Operator::new("^", Associativity::RIGHT, Precedence::new(30))
  }

  fn span(start: usize, end: usize) -> Span {
    Span::new(SourceOffset(start), SourceOffset(end))
  }

  #[test]
  fn test_full_assoc_op() {
    let tokens = vec![
      Token { data: TokenData::Scalar(1), span: span(0, 1) },
      Token { data: TokenData::Operator(plus()), span: span(1, 2) },
      Token { data: TokenData::Scalar(2), span: span(2, 3) },
      Token { data: TokenData::Operator(plus()), span: span(3, 4) },
      Token { data: TokenData::Scalar(3), span: span(4, 5) },
    ];
    let result = parse(&mut TestDriver, tokens).unwrap();
    assert_eq!(
      TestExpr::bin_op(
        TestExpr::bin_op(
          TestExpr::Scalar(1),
          "+",
          TestExpr::Scalar(2),
        ),
        "+",
        TestExpr::Scalar(3),
      ),
      result,
    );
  }

  #[test]
  fn test_left_assoc_op() {
    let tokens = vec![
      Token { data: TokenData::Scalar(1), span: span(0, 1) },
      Token { data: TokenData::Operator(minus()), span: span(1, 2) },
      Token { data: TokenData::Scalar(2), span: span(2, 3) },
      Token { data: TokenData::Operator(minus()), span: span(3, 4) },
      Token { data: TokenData::Scalar(3), span: span(4, 5) },
    ];
    let result = parse(&mut TestDriver, tokens).unwrap();
    assert_eq!(
      TestExpr::bin_op(
        TestExpr::bin_op(
          TestExpr::Scalar(1),
          "-",
          TestExpr::Scalar(2),
        ),
        "-",
        TestExpr::Scalar(3),
      ),
      result,
    );
  }

  #[test]
  fn test_right_assoc_op() {
    let tokens = vec![
      Token { data: TokenData::Scalar(1), span: span(0, 1) },
      Token { data: TokenData::Operator(pow()), span: span(1, 2) },
      Token { data: TokenData::Scalar(2), span: span(2, 3) },
      Token { data: TokenData::Operator(pow()), span: span(3, 4) },
      Token { data: TokenData::Scalar(3), span: span(4, 5) },
    ];
    let result = parse(&mut TestDriver, tokens).unwrap();
    assert_eq!(
      TestExpr::bin_op(
        TestExpr::Scalar(1),
        "^",
        TestExpr::bin_op(
          TestExpr::Scalar(2),
          "^",
          TestExpr::Scalar(3),
        ),
      ),
      result,
    );
  }

  #[test]
  fn test_differing_assoc_op_higher_on_right() {
    let tokens = vec![
      Token { data: TokenData::Scalar(1), span: span(0, 1) },
      Token { data: TokenData::Operator(plus()), span: span(1, 2) },
      Token { data: TokenData::Scalar(2), span: span(2, 3) },
      Token { data: TokenData::Operator(times()), span: span(3, 4) },
      Token { data: TokenData::Scalar(3), span: span(4, 5) },
    ];
    let result = parse(&mut TestDriver, tokens).unwrap();
    assert_eq!(
      TestExpr::bin_op(
        TestExpr::Scalar(1),
        "+",
        TestExpr::bin_op(
          TestExpr::Scalar(2),
          "*",
          TestExpr::Scalar(3),
        ),
      ),
      result,
    );
  }

  #[test]
  fn test_differing_assoc_op_higher_on_left() {
    let tokens = vec![
      Token { data: TokenData::Scalar(1), span: span(0, 1) },
      Token { data: TokenData::Operator(times()), span: span(1, 2) },
      Token { data: TokenData::Scalar(2), span: span(2, 3) },
      Token { data: TokenData::Operator(plus()), span: span(3, 4) },
      Token { data: TokenData::Scalar(3), span: span(4, 5) },
    ];
    let result = parse(&mut TestDriver, tokens).unwrap();
    assert_eq!(
      TestExpr::bin_op(
        TestExpr::bin_op(
          TestExpr::Scalar(1),
          "*",
          TestExpr::Scalar(2),
        ),
        "+",
        TestExpr::Scalar(3),
      ),
      result,
    );
  }
}
