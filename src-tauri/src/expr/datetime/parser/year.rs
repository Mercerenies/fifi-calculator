
use super::Token;
use super::error::DatetimeParseError;

use thiserror::Error;

use std::str::FromStr;

/// Finds and extracts the year component from a tokenized datetime
/// string. The year is scanned using the following rules.
///
/// 1. If a year suffix (such as "AD" or "BC") appears in the string,
///    then the token immediately before it MUST be the year.
///
/// 2. If not, then the first integer which is "obviously" a year is
///    considered the year. An integer is "obviously" the year is one
///    which
///
///   * is written with at least three digits (possibly including
///     leading zeroes), or
///
///   * has an explicit `+` or `-` sign.
///
/// 3. Otherwise, there is no year, so `Ok(None)` is returned.
///
/// On success, the token(s) related to the year are removed. On
/// failure or success with no year found, the vector is unmodified.
pub(super) fn find_and_extract_year(tokens: &mut Vec<Token>) -> Result<Option<i32>, DatetimeParseError> {
  // Look for year suffix.
  if let Some((index, suffix)) = find_year_suffix(tokens) {
    let year_num = tokens.get(index.wrapping_sub(1)).and_then(|t| t.as_str().parse::<i32>().ok())
      .ok_or(DatetimeParseError::MalformedYearField)?;
    tokens.drain(index-1..=index);
    return Ok(Some(suffix.apply(year_num)));
  }
  // Look for an obviously-year number.
  for (index, token) in tokens.iter().enumerate() {
    let s = token.as_str();
    if s.starts_with(['+', '-']) || s.len() >= 3 {
      if let Ok(n) = s.parse::<i32>() {
        tokens.remove(index);
        return Ok(Some(n));
      }
    }
  }
  // No year to report
  Ok(None)
}

fn find_year_suffix(tokens: &[Token]) -> Option<(usize, YearSuffix)> {
  for (i, token) in tokens.iter().enumerate() {
    if let Ok(suffix) = token.as_str().parse::<YearSuffix>() {
      return Some((i, suffix));
    }
  }
  None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum YearSuffix {
  Negative,
  Positive,
}

#[derive(Debug, Clone, Error)]
#[error("Invalid year suffix")]
struct YearSuffixFromStrError;

impl YearSuffix {
  fn apply(self, year: i32) -> i32 {
    match self {
      YearSuffix::Negative => -year,
      YearSuffix::Positive => year,
    }
  }
}

impl FromStr for YearSuffix {
  type Err = YearSuffixFromStrError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_ascii_lowercase().as_str() {
      "ad" | "a.d" | "a.d." | "ce" | "c.e." | "c.e" => Ok(YearSuffix::Positive),
      "bc" | "b.c." | "b.c" | "bce" | "b.c.e." | "b.c.e" => Ok(YearSuffix::Negative),
      _ => Err(YearSuffixFromStrError),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_year_suffix_from_str() {
    assert_eq!("AD".parse::<YearSuffix>().unwrap(), YearSuffix::Positive);
    assert_eq!("b.c".parse::<YearSuffix>().unwrap(), YearSuffix::Negative);
    assert_eq!("BcE".parse::<YearSuffix>().unwrap(), YearSuffix::Negative);
    assert!("".parse::<YearSuffix>().is_err());
    assert!("foo".parse::<YearSuffix>().is_err());
  }

  #[test]
  fn test_apply_year_suffix() {
    assert_eq!(YearSuffix::Positive.apply(100), 100);
    assert_eq!(YearSuffix::Positive.apply(-100), -100);
    assert_eq!(YearSuffix::Negative.apply(-100), 100);
    assert_eq!(YearSuffix::Negative.apply(100), -100);
  }

  #[test]
  fn test_find_year_suffix() {
    let tokens = [Token::new("A"), Token::new("9"), Token::new("ad"), Token::new("67")];
    assert_eq!(find_year_suffix(&tokens), Some((2, YearSuffix::Positive)));
    let tokens = [Token::new("A"), Token::new("9"), Token::new("ce"), Token::new("67")];
    assert_eq!(find_year_suffix(&tokens), Some((2, YearSuffix::Positive)));
    let tokens = [Token::new("A"), Token::new("9"), Token::new("B.C.E"), Token::new("67")];
    assert_eq!(find_year_suffix(&tokens), Some((2, YearSuffix::Negative)));
    let tokens = [Token::new("A"), Token::new("9"), Token::new("W"), Token::new("67")];
    assert_eq!(find_year_suffix(&tokens), None);
  }

  #[test]
  fn test_find_and_extract_year() {
    let mut tokens = vec![Token::new("aa"), Token::new("2010"), Token::new("BC")];
    let year = find_and_extract_year(&mut tokens).unwrap();
    assert_eq!(year, Some(-2010));
    assert_eq!(tokens, [Token::new("aa")]);

    let mut tokens = vec![Token::new("aa"), Token::new("2009"), Token::new("ee"), Token::new("3")];
    let year = find_and_extract_year(&mut tokens).unwrap();
    assert_eq!(year, Some(2009));
    assert_eq!(tokens, [Token::new("aa"), Token::new("ee"), Token::new("3")]);

    let mut tokens = vec![Token::new("12"), Token::new("2009"), Token::new("ee"), Token::new("3")];
    let year = find_and_extract_year(&mut tokens).unwrap();
    assert_eq!(year, Some(2009));
    assert_eq!(tokens, [Token::new("12"), Token::new("ee"), Token::new("3")]);

    let mut tokens = vec![Token::new("12"), Token::new("-3"), Token::new("ee"), Token::new("3")];
    let year = find_and_extract_year(&mut tokens).unwrap();
    assert_eq!(year, Some(-3));
    assert_eq!(tokens, [Token::new("12"), Token::new("ee"), Token::new("3")]);

    let mut tokens = vec![Token::new("12"), Token::new("+3"), Token::new("ee"), Token::new("3")];
    let year = find_and_extract_year(&mut tokens).unwrap();
    assert_eq!(year, Some(3));
    assert_eq!(tokens, [Token::new("12"), Token::new("ee"), Token::new("3")]);

    let mut tokens = vec![Token::new("12"), Token::new("003"), Token::new("ee"), Token::new("3")];
    let year = find_and_extract_year(&mut tokens).unwrap();
    assert_eq!(year, Some(3));
    assert_eq!(tokens, [Token::new("12"), Token::new("ee"), Token::new("3")]);

    let mut tokens = vec![Token::new("12"), Token::new("w"), Token::new("ee"), Token::new("3")];
    let year = find_and_extract_year(&mut tokens).unwrap();
    assert_eq!(year, None);
    assert_eq!(tokens, [Token::new("12"), Token::new("w"), Token::new("ee"), Token::new("3")]);

    let mut tokens = vec![Token::new("ee"), Token::new("ff")];
    let year = find_and_extract_year(&mut tokens).unwrap();
    assert_eq!(year, None);
    assert_eq!(tokens, [Token::new("ee"), Token::new("ff")]);
  }

  #[test]
  fn test_find_and_extract_year_failure() {
    let mut tokens = vec![Token::new("a"), Token::new("AD")];
    let err = find_and_extract_year(&mut tokens).unwrap_err();
    assert_eq!(err, DatetimeParseError::MalformedYearField);
    assert_eq!(tokens, [Token::new("a"), Token::new("AD")]);
  }
}
