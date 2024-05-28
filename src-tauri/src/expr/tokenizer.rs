
use super::number::{Number, ParseNumberError};
use crate::parsing::operator::{Operator, OperatorTable};
use crate::parsing::source::{Span, SourceOffset};
use crate::parsing::tokenizer::TokenizerState;
use crate::util::regex_opt_with;

use regex::Regex;
use once_cell::sync::Lazy;
use thiserror::Error;

use std::str::FromStr;
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug)]
pub struct ExprTokenizer<'a> {
  operator_table: &'a OperatorTable,
  operator_regex: Regex,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
  pub data: TokenData,
  pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenData {
  Number(Number),
  Operator(Operator),
  FunctionCallStart(String),
  LeftParen,
  Comma,
  RightParen,
}

#[derive(Debug, Clone, Error, PartialEq)]
#[non_exhaustive]
pub enum TokenizerError {
  #[error("Expected token, but found EOF at {0}")]
  UnexpectedEOF(SourceOffset),
  #[error("Expected token, but found '{0}' at {1}")]
  UnexpectedChar(char, SourceOffset),
  #[error("Failed to parse number")]
  ParseNumberError(#[from] ParseNumberError)
}

impl<'a> ExprTokenizer<'a> {
  pub fn new(operator_table: &'a OperatorTable) -> Self {
    let operator_names = operator_table.iter().map(|op| op.display_name());
    let operator_regex = regex_opt_with(operator_names, |s| format!("^{s}"));
    Self { operator_table, operator_regex }
  }

  pub fn read_tokens(&self, state: &mut TokenizerState<'_>) -> Result<Vec<Token>, TokenizerError> {
    let start_pos = state.current_pos();
    let mut tokens = Vec::new();
    loop {
      state.consume_spaces();
      match self.read_one_token(state) {
        Ok(token) => {
          tokens.push(token);
        }
        Err(TokenizerError::UnexpectedEOF(_)) => {
          return Ok(tokens);
        }
        Err(err) => {
          state.seek(start_pos);
          return Err(err);
        }
      }
    }
  }

  pub fn read_one_token(&self, state: &mut TokenizerState<'_>) -> Result<Token, TokenizerError> {
    if let Some(tok) = self.read_char_token(state) {
      Ok(tok)
    } else if let Some(tok) = self.read_function_call_token(state) {
      Ok(tok)
    } else if let Some(res) = self.read_number_literal(state) {
      res
    } else if let Some(tok) = self.read_operator(state) {
      Ok(tok)
    } else {
      match state.peek() {
        None => {
          let pos = state.current_pos();
          Err(TokenizerError::UnexpectedEOF(pos))
        }
        Some(ch) => {
          let pos = state.current_pos();
          Err(TokenizerError::UnexpectedChar(ch, pos))
        }
      }
    }
  }

  fn read_char_token(&self, state: &mut TokenizerState<'_>) -> Option<Token> {
    #[allow(clippy::manual_map)] // Cleaner in an if-else chain
    if let Some(m) = state.read_literal("(") {
      Some(Token::new(TokenData::LeftParen, m.span()))
    } else if let Some(m) = state.read_literal(")") {
      Some(Token::new(TokenData::RightParen, m.span()))
    } else if let Some(m) = state.read_literal(",") {
      Some(Token::new(TokenData::Comma, m.span()))
    } else {
      None
    }
  }

  fn read_function_call_token(&self, state: &mut TokenizerState<'_>) -> Option<Token> {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^([a-zA-Z_][a-zA-Z0-9_]*)\(").unwrap());
    state.read_regex_with_captures(&RE).map(|m| {
      let function_name = m.get(1).expect("expected at least one capture group");
      Token::new(TokenData::FunctionCallStart(function_name.to_owned()), m.span())
    })
  }

  fn read_operator(&self, state: &mut TokenizerState<'_>) -> Option<Token> {
    state.read_regex(&self.operator_regex).map(|m| {
      let operator = self.operator_table.get_by_display_name(m.as_str()).expect("expected operator to exist");
      Token::new(TokenData::Operator(operator.clone()), m.span())
    })
  }

