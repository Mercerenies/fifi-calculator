
//! Basic parsing language for the [basic language
//! mode](crate::display::basic::BasicLanguageMode].

use super::Expr;
use super::number::ComplexNumber;
use super::tokenizer::{ExprTokenizer, Token, TokenData, TokenizerError};
use crate::parsing::shunting_yard::{self, ShuntingYardDriver, ShuntingYardError};
use crate::parsing::operator::{Operator, OperatorTable};
use crate::parsing::operator::chain::{self, Token as ChainToken, TaggedToken, OperatorChainError};
use crate::parsing::operator::fixity::{InfixProperties, PrefixProperties, PostfixProperties};
use crate::parsing::source::{Span, Spanned, SourceOffset};
use crate::parsing::tokenizer::TokenizerState;

use thiserror::Error;

use std::convert::Infallible;

#[derive(Clone, Debug)]
pub struct ExprParser<'a> {
  tokenizer: ExprTokenizer<'a>,
}

#[derive(Clone, Debug, Error)]
pub enum ParseError {
  #[error("Tokenizer error: {0}")]
  TokenizerError(#[from] TokenizerError),
  #[error("Parsing error: {0}")]
  ParsingError(#[from] ParsingError),
  #[error("Operator precedence error: {0}")]
  ShuntingYardError(#[from] ShuntingYardError<Expr, Infallible>),
  #[error("Operator parsing error: {0}")]
  OperatorChainError(#[from] OperatorChainError<Expr>),
}

#[derive(Clone, Debug, Error)]
#[non_exhaustive]
pub enum ParsingError {
  #[error("Expected start of expression at {0}")]
  ExpectedStartOfExpr(SourceOffset),
  #[error("Expected right parenthesis or comma, got {0} at {1}")]
  ExpectedRParenOrComma(TokenData, SourceOffset),
  #[error("Expected parenthesized expression or complex number at {0}")]
  ExpectedParensOrComplex(SourceOffset),
  #[error("Expected operator")]
  ExpectedOperator,
  #[error("Unexpected EOF")]
  UnexpectedEOF, // TODO: We don't get SourceOffset here; find a way to get that info
  #[error("Expecting EOF at {0}")]
  ExpectedEOF(SourceOffset),
}

#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct ExprShuntingYardDriver {}

type IResult<'t, T> = Result<(T, &'t [Token]), ParseError>;

impl<'a> ExprParser<'a> {
  pub fn new(operator_table: &'a OperatorTable) -> Self {
    Self {
      tokenizer: ExprTokenizer::new(operator_table),
    }
  }

  pub fn tokenizer(&self) -> &ExprTokenizer<'a> {
    &self.tokenizer
  }

  pub fn tokenize_and_parse(&self, input: &str) -> Result<Expr, ParseError> {
    let mut state = TokenizerState::new(input);
    let tokens = self.tokenizer.read_tokens(&mut state)?;
    let expr = self.parse(&tokens)?;
    Ok(expr)
  }

  pub fn parse(&self, stream: &[Token]) -> Result<Expr, ParseError> {
    let (expr, stream) = self.parse_expr(stream)?;
    match stream.first() {
      Some(stray_token) => Err(ParsingError::ExpectedEOF(stray_token.span.start).into()),
      None => Ok(expr),
    }
  }

  fn parse_expr<'t>(&self, stream: &'t [Token]) -> IResult<'t, Expr> {
    let Some(token) = stream.first() else {
      return Err(ParsingError::UnexpectedEOF.into());
    };
    match &token.data {
      TokenData::Comma | TokenData::RightParen => {
        Err(ParsingError::ExpectedStartOfExpr(token.span.start).into())
      }
      TokenData::Var(_) | TokenData::Operator(_) | TokenData::FunctionCallStart(_) |
      TokenData::LeftParen | TokenData::Number(_) => {
        self.parse_operator_chain(stream)
      }
    }
  }

