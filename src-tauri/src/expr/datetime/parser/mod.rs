
mod error;
mod month;
mod time;
mod timezone;
mod year;

pub use error::DatetimeParseError;

use super::DateTime;
use super::structure::{DatetimeValues, DateValues, DatetimeConstructionError};
use time::{TimeOfDay, search_for_time};
use timezone::{Timezone, search_for_timezone};
use year::find_and_extract_year;
use month::ParsedMonth;

use regex::Regex;
use once_cell::sync::Lazy;
use either::Either;
use phf::phf_set;

/// A token is a sequence of letters or numbers, but not both. A
/// single leading plus or minus sign is permitted if succeeded by a
/// digit.
///
/// The tokenizer function [`tokenize_datetime_str`] additionally
/// strips off the sign if it's preceded by a non-whitespace
/// character.
static TOKEN_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"[a-zA-Z]+|[+-]?[0-9]+").unwrap());

static DAYS_OF_WEEK: phf::Set<&'static str> = phf_set! {
  "mon", "tue", "wed", "thu", "fri", "sat", "sun",
  "monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday",
};

#[derive(Debug, Clone)]
pub struct DatetimeParser {
  now: DateTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Token<'a> {
  datum: &'a str,
}

/// If any element of the vector satisfies the predicate, removes that
/// element and returns the first match. Otherwise, returns `None`
/// without modifying the vector.
fn find_and_extract_match<T, S, F>(elements: &mut Vec<T>, mut pred: F) -> Option<S>
where F: FnMut(&T) -> Option<S> {
  for (i, elem) in elements.iter().enumerate() {
    if let Some(m) = pred(elem) {
      elements.remove(i);
      return Some(m);
    }
  }
  None
}

fn find_and_extract_month(tokens: &mut Vec<Token>) -> Option<ParsedMonth> {
  fn starts_with_alphabetic(s: &str) -> bool {
    s.chars().next().is_some_and(|c| c.is_alphabetic())
  }

  fn alphabetic_parse_month(t: &Token) -> Option<ParsedMonth> {
    if starts_with_alphabetic(t.as_str()) {
      t.as_str().parse::<ParsedMonth>().ok()
    } else {
      None
    }
  }

  // Prefer alphabetic month names, if there are any.
  find_and_extract_match(tokens, alphabetic_parse_month).or_else(|| {
    find_and_extract_match(tokens, |t| t.as_str().parse::<ParsedMonth>().ok())
  })
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
    const OFFSET_UTC: Timezone = Timezone(0);

    let mut input = input.to_lowercase();
    let time_of_day = search_and_remove_time(&mut input)?;
    let timezone = search_and_remove_timezone(&mut input)
      .unwrap_or(OFFSET_UTC);

    let mut tokens: Vec<_> = tokenize_datetime_str(&input).collect();

    // Remove any tokens that refer to days of the week, since that
    // information (if provided) is redundant.
    tokens.retain(|t| !DAYS_OF_WEEK.contains(&t.as_str().to_lowercase()));

    let year = find_and_extract_year(&mut tokens)?
      .unwrap_or_else(|| self.now.year());
    let month = find_and_extract_month(&mut tokens)
      .unwrap_or_else(|| ParsedMonth(self.now.month()));
    let day = find_and_extract_match(&mut tokens, |t| t.as_str().parse::<u8>().ok())
      .unwrap_or_else(|| self.now.day());
    if !tokens.is_empty() {
      return Err(DatetimeParseError::UnexpectedToken { token: tokens[0].as_str().to_owned() });
    }

    match time_of_day {
      None => Ok(Either::Right(DateValues {
        year,
        month: month.0.into(),
        day,
      })),
      Some(time_of_day) => Ok(Either::Left(DatetimeValues {
        year,
        month: month.0.into(),
        day,
        hour: time_of_day.hour,
        minute: time_of_day.minute,
        second: time_of_day.second,
        micro: time_of_day.microsecond,
        offset: timezone.0,
      }))
    }
  }
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

fn search_and_remove_timezone(text: &mut String) -> Option<Timezone> {
  if let Some((tz, m)) = search_for_timezone(text) {
    text.drain(m.start()..m.end());
    Some(tz)
  } else {
    None
  }
}

