
use super::number::{Number, ParseNumberError};
use super::var::{self, Var};
use super::atom::{write_escaped_str, process_escape_char, InvalidEscapeError};
use crate::parsing::operator::{Operator, OperatorTable};
use crate::parsing::source::{Span, SourceOffset};
use crate::parsing::tokenizer::{TokenizerState, TokenizerCaptures};
use crate::util::regex_opt_with;
use crate::util::radix::{Radix, Digits, FromDigits, RadixFromStrError,
                         ValidateForRadixError, DigitsFromStrError, FromDigitsError};

use num::{BigInt, Zero};
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
  pub data: TokenData,
  pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenData {
  Number(Number),
  String(String),
  Var(Var),
  Operator(Operator),
  FunctionCallStart(String),
  LeftParen,
  Comma,
  RightParen,
  LeftBracket,
  RightBracket,
  DatetimeString(String),
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum TokenizerError {
  /// Unexpected EOF during tokenization. This error variant should
  /// ONLY be used in situations where it is valid to cancel
  /// tokenization and simply terminate the process. That is, it
  /// should NOT be used if EOF is encountered in an unusual state,
  /// such as in the middle of a string literal.
  #[error("Expected token, but found EOF at {0}")]
  UnexpectedEOF(SourceOffset),
  #[error("Un-terminated string literal at {0}")]
  UnterminatedString(SourceOffset),
  #[error("Un-terminated date string literal at {0}")]
  UnterminatedDatetimeString(SourceOffset),
  #[error("Expected token, but found '{0}' at {1}")]
  UnexpectedChar(char, SourceOffset),
  #[error("Failed to parse number")]
  ParseNumberError(#[from] ParseNumberError),
  #[error("{0}")]
  TokenizerRadixError(#[from] TokenizerRadixError),
  #[error("{0}")]
  InvalidEscapeError(#[from] InvalidEscapeError),
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum TokenizerRadixError {
  #[error("{0}")]
  RadixFromStrError(#[from] RadixFromStrError),
  #[error("{0}")]
  FromDigitsError(#[from] FromDigitsError),
  #[error("{0}")]
  ValidateForRadixError(#[from] ValidateForRadixError),
  #[error("{0}")]
  DigitsFromStrError(#[from] DigitsFromStrError),
}

impl<'a> ExprTokenizer<'a> {
  pub fn new(operator_table: &'a OperatorTable) -> Self {
    let operator_names = operator_table.iter().map(|op| op.operator_name());
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
    } else if let Some(res) = self.read_datetime_string_token(state) {
      res
    } else if let Some(res) = self.read_string_token(state) {
      res
    } else if let Some(tok) = self.read_function_call_token(state) {
      Ok(tok)
    } else if let Some(tok) = self.read_variable_token(state) {
      Ok(tok)
    } else if let Some(res) = self.read_radix_number_literal(state) {
      res
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
    } else if let Some(m) = state.read_literal("[") {
      Some(Token::new(TokenData::LeftBracket, m.span()))
    } else if let Some(m) = state.read_literal("]") {
      Some(Token::new(TokenData::RightBracket, m.span()))
    } else {
      None
    }
  }

  fn read_string_token(&self, state: &mut TokenizerState<'_>) -> Option<Result<Token, TokenizerError>> {
    let span_start = state.current_pos();
    state.read_literal("\"")?;
    Some(self.read_string_token_committed(state, span_start))
  }

  fn read_string_token_committed(
    &self,
    state: &mut TokenizerState<'_>,
    span_start: SourceOffset,
  ) -> Result<Token, TokenizerError> {
    let mut s = String::new();
    while let Some(ch) = state.peek() {
      state.advance(1);
      match ch {
        '"' => {
          let span_end = state.current_pos();
          let span = Span::new(span_start, span_end);
          let token = Token::new(TokenData::String(s), span);
          return Ok(token);
        }
        '\\' => {
          let Some(next_char) = state.peek() else {
            return Err(TokenizerError::UnterminatedString(state.current_pos()));
          };
          state.advance(1);
          s.push(process_escape_char(next_char)?);
        }
        ch => {
          s.push(ch);
        }
      }
    }
    Err(TokenizerError::UnterminatedString(state.current_pos()))
  }

  fn read_datetime_string_token(&self, state: &mut TokenizerState<'_>) -> Option<Result<Token, TokenizerError>> {
    let span_start = state.current_pos();
    state.read_literal("#<")?;
    Some(self.read_datetime_string_token_committed(state, span_start))
  }

  fn read_datetime_string_token_committed(
    &self,
    state: &mut TokenizerState<'_>,
    span_start: SourceOffset,
  ) -> Result<Token, TokenizerError> {
    let mut s = String::new();
    while let Some(ch) = state.peek() {
      state.advance(1);
      match ch {
        '>' => {
          let span_end = state.current_pos();
          let span = Span::new(span_start, span_end);
          let token = Token::new(TokenData::DatetimeString(s), span);
          return Ok(token);
        }
        ch => {
          s.push(ch);
        }
      }
    }
    Err(TokenizerError::UnterminatedDatetimeString(state.current_pos()))
  }

  fn read_function_call_token(&self, state: &mut TokenizerState<'_>) -> Option<Token> {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^([a-zA-Z_][a-zA-Z0-9_]*)\(").unwrap());
    state.read_regex_with_captures(&RE).map(|m| {
      let function_name = m.get(1).expect("expected at least one capture group");
      Token::new(TokenData::FunctionCallStart(function_name.to_owned()), m.span())
    })
  }

  fn read_variable_token(&self, state: &mut TokenizerState<'_>) -> Option<Token> {
    state.read_regex(&var::VALID_NAME_PREFIX_RE).map(|m| {
      let var = Var::new(m.as_str()).expect("expected valid variable name from tokenizer");
      Token::new(TokenData::Var(var), m.span())
    })
  }

  fn read_operator(&self, state: &mut TokenizerState<'_>) -> Option<Token> {
    state.read_regex(&self.operator_regex).map(|m| {
      let operator = self.operator_table.get_by_operator_name(m.as_str()).expect("expected operator to exist");
      Token::new(TokenData::Operator(operator.clone()), m.span())
    })
  }

  fn read_radix_number_literal(&self, state: &mut TokenizerState<'_>) -> Option<Result<Token, TokenizerError>> {
    static RE: Lazy<Regex> = Lazy::new(|| {
      let radix_re = r"([0-9]{1,2})#";
      let ratio_re = r"[0-9A-Za-z]+:[0-9A-Za-z]+";
      let int_float_re = r"[0-9A-Za-z]+(\.[0-9A-Za-z]+)?"; // TODO: Scientific notation?
      Regex::new(&format!("^{radix_re}({ratio_re}|{int_float_re})")).unwrap()
    });
    let reset_pos = state.current_pos();
    let m = state.read_regex_with_captures(&RE)?;
    Some(match self.read_radix_number_literal_committed(m) {
      Ok(x) => Ok(x),
      Err(err) => {
        state.seek(reset_pos);
        Err(err)
      }
    })
  }

  fn read_radix_number_literal_committed(
    &self,
    m: TokenizerCaptures,
  ) -> Result<Token, TokenizerError> {
    let radix = Radix::from_str(m.get(1).expect("expected at least one capture group"))?;
    let digits_str = m.get(2).expect("expected at least two capture groups");
    if digits_str.contains(':') {
      // Rational number
      let [numer, denom] = digits_str.split(':').collect::<Vec<_>>().try_into().unwrap();
      let numer_digits = Digits::from_str(numer)?;
      let denom_digits = Digits::from_str(denom)?;
      numer_digits.validate_for_radix(radix)?;
      denom_digits.validate_for_radix(radix)?;
      let numer = BigInt::from_digits(numer_digits, radix)?;
      let denom = BigInt::from_digits(denom_digits, radix)?;
      if denom == BigInt::zero() {
        return Err(TokenizerError::from(ParseNumberError {}));
      }
      let n = Number::ratio(numer, denom);
      Ok(Token::new(TokenData::Number(n), m.span()))
    } else {
      // Integer or floating value
      let digits = Digits::from_str(digits_str)?;
      digits.validate_for_radix(radix)?;
      if digits.fraction.is_empty() {
        let n = Number::from(BigInt::from_digits(digits, radix)?);
        Ok(Token::new(TokenData::Number(n), m.span()))
      } else {
        let n = Number::from(f64::from_digits(digits, radix)?);
        Ok(Token::new(TokenData::Number(n), m.span()))
      }
    }
  }

  fn read_number_literal(&self, state: &mut TokenizerState<'_>) -> Option<Result<Token, TokenizerError>> {
    static RE: Lazy<Regex> = Lazy::new(|| {
      let ratio_re = r"[0-9]+:[0-9]+";
      let int_float_re = r"[0-9]+(\.[0-9]+)?([eE][+-]?[0-9]+)?";
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

impl From<RadixFromStrError> for TokenizerError {
  fn from(err: RadixFromStrError) -> Self {
    TokenizerError::from(TokenizerRadixError::from(err))
  }
}

impl From<FromDigitsError> for TokenizerError {
  fn from(err: FromDigitsError) -> Self {
    TokenizerError::from(TokenizerRadixError::from(err))
  }
}

impl From<ValidateForRadixError> for TokenizerError {
  fn from(err: ValidateForRadixError) -> Self {
    TokenizerError::from(TokenizerRadixError::from(err))
  }
}

impl From<DigitsFromStrError> for TokenizerError {
  fn from(err: DigitsFromStrError) -> Self {
    TokenizerError::from(TokenizerRadixError::from(err))
  }
}

impl Display for TokenData {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      TokenData::Number(n) => write!(f, "{n}"),
      TokenData::Var(v) => write!(f, "{v}"),
      TokenData::String(s) => write_escaped_str(f, s),
      TokenData::DatetimeString(s) => write!(f, "#<{}>", s),
      TokenData::Operator(op) => write!(f, "{}", op.operator_name()),
      TokenData::FunctionCallStart(name) => write!(f, "{name}("),
      TokenData::LeftParen => write!(f, "("),
      TokenData::Comma => write!(f, ","),
      TokenData::RightParen => write!(f, ")"),
      TokenData::LeftBracket => write!(f, "["),
      TokenData::RightBracket => write!(f, "]"),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::parsing::operator::{Precedence, Associativity};
  use crate::parsing::operator::fixity::Fixity;

  fn sample_operator_table() -> OperatorTable {
    // Note: These are tokenizer unit tests. They should never use the
    // `function_name`, but we set it to something distinct from the
    // display name to make sure we grab the right name value.
    vec![
      Operator::new("+", Fixity::new().with_infix("plus", Associativity::LEFT, Precedence::new(0))),
      Operator::new("++", Fixity::new().with_infix("concat", Associativity::LEFT, Precedence::new(0))),
      Operator::new("*", Fixity::new().with_infix("times", Associativity::LEFT, Precedence::new(0))),
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
    assert_eq!(token, Token::new(TokenData::Operator(table.get_by_operator_name("+").unwrap().clone()), span(0, 1)));
    assert_eq!(state.current_pos(), SourceOffset(1));

    let mut state = TokenizerState::new("++");
    let token = tokenizer.read_one_token(&mut state).expect("expected token");
    assert_eq!(token, Token::new(TokenData::Operator(table.get_by_operator_name("++").unwrap().clone()), span(0, 2)));
    assert_eq!(state.current_pos(), SourceOffset(2));

    let mut state = TokenizerState::new("+++");
    let token = tokenizer.read_one_token(&mut state).expect("expected token");
    assert_eq!(token, Token::new(TokenData::Operator(table.get_by_operator_name("++").unwrap().clone()), span(0, 2)));
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
  }

  #[test]
  fn test_number_ratio() {
    let table = sample_operator_table();
    let tokenizer = ExprTokenizer::new(&table);

    let mut state = TokenizerState::new("3:2");
    let token = tokenizer.read_one_token(&mut state).expect("expected token");
    assert_eq!(token, Token::new(TokenData::Number(Number::ratio(3, 2)), span(0, 3)));
    assert_eq!(state.current_pos(), SourceOffset(3));
  }

  #[test]
  fn test_number_binary() {
    let table = sample_operator_table();
    let tokenizer = ExprTokenizer::new(&table);

    let mut state = TokenizerState::new("2#110");
    let token = tokenizer.read_one_token(&mut state).expect("expected token");
    assert_eq!(token, Token::new(TokenData::Number(Number::from(6)), span(0, 5)));
    assert_eq!(state.current_pos(), SourceOffset(5));
  }

  #[test]
  fn test_number_hexadecimal() {
    let table = sample_operator_table();
    let tokenizer = ExprTokenizer::new(&table);

    let mut state = TokenizerState::new("16#Ab");
    let token = tokenizer.read_one_token(&mut state).expect("expected token");
    assert_eq!(token, Token::new(TokenData::Number(Number::from(171)), span(0, 5)));
    assert_eq!(state.current_pos(), SourceOffset(5));
  }

  #[test]
  fn test_number_binary_with_fractional_part() {
    let table = sample_operator_table();
    let tokenizer = ExprTokenizer::new(&table);

    let mut state = TokenizerState::new("2#110.01");
    let token = tokenizer.read_one_token(&mut state).expect("expected token");
    assert_eq!(token, Token::new(TokenData::Number(Number::from(6.25)), span(0, 8)));
    assert_eq!(state.current_pos(), SourceOffset(8));

    let mut state = TokenizerState::new("2#110.00");
    let token = tokenizer.read_one_token(&mut state).expect("expected token");
    assert_eq!(token, Token::new(TokenData::Number(Number::from(6.0)), span(0, 8)));
    assert_eq!(state.current_pos(), SourceOffset(8));
  }

  #[test]
  fn test_number_binary_rational() {
    let table = sample_operator_table();
    let tokenizer = ExprTokenizer::new(&table);

    let mut state = TokenizerState::new("2#110:101");
    let token = tokenizer.read_one_token(&mut state).expect("expected token");
    assert_eq!(token, Token::new(TokenData::Number(Number::ratio(6, 5)), span(0, 9)));
    assert_eq!(state.current_pos(), SourceOffset(9));
  }

  #[test]
  fn test_number_binary_digit_out_of_bounds() {
    let table = sample_operator_table();
    let tokenizer = ExprTokenizer::new(&table);

    let mut state = TokenizerState::new("2#102");
    let err = tokenizer.read_one_token(&mut state).unwrap_err();
    assert!(matches!(err, TokenizerError::TokenizerRadixError(TokenizerRadixError::ValidateForRadixError(_))));
    assert_eq!(state.current_pos(), SourceOffset(0));
  }

  #[test]
  fn test_number_with_bad_radix() {
    let table = sample_operator_table();
    let tokenizer = ExprTokenizer::new(&table);

    let mut state = TokenizerState::new("42#1");
    let err = tokenizer.read_one_token(&mut state).unwrap_err();
    assert!(matches!(err, TokenizerError::TokenizerRadixError(TokenizerRadixError::RadixFromStrError(_))));
    assert_eq!(state.current_pos(), SourceOffset(0));
  }

  #[test]
  fn test_number_ratio_zero_denominator() {
    let table = sample_operator_table();
    let tokenizer = ExprTokenizer::new(&table);

    let mut state = TokenizerState::new("3:0");
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
  fn test_token_stream_with_brackets() {
    let table = sample_operator_table();
    let tokenizer = ExprTokenizer::new(&table);

    let mut state = TokenizerState::new("[ a() , ]");
    let tokens = tokenizer.read_tokens(&mut state).unwrap();
    assert_eq!(
      tokens,
      vec![
        Token::new(TokenData::LeftBracket, span(0, 1)),
        Token::new(TokenData::FunctionCallStart("a".to_owned()), span(2, 4)),
        Token::new(TokenData::RightParen, span(4, 5)),
        Token::new(TokenData::Comma, span(6, 7)),
        Token::new(TokenData::RightBracket, span(8, 9)),
      ],
    );
    assert!(state.is_eof());
  }

  #[test]
  fn test_token_stream_with_var() {
    let table = sample_operator_table();
    let tokenizer = ExprTokenizer::new(&table);

    let mut state = TokenizerState::new("1( a() , z z) z'");
    let tokens = tokenizer.read_tokens(&mut state).unwrap();
    assert_eq!(
      tokens,
      vec![
        Token::new(TokenData::Number(Number::from(1)), span(0, 1)),
        Token::new(TokenData::LeftParen, span(1, 2)),
        Token::new(TokenData::FunctionCallStart("a".to_owned()), span(3, 5)),
        Token::new(TokenData::RightParen, span(5, 6)),
        Token::new(TokenData::Comma, span(7, 8)),
        Token::new(TokenData::Var(Var::new("z").unwrap()), span(9, 10)),
        Token::new(TokenData::Var(Var::new("z").unwrap()), span(11, 12)),
        Token::new(TokenData::RightParen, span(12, 13)),
        Token::new(TokenData::Var(Var::new("z'").unwrap()), span(14, 16)),
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

  #[test]
  fn test_token_stream_on_string() {
    let table = sample_operator_table();
    let tokenizer = ExprTokenizer::new(&table);

    let mut state = TokenizerState::new(r#" "a\"b" "#);
    let tokens = tokenizer.read_tokens(&mut state).unwrap();
    assert_eq!(
      tokens,
      vec![
        Token::new(TokenData::String("a\"b".to_owned()), span(1, 7)),
      ],
    )
  }

  #[test]
  fn test_token_stream_on_string_with_bad_escape_sequence() {
    let table = sample_operator_table();
    let tokenizer = ExprTokenizer::new(&table);

    let mut state = TokenizerState::new(r#" "a\9b" "#);
    let err = tokenizer.read_tokens(&mut state).unwrap_err();
    assert_eq!(err, TokenizerError::InvalidEscapeError(InvalidEscapeError { character: '9' }));
  }

  #[test]
  fn test_token_stream_nonterminated_string() {
    let table = sample_operator_table();
    let tokenizer = ExprTokenizer::new(&table);

    let mut state = TokenizerState::new(r#""a"#);
    let err = tokenizer.read_tokens(&mut state).unwrap_err();
    assert_eq!(err, TokenizerError::UnterminatedString(SourceOffset(2)));
  }

  #[test]
  fn test_token_stream_nonterminated_string_in_escape() {
    let table = sample_operator_table();
    let tokenizer = ExprTokenizer::new(&table);

    let mut state = TokenizerState::new(r#""a\"#);
    let err = tokenizer.read_tokens(&mut state).unwrap_err();
    assert_eq!(err, TokenizerError::UnterminatedString(SourceOffset(3)));
  }
}