  fn parse_operator_chain<'t>(&self, mut stream: &'t [Token]) -> IResult<'t, Expr> {
    let mut tokens: Vec<Spanned<ChainToken<Expr>>> = Vec::new();
    // Read a sequence of operators or expressions, arbitrarily
    // intertwined.
    loop {
      match stream.first().map(|x| &x.data) {
        Some(TokenData::Operator(_)) => {
          // Read operator
          let (spanned, tail) = self.parse_operator(stream)?;
          tokens.push(spanned.map(ChainToken::Operator));
          stream = tail;
        }
        Some(TokenData::Var(_) | TokenData::FunctionCallStart(_) | TokenData::LeftParen | TokenData::Number(_)) => {
          // Read atomic expression
          let (spanned, tail) = self.parse_atom(stream)?;
          tokens.push(spanned.map(ChainToken::Scalar));
          stream = tail;
        }
        _ => {
          // End of operator chain
          break;
        }
      }
    }

    // Identify which operators should be treated as infix/postfix/prefix.
    let tagged_tokens: Vec<Spanned<TaggedToken<Expr>>> = chain::tag_chain_sequence(tokens)?;

    // Now use shunting yard to simplify the vector.
    let mut shunting_yard_driver = ExprShuntingYardDriver::new();
    let expr = shunting_yard::parse(&mut shunting_yard_driver, tagged_tokens)?;
    Ok((expr, stream))
  }

  fn parse_operator<'t>(&self, stream: &'t [Token]) -> IResult<'t, Spanned<Operator>> {
    if let Some(Token { data: TokenData::Operator(op), span }) = stream.first() {
      Ok((Spanned::new(op.clone(), *span), &stream[1..]))
    } else {
      Err(ParsingError::ExpectedOperator.into())
    }
  }

  fn parse_atom<'t>(&self, stream: &'t [Token]) -> IResult<'t, Spanned<Expr>> {
    let Some(token) = stream.first() else {
      return Err(ParsingError::UnexpectedEOF.into());
    };
    match &token.data {
      TokenData::Number(n) => {
        Ok((Spanned::new(Expr::from(n.clone()), token.span), &stream[1..]))
      }
      TokenData::Var(v) => {
        Ok((Spanned::new(Expr::from(v.clone()), token.span), &stream[1..]))
      }
      TokenData::FunctionCallStart(f) => {
        let ((args, end), tail) = self.parse_function_args(&stream[1..])?;
        Ok((Spanned::new(Expr::call(f, args), Span::new(token.span.start, end)), tail))
      }
      TokenData::LeftParen => {
        let ((mut args, end), tail) = self.parse_function_args(&stream[1..])?;
        let span = Span::new(token.span.start, end);
        match args.len() {
          1 => {
            // Parenthesized expression, just return the inside
            let arg = args.pop().unwrap();
            Ok((Spanned::new(arg, span), tail))
          }
          2 => {
            // Complex number expression. Right now we write this
            // explicitly and count on a simplifier to simplify it if
            // given literal numbers.
            let imag = args.pop().unwrap();
            let real = args.pop().unwrap();
            let expr = Expr::call("+", vec![real, Expr::call("*", vec![imag, Expr::from(ComplexNumber::ii())])]);
            Ok((Spanned::new(expr, span), tail))
          }
          _ => {
            Err(ParsingError::ExpectedParensOrComplex(token.span.start).into())
          }
        }
      }
      _ => {
        Err(ParsingError::ExpectedStartOfExpr(token.span.start).into())
      }
    }
  }

  // Expects and consumes a closing parenthesis at the end.
  fn parse_function_args<'t>(&self, mut stream: &'t [Token]) -> IResult<'t, (Vec<Expr>, SourceOffset)> {
    let close_paren_offset: SourceOffset;
    let mut output = Vec::new();
    loop {
      let Some(token) = stream.first() else {
        return Err(ParsingError::UnexpectedEOF.into());
      };
      if token.data == TokenData::RightParen {
        close_paren_offset = token.span.end;
        stream = &stream[1..];
        break;
      }
      let (next_expr, tail) = self.parse_expr(stream)?;
      output.push(next_expr);
      stream = tail;
      match stream.first() {
        Some(Token { data: TokenData::Comma, .. }) => {
          // Consume the comma (might be a trailing comma, but we
          // allow those).
          stream = &stream[1..];
        }
        Some(Token { data: TokenData::RightParen, .. }) => {
          // We're at the end of the list; next loop iteration will
          // terminate. Do nothing for now.
        }
        None => {
          return Err(ParsingError::UnexpectedEOF.into());
        }
        Some(Token { data, span }) => {
          return Err(ParsingError::ExpectedRParenOrComma(data.clone(), span.start).into());
        }
      }
    }
    Ok(((output, close_paren_offset), stream))
  }
}

impl ExprShuntingYardDriver {
  pub fn new() -> Self {
    Self {}
  }
}

impl ShuntingYardDriver<Expr> for ExprShuntingYardDriver {
  type Output = Expr;
  type Error = Infallible;

  fn compile_scalar(&mut self, scalar: Expr) -> Result<Expr, Infallible> {
    Ok(scalar)
  }

  fn compile_infix_op(&mut self, left: Expr, infix: &InfixProperties, right: Expr) -> Result<Expr, Infallible> {
    Ok(Expr::call(infix.function_name(), vec![left, right]))
  }

  fn compile_prefix_op(&mut self, prefix: &PrefixProperties, right: Expr) -> Result<Expr, Infallible> {
    Ok(Expr::call(prefix.function_name(), vec![right]))
  }

