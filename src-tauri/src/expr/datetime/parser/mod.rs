
mod error;
mod time;
mod year;

pub use error::DatetimeParseError;

use super::DateTime;
use super::structure::{DatetimeValues, DateValues, DatetimeConstructionError};
use time::{TimeOfDay, search_for_time};

use regex::Regex;
use once_cell::sync::Lazy;
use either::Either;

/// A token is a sequence of letters or numbers, but not both.
static TOKEN_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"[a-zA-Z]+|[0-9]+").unwrap());

#[derive(Debug, Clone)]
pub struct DatetimeParser {
  now: DateTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Token<'a> {
  datum: &'a str,
}

impl DatetimeParser {
  pub fn new(now: DateTime) -> DatetimeParser {
    DatetimeParser { now }
  }

  pub fn with_current_local_time() -> DatetimeParser {
    DatetimeParser::new(DateTime::now_local())
  }

  pub fn parse_datetime_str(&self, input: &str) -> Result<DateTime, DatetimeParseError> {
    match self.parse_datetime_str_values(input)? {
      Either::Left(values) => Ok(DateTime::try_from(values)?),
      Either::Right(values) => Ok(DateTime::try_from(values)?),
    }
  }

  pub fn parse_datetime_str_values(&self, input: &str) -> Result<Either<DatetimeValues, DateValues>, DatetimeParseError> {
    let mut input = input.to_lowercase();
    let time_of_day = search_and_remove_time(&mut input)?;
    let mut tokens: Vec<_> = tokenize_datetime_str(&input).collect();
    
    todo!()
  }
}

fn extract_year(tokens: &mut Vec<Token>) -> Option<i32> {
  todo!()
}

impl<'a> Token<'a> {
  fn new(datum: &'a str) -> Self {
    Token { datum }
  }

  fn as_str(&self) -> &'a str {
    self.datum
  }
}

impl<T> From<DatetimeConstructionError<T>> for DatetimeParseError {
  fn from(err: DatetimeConstructionError<T>) -> Self {
    DatetimeParseError::DatetimeConstructionError { field_name: err.name() }
  }
}

fn search_and_remove_time(text: &mut String) -> Result<Option<TimeOfDay>, DatetimeParseError> {
  if let Some((time, m)) = search_for_time(text)? {
    text.drain(m.start()..m.end());
    Ok(Some(time))
  } else {
    Ok(None)
  }
}

fn tokenize_datetime_str(input: &str) -> impl Iterator<Item=Token> + '_ {
  TOKEN_RE.find_iter(input)
    .map(|m| Token::new(m.as_str()))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_tokenize_datetime_str() {
    let input = "ABC DEF ghi  jkl\tmno\npqrs";
    assert_eq!(
      tokenize_datetime_str(input).collect::<Vec<_>>(),
      vec![Token { datum: "ABC" }, Token { datum: "DEF" }, Token { datum: "ghi" },
           Token { datum: "jkl" }, Token { datum: "mno" }, Token { datum: "pqrs" }],
    );

    let input = "    ABC DEF ghi  jkl\tmno\npqrs  \n\n\r\n";
    assert_eq!(
      tokenize_datetime_str(input).collect::<Vec<_>>(),
      vec![Token { datum: "ABC" }, Token { datum: "DEF" }, Token { datum: "ghi" },
           Token { datum: "jkl" }, Token { datum: "mno" }, Token { datum: "pqrs" }],
    );

    let input = "0 1 2a b3 4CC4";
    assert_eq!(
      tokenize_datetime_str(input).collect::<Vec<_>>(),
      vec![Token { datum: "0" }, Token { datum: "1" }, Token { datum: "2" },
           Token { datum: "a" }, Token { datum: "b" }, Token { datum: "3" },
           Token { datum: "4" }, Token { datum: "CC" }, Token { datum: "4" }],
    );
  }
}