  fn read_number_literal(&self, state: &mut TokenizerState<'_>) -> Option<Result<Token, TokenizerError>> {
    static RE: Lazy<Regex> = Lazy::new(|| {
      let ratio_re = r"[+-]?[0-9]+:[+-]?[0-9]+";
      let int_float_re = r"[+-]?[0-9]+(\.[0-9]+)?([eE][+-]?[0-9]+)?";
      Regex::new(&format!("^(?:{ratio_re}|{int_float_re})")).unwrap()
    });
    let reset_pos = state.current_pos();
    let m = state.read_regex(&RE)?;
    match Number::from_str(m.as_str()) {
      Err(err) => {
        state.seek(reset_pos);
        Some(Err(err.into()))
      }
      Ok(number) => {
        Some(Ok(Token::new(TokenData::Number(number), m.span())))
      }
    }
  }
}

impl Token {
  pub fn new(data: TokenData, span: Span) -> Self {
    Self { data, span }
  }
}

impl Display for TokenData {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      TokenData::Number(n) => write!(f, "{n}"),
      TokenData::Operator(op) => write!(f, "{}", op.display_name()),
      TokenData::FunctionCallStart(name) => write!(f, "{name}("),
      TokenData::LeftParen => write!(f, "("),
      TokenData::Comma => write!(f, ","),
      TokenData::RightParen => write!(f, ")"),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::parsing::operator::{Precedence, Associativity};

  fn sample_operator_table() -> OperatorTable {
    // Note: These are tokenizer unit tests. They should never use the
    // `function_name`, but we set it to something distinct from the
    // display name to make sure we grab the right name value.
    vec![
      Operator::new("+", Associativity::LEFT, Precedence::new(0)).with_function_name("plus"),
      Operator::new("++", Associativity::LEFT, Precedence::new(0)).with_function_name("concat"),
      Operator::new("*", Associativity::LEFT, Precedence::new(0)).with_function_name("times"),
    ].into_iter().collect()
  }

  fn span(start: usize, end: usize) -> Span {
    Span::new(SourceOffset(start), SourceOffset(end))
  }

  #[test]
  fn test_read_one_token_literal_char() {
    let table = sample_operator_table();
    let tokenizer = ExprTokenizer::new(&table);

    let mut state = TokenizerState::new("(");
    let token = tokenizer.read_one_token(&mut state).expect("expected token");
    assert_eq!(token, Token::new(TokenData::LeftParen, span(0, 1)));
    assert_eq!(state.current_pos(), SourceOffset(1));

    let mut state = TokenizerState::new(",");
    let token = tokenizer.read_one_token(&mut state).expect("expected token");
    assert_eq!(token, Token::new(TokenData::Comma, span(0, 1)));
    assert_eq!(state.current_pos(), SourceOffset(1));

    let mut state = TokenizerState::new(")");
    let token = tokenizer.read_one_token(&mut state).expect("expected token");
    assert_eq!(token, Token::new(TokenData::RightParen, span(0, 1)));
    assert_eq!(state.current_pos(), SourceOffset(1));
  }

  #[test]
  fn test_read_operator() {
    let table = sample_operator_table();
    let tokenizer = ExprTokenizer::new(&table);

    let mut state = TokenizerState::new("+");
    let token = tokenizer.read_one_token(&mut state).expect("expected token");
    assert_eq!(token, Token::new(TokenData::Operator(table.get_by_display_name("+").unwrap().clone()), span(0, 1)));
    assert_eq!(state.current_pos(), SourceOffset(1));

    let mut state = TokenizerState::new("++");
    let token = tokenizer.read_one_token(&mut state).expect("expected token");
    assert_eq!(token, Token::new(TokenData::Operator(table.get_by_display_name("++").unwrap().clone()), span(0, 2)));
    assert_eq!(state.current_pos(), SourceOffset(2));

    let mut state = TokenizerState::new("+++");
    let token = tokenizer.read_one_token(&mut state).expect("expected token");
    assert_eq!(token, Token::new(TokenData::Operator(table.get_by_display_name("++").unwrap().clone()), span(0, 2)));
    assert_eq!(state.current_pos(), SourceOffset(2));
  }

  #[test]
  fn test_function_call_start() {
    let table = sample_operator_table();
    let tokenizer = ExprTokenizer::new(&table);

    let mut state = TokenizerState::new("foo(");
    let token = tokenizer.read_one_token(&mut state).expect("expected token");
    assert_eq!(token, Token::new(TokenData::FunctionCallStart("foo".to_owned()), span(0, 4)));
    assert_eq!(state.current_pos(), SourceOffset(4));
  }

  #[test]
  fn test_number_int() {
    let table = sample_operator_table();
    let tokenizer = ExprTokenizer::new(&table);

    let mut state = TokenizerState::new("99");
    let token = tokenizer.read_one_token(&mut state).expect("expected token");
    assert_eq!(token, Token::new(TokenData::Number(Number::from(99)), span(0, 2)));
    assert_eq!(state.current_pos(), SourceOffset(2));

    let mut state = TokenizerState::new("-321");
    let token = tokenizer.read_one_token(&mut state).expect("expected token");
    assert_eq!(token, Token::new(TokenData::Number(Number::from(-321)), span(0, 4)));
    assert_eq!(state.current_pos(), SourceOffset(4));

    let mut state = TokenizerState::new("+321");
    let token = tokenizer.read_one_token(&mut state).expect("expected token");
    assert_eq!(token, Token::new(TokenData::Number(Number::from(321)), span(0, 4)));
    assert_eq!(state.current_pos(), SourceOffset(4));
  }

