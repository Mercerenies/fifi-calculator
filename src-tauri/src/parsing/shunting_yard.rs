
use super::operator::Operator;
use super::source::{Span, SourceOffset};

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
enum OpStackValue {
  Operator(Operator, Span),
  OpenParen(Option<String>, usize, Span),
}

/// The content of a token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenData<T> {
  /// A value in the target language.
  Scalar(T),
  /// An infix, binary operator.
  Operator(Operator),
  /// A comma, separating a parenthesized expression or function call
  /// arguments.
  Comma,
  /// An opening parenthesis literal, optionally associated with a
  /// function name.
  OpenParen(Option<String>),
  /// A close paren literal.
  CloseParen,
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
  fn compile_function_call(
    &mut self,
    function_name: Option<String>,
    args: Vec<Self::Output>,
  ) -> Result<Self::Output, Self::Error>;
}

impl OpStackValue {
  fn span(&self) -> Span {
    match self {
      OpStackValue::Operator(_, s) | OpStackValue::OpenParen(_, _, s) => *s,
    }
  }
}

impl<T: Display> Display for TokenData<T> {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
    match self {
      TokenData::Scalar(s) => s.fmt(f),
      TokenData::Operator(op) => op.name().fmt(f),
      TokenData::Comma => write!(f, ","),
      TokenData::OpenParen(name) => write!(f, "{}(", name.as_deref().unwrap_or("")),
      TokenData::CloseParen => write!(f, ")"),
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
        // Loop until we hit an open paren, the bottom, or an operator
        // of higher precedence.
        loop {
          match operator_stack.pop() {
            None => {
              break;
            }
            Some(OpStackValue::OpenParen(f, arity, span)) => {
              operator_stack.push(OpStackValue::OpenParen(f, arity, span));
              break;
            }
            Some(OpStackValue::Operator(stack_op, span)) => {
              if compare_precedence(&stack_op, &op) {
                let token = Token { data: TokenData::Operator(op.clone()), span };
                let error = ShuntingYardError::UnexpectedToken(token);
                simplify_operator(driver, &mut output_stack, stack_op, error)?;
              } else {
                operator_stack.push(OpStackValue::Operator(stack_op, span));
                break;
              }
            }
          }
        }
        operator_stack.push(OpStackValue::Operator(op, token.span));
      }
      TokenData::Comma => {
        // Loop until we find an open parenthesis, and simplify
        // operators.
        loop {
          match operator_stack.pop() {
            None => {
              let token = Token { data: TokenData::Comma, span: token.span };
              return Err(ShuntingYardError::UnexpectedToken(token));
            }
            Some(OpStackValue::OpenParen(f, arity, span)) => {
              // We've found the open paren; put it back and move on.
              operator_stack.push(OpStackValue::OpenParen(f, arity + 1, span));
              break;
            }
            Some(OpStackValue::Operator(op, _)) => {
              let token = Token { data: TokenData::Comma, span: token.span };
              let error = ShuntingYardError::UnexpectedToken(token);
              simplify_operator(driver, &mut output_stack, op, error)?;
            }
          }
        }
      }
      TokenData::OpenParen(function_name) => {
        operator_stack.push(OpStackValue::OpenParen(function_name, 0, token.span));
      }
      TokenData::CloseParen => {
        // Loop until we find an open parenthesis.
        todo!();
      }
    }
  }

  // Pop and resolve remaining operators.
  todo!();

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
