
use time::Month;

use thiserror::Error;

use std::str::FromStr;

/// Newtype wrapper around [`time::Month`] with a custom [`FromStr`]
/// implementation that can read numbers, fully-written month names,
/// or month abbreviations. All strings are case-insensitive when
/// parsed by this type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ParsedMonth(pub Month);

#[derive(Debug, Clone, Error)]
#[error("Malformed month string")]
pub(super) struct MalformedMonthString;

impl FromStr for ParsedMonth {
  type Err = MalformedMonthString;

  fn from_str(s: &str) -> Result<ParsedMonth, MalformedMonthString> {
    if let Some(m) = s.parse::<u8>().ok().and_then(|n| Month::try_from(n).ok()) {
      return Ok(ParsedMonth(m))
    }
    match s.to_lowercase().as_str() {
      "jan" | "january" => Ok(ParsedMonth(Month::January)),
      "feb" | "february" => Ok(ParsedMonth(Month::February)),
      "mar" | "march" => Ok(ParsedMonth(Month::March)),
      "apr" | "april" => Ok(ParsedMonth(Month::April)),
      "may" => Ok(ParsedMonth(Month::May)),
      "jun" | "june" => Ok(ParsedMonth(Month::June)),
      "jul" | "july" => Ok(ParsedMonth(Month::July)),
      "aug" | "august" => Ok(ParsedMonth(Month::August)),
      "sep" | "september" => Ok(ParsedMonth(Month::September)),
      "oct" | "october" => Ok(ParsedMonth(Month::October)),
      "nov" | "november" => Ok(ParsedMonth(Month::November)),
      "dec" | "december" => Ok(ParsedMonth(Month::December)),
      _ => Err(MalformedMonthString),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_month_str() {
    assert_eq!("january".parse::<ParsedMonth>().unwrap().0, Month::January);
    assert_eq!("APR".parse::<ParsedMonth>().unwrap().0, Month::April);
    assert_eq!("December".parse::<ParsedMonth>().unwrap().0, Month::December);
    assert_eq!("DeCeMbEr".parse::<ParsedMonth>().unwrap().0, Month::December);
    assert_eq!("12".parse::<ParsedMonth>().unwrap().0, Month::December);
    assert_eq!("1".parse::<ParsedMonth>().unwrap().0, Month::January);
    assert_eq!("002".parse::<ParsedMonth>().unwrap().0, Month::February);
  }

  #[test]
  fn test_parse_month_str_failures() {
    "19".parse::<ParsedMonth>().unwrap_err();
    "13".parse::<ParsedMonth>().unwrap_err();
    "0".parse::<ParsedMonth>().unwrap_err();
    "-1".parse::<ParsedMonth>().unwrap_err();
    "zzz".parse::<ParsedMonth>().unwrap_err();
    "".parse::<ParsedMonth>().unwrap_err();
    "Janu".parse::<ParsedMonth>().unwrap_err();
    "Feburary".parse::<ParsedMonth>().unwrap_err();
  }
}
