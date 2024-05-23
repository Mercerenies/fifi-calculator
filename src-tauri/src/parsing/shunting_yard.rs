
use super::operator::Operator;
use super::source::Span;

use std::error::Error;

/// A token, for the purposes of the shunting yard algorithm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token<T> {
  pub data: TokenData<T>,
  pub span: Span,
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
pub enum ShuntingYardError<E: Error> {
  CustomError(E),
}

/// A type implementing this trait is capable of driving the shunting
/// yard algorithm and compiling tokens to a given target language.
pub trait ShuntingYardDriver<T> {
  type Output;
  type Error;

  fn compile_scalar(&mut self, scalar: T) -> Result<Self::Output, Self::Error>;
  fn compile_bin_op(
    &mut self,
    left: T,
    operator: Operator,
    right: T,
  ) -> Result<Self::Output, Self::Error>;
  fn compile_function_call(
    &mut self,
    function_name: Option<String>,
    args: Vec<T>,
  ) -> Result<Self::Output, Self::Error>;
}