  fn compile_postfix_op(&mut self, left: Expr, postfix: &PostfixProperties) -> Result<Expr, Infallible> {
    Ok(Expr::call(postfix.function_name(), vec![left]))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::expr::number::Number;

  #[test]
  fn test_atom_parse() {
    let table = OperatorTable::common_operators();
    let parser = ExprParser::new(&table);

    let expr = parser.tokenize_and_parse("1").unwrap();
    assert_eq!(expr, Expr::from(1));

    let expr = parser.tokenize_and_parse("1.5").unwrap();
    assert_eq!(expr, Expr::from(Number::from(1.5)));

    let expr = parser.tokenize_and_parse("-1:3").unwrap();
    assert_eq!(expr, Expr::call("negate", vec![Expr::from(Number::ratio(1, 3))]));
  }

  #[test]
  fn test_parenthesized_expression() {
    let table = OperatorTable::common_operators();
    let parser = ExprParser::new(&table);

    let expr = parser.tokenize_and_parse("(1)").unwrap();
    assert_eq!(expr, Expr::from(1));

    let expr = parser.tokenize_and_parse("((((1))))").unwrap();
    assert_eq!(expr, Expr::from(1));
  }

  #[test]
  fn test_complex_number_expr() {
    let table = OperatorTable::common_operators();
    let parser = ExprParser::new(&table);

    let expr = parser.tokenize_and_parse("(1, 2)").unwrap();
    assert_eq!(
      expr,
      Expr::call(
        "+",
        vec![
          Expr::from(1),
          Expr::call("*", vec![Expr::from(2), Expr::from(ComplexNumber::ii())]),
        ],
      ),
    );
  }

  #[test]
  fn test_operator_sequence() {
    let table = OperatorTable::common_operators();
    let parser = ExprParser::new(&table);

    let expr = parser.tokenize_and_parse("1 + 2 * 3").unwrap();
    assert_eq!(expr, Expr::call("+", vec![Expr::from(1), Expr::call("*", vec![Expr::from(2), Expr::from(3)])]));

    let expr = parser.tokenize_and_parse("1 * 2 + 3").unwrap();
    assert_eq!(expr, Expr::call("+", vec![Expr::call("*", vec![Expr::from(1), Expr::from(2)]), Expr::from(3)]));
  }

  #[test]
  fn test_operator_sequence_with_parens() {
    let table = OperatorTable::common_operators();
    let parser = ExprParser::new(&table);

    let expr = parser.tokenize_and_parse("(1 + 2) * 3").unwrap();
    assert_eq!(expr, Expr::call("*", vec![Expr::call("+", vec![Expr::from(1), Expr::from(2)]), Expr::from(3)]));

    let expr = parser.tokenize_and_parse("1 + (2 * 3)").unwrap();
    assert_eq!(expr, Expr::call("+", vec![Expr::from(1), Expr::call("*", vec![Expr::from(2), Expr::from(3)])]));
  }

  #[test]
  fn test_function_call() {
    let table = OperatorTable::common_operators();
    let parser = ExprParser::new(&table);

    let expr = parser.tokenize_and_parse("foo((1 + 2) * 3)").unwrap();
    assert_eq!(expr, Expr::call("foo", vec![
      Expr::call("*", vec![Expr::call("+", vec![Expr::from(1), Expr::from(2)]), Expr::from(3)]),
    ]));
  }

  #[test]
  fn test_prefix_ops() {
    let table = OperatorTable::common_operators();
    let parser = ExprParser::new(&table);

    let expr = parser.tokenize_and_parse("+ + - 3").unwrap();
    assert_eq!(expr, Expr::call(
      "identity",
      vec![
        Expr::call(
          "identity",
          vec![
            Expr::call("negate", vec![Expr::from(3)]),
          ],
        ),
      ],
    ));
  }

  #[test]
  fn test_prefix_ops_with_infix() {
    let table = OperatorTable::common_operators();
    let parser = ExprParser::new(&table);

    let expr = parser.tokenize_and_parse("2 - - 3").unwrap();
    assert_eq!(expr, Expr::call(
      "-",
      vec![
        Expr::from(2),
        Expr::call("negate", vec![Expr::from(3)]),
      ],
    ));
  }

  #[test]
  fn test_prefix_ops_with_infix_and_at_beginning() {
    let table = OperatorTable::common_operators();
    let parser = ExprParser::new(&table);

    let expr = parser.tokenize_and_parse("- 2 - - 3").unwrap();
    assert_eq!(expr, Expr::call(
      "-",
      vec![
        Expr::call("negate", vec![Expr::from(2)]),
        Expr::call("negate", vec![Expr::from(3)]),
      ],
    ));
  }
}