fn tokenize_datetime_str(input: &str) -> impl Iterator<Item=Token> + '_ {
  TOKEN_RE.find_iter(input)
    .map(|m| {
      let s = m.as_str();
      if s.starts_with(['+', '-']) && !is_empty_or_whitespace(char_before(input, m.start())) {
        // Byte-indexing is safe since we know the first char is '+'
        // or '-'.
        Token::new(&s[1..])
      } else {
        Token::new(s)
      }
    })
}

/// UTF-8 character before the specified index. Returns `None` if
/// there is no character before, or if the given byte index does not
/// fall on a UTF-8 codepoint boundary.
fn char_before(s: &str, idx: usize) -> Option<char> {
  s.get(..idx)?.chars().next_back()
}

fn is_empty_or_whitespace(ch: Option<char>) -> bool {
  match ch {
    None => true,
    Some(ch) => ch.is_whitespace(),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use ::time::{OffsetDateTime, Date, Time, Month};

  /// Arbitrary epoch for testing purposes.
  fn epoch() -> DateTime {
    DateTime::from(OffsetDateTime::new_utc(
      Date::from_calendar_date(2000, Month::February, 3).unwrap(),
      Time::from_hms(4, 5, 6).unwrap(),
    ))
  }

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

  #[test]
  fn test_tokenize_datetime_str_with_signs() {
    let input = "a+bb+3 -2-1";
    assert_eq!(
      tokenize_datetime_str(input).collect::<Vec<_>>(),
      vec![Token { datum: "a" }, Token { datum: "bb" }, Token { datum: "3" },
           Token { datum: "-2" }, Token { datum: "1" }],
    );

    let input = "+7";
    assert_eq!(
      tokenize_datetime_str(input).collect::<Vec<_>>(),
      vec![Token { datum: "+7" }],
    );

    let input = "-1-2";
    assert_eq!(
      tokenize_datetime_str(input).collect::<Vec<_>>(),
      vec![Token { datum: "-1" }, Token { datum: "2" }],
    );
  }

  #[test]
  fn test_parse_datetime_str_values_empty() {
    let parser = DatetimeParser::new(epoch());

    let values = parser.parse_datetime_str_values("").unwrap().unwrap_right();
    assert_eq!(values, DateValues {
      year: 2000,
      month: 2,
      day: 3,
    });

    let values = parser.parse_datetime_str_values("     ").unwrap().unwrap_right();
    assert_eq!(values, DateValues {
      year: 2000,
      month: 2,
      day: 3,
    });
  }

  #[test]
  fn test_parse_datetime_str_values_with_single_field() {
    let parser = DatetimeParser::new(epoch());

    let values = parser.parse_datetime_str_values("2020").unwrap().unwrap_right();
    assert_eq!(values, DateValues {
      year: 2020,
      month: 2,
      day: 3,
    });

    let values = parser.parse_datetime_str_values("-3").unwrap().unwrap_right();
    assert_eq!(values, DateValues {
      year: -3,
      month: 2,
      day: 3,
    });

    let values = parser.parse_datetime_str_values("3bc").unwrap().unwrap_right();
    assert_eq!(values, DateValues {
      year: -3,
      month: 2,
      day: 3,
    });

    let values = parser.parse_datetime_str_values("10 CE").unwrap().unwrap_right();
    assert_eq!(values, DateValues {
      year: 10,
      month: 2,
      day: 3,
    });

    let values = parser.parse_datetime_str_values("Jan").unwrap().unwrap_right();
    assert_eq!(values, DateValues {
      year: 2000,
      month: 1,
      day: 3,
    });

    let values = parser.parse_datetime_str_values("\t\tMARCh ").unwrap().unwrap_right();
    assert_eq!(values, DateValues {
      year: 2000,
      month: 3,
      day: 3,
    });

    let values = parser.parse_datetime_str_values("12").unwrap().unwrap_right();
    assert_eq!(values, DateValues {
      year: 2000,
      month: 12,
      day: 3,
    });

    let values = parser.parse_datetime_str_values("13").unwrap().unwrap_right();
    assert_eq!(values, DateValues {
      year: 2000,
      month: 2,
      day: 13,
    });

    let values = parser.parse_datetime_str_values("+12").unwrap().unwrap_right();
    assert_eq!(values, DateValues {
      year: 12,
      month: 2,
      day: 3,
    });

    let values = parser.parse_datetime_str_values("+12").unwrap().unwrap_right();
    assert_eq!(values, DateValues {
      year: 12,
      month: 2,
      day: 3,
    });

    let values = parser.parse_datetime_str_values("012").unwrap().unwrap_right();
    assert_eq!(values, DateValues {
      year: 12,
      month: 2,
      day: 3,
    });

    let values = parser.parse_datetime_str_values("12 MON").unwrap().unwrap_right();
    assert_eq!(values, DateValues {
      year: 2000,
      month: 12,
      day: 3,
    });
  }

  #[test]
  fn test_parse_datetime_str_values_with_two_fields() {
    let parser = DatetimeParser::new(epoch());

    let values = parser.parse_datetime_str_values("2020 9").unwrap().unwrap_right();
    assert_eq!(values, DateValues {
      year: 2020,
      month: 9,
      day: 3,
    });

    let values = parser.parse_datetime_str_values("2020 1").unwrap().unwrap_right();
    assert_eq!(values, DateValues {
      year: 2020,
      month: 1,
      day: 3,
    });

    let values = parser.parse_datetime_str_values("4 2020").unwrap().unwrap_right();
    assert_eq!(values, DateValues {
      year: 2020,
      month: 4,
      day: 3,
    });

    let values = parser.parse_datetime_str_values("14 2020").unwrap().unwrap_right();
    assert_eq!(values, DateValues {
      year: 2020,
      month: 2,
      day: 14,
    });

    // Note: 99 is not a valid day in February, but this function is
    // not responsible for validating that.
    let values = parser.parse_datetime_str_values("99 2020").unwrap().unwrap_right();
    assert_eq!(values, DateValues {
      year: 2020,
      month: 2,
      day: 99,
    });

    let values = parser.parse_datetime_str_values("100 202").unwrap().unwrap_right();
    assert_eq!(values, DateValues {
      year: 100,
      month: 2,
      day: 202,
    });

    let values = parser.parse_datetime_str_values("Jan 2001").unwrap().unwrap_right();
    assert_eq!(values, DateValues {
      year: 2001,
      month: 1,
      day: 3,
    });

    let values = parser.parse_datetime_str_values("SUN Jan 2001").unwrap().unwrap_right();
    assert_eq!(values, DateValues {
      year: 2001,
      month: 1,
      day: 3,
    });

    let values = parser.parse_datetime_str_values("MON Jan 2001").unwrap().unwrap_right();
    assert_eq!(values, DateValues {
      year: 2001,
      month: 1,
      day: 3,
    });

    let values = parser.parse_datetime_str_values("2001 Jan").unwrap().unwrap_right();
    assert_eq!(values, DateValues {
      year: 2001,
      month: 1,
      day: 3,
    });
  }

  #[test]
  fn test_parse_datetime_str_values_with_three_fields() {
    let parser = DatetimeParser::new(epoch());

    let values = parser.parse_datetime_str_values("2020 Jan 9").unwrap().unwrap_right();
    assert_eq!(values, DateValues {
      year: 2020,
      month: 1,
      day: 9,
    });

    let values = parser.parse_datetime_str_values("2020 9 Jan").unwrap().unwrap_right();
    assert_eq!(values, DateValues {
      year: 2020,
      month: 1,
      day: 9,
    });

    let values = parser.parse_datetime_str_values("4-5-2020").unwrap().unwrap_right();
    assert_eq!(values, DateValues {
      year: 2020,
      month: 4,
      day: 5,
    });

    let values = parser.parse_datetime_str_values("2020 Apr, 5").unwrap().unwrap_right();
    assert_eq!(values, DateValues {
      year: 2020,
      month: 4,
      day: 5,
    });
  }

  #[test]
  fn test_parse_datetime_str_with_time() {
    let parser = DatetimeParser::new(epoch());

    let values = parser.parse_datetime_str_values("3am").unwrap().unwrap_left();
    assert_eq!(values, DatetimeValues {
      year: 2000,
      month: 2,
      day: 3,
      hour: 3,
      minute: 0,
      second: 0,
      micro: 0,
      offset: 0,
    });

    let values = parser.parse_datetime_str_values("3:01pm").unwrap().unwrap_left();
    assert_eq!(values, DatetimeValues {
      year: 2000,
      month: 2,
      day: 3,
      hour: 15,
      minute: 1,
      second: 0,
      micro: 0,
      offset: 0,
    });

    let values = parser.parse_datetime_str_values("3:01:10.4pm").unwrap().unwrap_left();
    assert_eq!(values, DatetimeValues {
      year: 2000,
      month: 2,
      day: 3,
      hour: 15,
      minute: 1,
      second: 10,
      micro: 400_000,
      offset: 0,
    });

    let values = parser.parse_datetime_str_values("3:01:10.45pm").unwrap().unwrap_left();
    assert_eq!(values, DatetimeValues {
      year: 2000,
      month: 2,
      day: 3,
      hour: 15,
      minute: 1,
      second: 10,
      micro: 450_000,
      offset: 0,
    });

    let values = parser.parse_datetime_str_values("3:01:10.45pm WeDnEsDaY").unwrap().unwrap_left();
    assert_eq!(values, DatetimeValues {
      year: 2000,
      month: 2,
      day: 3,
      hour: 15,
      minute: 1,
      second: 10,
      micro: 450_000,
      offset: 0,
    });
  }

  #[test]
  fn test_parse_datetime_str_values_with_time_and_single_field() {
    let parser = DatetimeParser::new(epoch());

    let values = parser.parse_datetime_str_values("2020 3:00").unwrap().unwrap_left();
    assert_eq!(values, DatetimeValues {
      year: 2020,
      month: 2,
      day: 3,
      hour: 3,
      minute: 0,
      second: 0,
      micro: 0,
      offset: 0,
    });

    let values = parser.parse_datetime_str_values("3:00 2020").unwrap().unwrap_left();
    assert_eq!(values, DatetimeValues {
      year: 2020,
      month: 2,
      day: 3,
      hour: 3,
      minute: 0,
      second: 0,
      micro: 0,
      offset: 0,
    });

    let values = parser.parse_datetime_str_values("3:00 5bc").unwrap().unwrap_left();
    assert_eq!(values, DatetimeValues {
      year: -5,
      month: 2,
      day: 3,
      hour: 3,
      minute: 0,
      second: 0,
      micro: 0,
      offset: 0,
    });

    let values = parser.parse_datetime_str_values("3:00 5  bc").unwrap().unwrap_left();
    assert_eq!(values, DatetimeValues {
      year: -5,
      month: 2,
      day: 3,
      hour: 3,
      minute: 0,
      second: 0,
      micro: 0,
      offset: 0,
    });


    let values = parser.parse_datetime_str_values("3:00 Apr").unwrap().unwrap_left();
    assert_eq!(values, DatetimeValues {
      year: 2000,
      month: 4,
      day: 3,
      hour: 3,
      minute: 0,
      second: 0,
      micro: 0,
      offset: 0,
    });

    let values = parser.parse_datetime_str_values("3:50pm Apr").unwrap().unwrap_left();
    assert_eq!(values, DatetimeValues {
      year: 2000,
      month: 4,
      day: 3,
      hour: 15,
      minute: 50,
      second: 0,
      micro: 0,
      offset: 0,
    });

    let values = parser.parse_datetime_str_values("12 12noon").unwrap().unwrap_left();
    assert_eq!(values, DatetimeValues {
      year: 2000,
      month: 12,
      day: 3,
      hour: 12,
      minute: 0,
      second: 0,
      micro: 0,
      offset: 0,
    });

    let values = parser.parse_datetime_str_values("12noon 12").unwrap().unwrap_left();
    assert_eq!(values, DatetimeValues {
      year: 2000,
      month: 12,
      day: 3,
      hour: 12,
      minute: 0,
      second: 0,
      micro: 0,
      offset: 0,
    });

    let values = parser.parse_datetime_str_values("12midnight 12").unwrap().unwrap_left();
    assert_eq!(values, DatetimeValues {
      year: 2000,
      month: 12,
      day: 3,
      hour: 0,
      minute: 0,
      second: 0,
      micro: 0,
      offset: 0,
    });

    let values = parser.parse_datetime_str_values("12midnight 12 Tue").unwrap().unwrap_left();
    assert_eq!(values, DatetimeValues {
      year: 2000,
      month: 12,
      day: 3,
      hour: 0,
      minute: 0,
      second: 0,
      micro: 0,
      offset: 0,
    });
  }

  #[test]
  fn test_parse_datetime_str_values_with_time_and_two_fields() {
    let parser = DatetimeParser::new(epoch());

    let values = parser.parse_datetime_str_values("2020 9 5:04pm").unwrap().unwrap_left();
    assert_eq!(values, DatetimeValues {
      year: 2020,
      month: 9,
      day: 3,
      hour: 17,
      minute: 4,
      second: 0,
      micro: 0,
      offset: 0,
    });

    let values = parser.parse_datetime_str_values("2020 5:04pm  \t 9").unwrap().unwrap_left();
    assert_eq!(values, DatetimeValues {
      year: 2020,
      month: 9,
      day: 3,
      hour: 17,
      minute: 4,
      second: 0,
      micro: 0,
      offset: 0,
    });

    let values = parser.parse_datetime_str_values("5:04pm2020\t9").unwrap().unwrap_left();
    assert_eq!(values, DatetimeValues {
      year: 2020,
      month: 9,
      day: 3,
      hour: 17,
      minute: 4,
      second: 0,
      micro: 0,
      offset: 0,
    });
  }

  #[test]
  fn test_parse_datetime_str_values_with_time_and_three_fields() {
    let parser = DatetimeParser::new(epoch());

    let values = parser.parse_datetime_str_values("5:04pm 2020-10-11").unwrap().unwrap_left();
    assert_eq!(values, DatetimeValues {
      year: 2020,
      month: 10,
      day: 11,
      hour: 17,
      minute: 4,
      second: 0,
      micro: 0,
      offset: 0,
    });

    let values = parser.parse_datetime_str_values("2020-10-5:04pm-11").unwrap().unwrap_left();
    assert_eq!(values, DatetimeValues {
      year: 2020,
      month: 10,
      day: 11,
      hour: 17,
      minute: 4,
      second: 0,
      micro: 0,
      offset: 0,
    });
  }

  #[test]
  fn test_parse_datetime_str_out_of_range() {
    let parser = DatetimeParser::new(epoch());

    let err = parser.parse_datetime_str("5:04pm 2020-09-31").unwrap_err();
    assert_eq!(err, DatetimeParseError::DatetimeConstructionError { field_name: "day" });
  }

  #[test]
  fn test_parse_datetime_str_values_day_field_out_of_range() {
    let parser = DatetimeParser::new(epoch());

    // Note: 9999 does not fit in the range for `day: u8`.
    let err = parser.parse_datetime_str_values("2020 Jan 9999").unwrap_err();
    assert_eq!(err, DatetimeParseError::UnexpectedToken { token: String::from("9999") });
  }

  #[test]
  fn test_parse_datetime_str_with_timezone() {
    let parser = DatetimeParser::new(epoch());

    let values = parser.parse_datetime_str_values("5:04pm 2020-10-11 utc-3").unwrap().unwrap_left();
    assert_eq!(values, DatetimeValues {
      year: 2020,
      month: 10,
      day: 11,
      hour: 17,
      minute: 4,
      second: 0,
      micro: 0,
      offset: -10_800,
    });

    let values = parser.parse_datetime_str_values("5:04pm utc  +1:20:01 2020-10-11").unwrap().unwrap_left();
    assert_eq!(values, DatetimeValues {
      year: 2020,
      month: 10,
      day: 11,
      hour: 17,
      minute: 4,
      second: 0,
      micro: 0,
      offset: 4_801,
    });

    let values = parser.parse_datetime_str_values("5:04pm utc+0  2020-10-11").unwrap().unwrap_left();
    assert_eq!(values, DatetimeValues {
      year: 2020,
      month: 10,
      day: 11,
      hour: 17,
      minute: 4,
      second: 0,
      micro: 0,
      offset: 0,
    });

    let values = parser.parse_datetime_str_values("5:04pm utc-0  2020-10-11").unwrap().unwrap_left();
    assert_eq!(values, DatetimeValues {
      year: 2020,
      month: 10,
      day: 11,
      hour: 17,
      minute: 4,
      second: 0,
      micro: 0,
      offset: 0,
    });

    // Note: Timezone data is ignored if no time is specified.
    let values = parser.parse_datetime_str_values("utc+13 2020-10-11").unwrap().unwrap_right();
    assert_eq!(values, DateValues {
      year: 2020,
      month: 10,
      day: 11,
    });
  }
}