  #[test]
  fn test_number_ratio() {
    let table = sample_operator_table();
    let tokenizer = ExprTokenizer::new(&table);

    let mut state = TokenizerState::new("3:2");
    let token = tokenizer.read_one_token(&mut state).expect("expected token");
    assert_eq!(token, Token::new(TokenData::Number(Number::ratio(3, 2)), span(0, 3)));
    assert_eq!(state.current_pos(), SourceOffset(3));

    let mut state = TokenizerState::new("-9:2");
    let token = tokenizer.read_one_token(&mut state).expect("expected token");
    assert_eq!(token, Token::new(TokenData::Number(Number::ratio(-9, 2)), span(0, 4)));
    assert_eq!(state.current_pos(), SourceOffset(4));

    let mut state = TokenizerState::new("9:-11");
    let token = tokenizer.read_one_token(&mut state).expect("expected token");
    assert_eq!(token, Token::new(TokenData::Number(Number::ratio(-9, 11)), span(0, 5)));
    assert_eq!(state.current_pos(), SourceOffset(5));
  }

  #[test]
  fn test_number_ratio_zero_denominator() {
    let table = sample_operator_table();
    let tokenizer = ExprTokenizer::new(&table);

    let mut state = TokenizerState::new("3:0");
    let err = tokenizer.read_one_token(&mut state).unwrap_err();
    assert!(matches!(err, TokenizerError::ParseNumberError(_)));
    assert_eq!(state.current_pos(), SourceOffset(0));

    let mut state = TokenizerState::new("3:-0");
    let err = tokenizer.read_one_token(&mut state).unwrap_err();
    assert!(matches!(err, TokenizerError::ParseNumberError(_)));
    assert_eq!(state.current_pos(), SourceOffset(0));
  }

  #[test]
  fn test_invalid_token() {
    let table = sample_operator_table();
    let tokenizer = ExprTokenizer::new(&table);

    let mut state = TokenizerState::new("@");
    let err = tokenizer.read_one_token(&mut state).unwrap_err();
    assert_eq!(err, TokenizerError::UnexpectedChar('@', SourceOffset(0)));
    assert_eq!(state.current_pos(), SourceOffset(0));
  }

  #[test]
  fn test_token_stream() {
    let table = sample_operator_table();
    let tokenizer = ExprTokenizer::new(&table);

    let mut state = TokenizerState::new("1( a() , )");
    let tokens = tokenizer.read_tokens(&mut state).unwrap();
    assert_eq!(
      tokens,
      vec![
        Token::new(TokenData::Number(Number::from(1)), span(0, 1)),
        Token::new(TokenData::LeftParen, span(1, 2)),
        Token::new(TokenData::FunctionCallStart("a".to_owned()), span(3, 5)),
        Token::new(TokenData::RightParen, span(5, 6)),
        Token::new(TokenData::Comma, span(7, 8)),
        Token::new(TokenData::RightParen, span(9, 10)),
      ],
    );
    assert!(state.is_eof());
  }

  #[test]
  fn test_token_stream_with_extra_whitespace() {
    let table = sample_operator_table();
    let tokenizer = ExprTokenizer::new(&table);

    let mut state = TokenizerState::new("    1  ( a() , )        ");
    assert!(tokenizer.read_tokens(&mut state).is_ok());
    assert!(state.is_eof());
  }

  #[test]
  fn test_token_stream_failure_on_bad_char() {
    let table = sample_operator_table();
    let tokenizer = ExprTokenizer::new(&table);

    let mut state = TokenizerState::new("1( a() @)");
    let err = tokenizer.read_tokens(&mut state).unwrap_err();
    assert_eq!(err, TokenizerError::UnexpectedChar('@', SourceOffset(7)));
    assert_eq!(state.current_pos(), SourceOffset(0));
  }

  #[test]
  fn test_token_stream_failure_on_number_parse() {
    let table = sample_operator_table();
    let tokenizer = ExprTokenizer::new(&table);

    let mut state = TokenizerState::new("1( a() 1:0)");
    let err = tokenizer.read_tokens(&mut state).unwrap_err();
    assert!(matches!(err, TokenizerError::ParseNumberError(_)));
    assert_eq!(state.current_pos(), SourceOffset(0));
  }
}
