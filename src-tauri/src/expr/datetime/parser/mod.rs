
use crate::util::regexes::WHITESPACE_RE;
use super::DateTime;
use super::structure::{DatetimeValues, DateValues, DatetimeConstructionError};

use regex::{Regex, Match};
use once_cell::sync::Lazy;
use thiserror::Error;
use either::Either;

use std::str::FromStr;
use std::num::ParseIntError;

static TIME_12_HOUR_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"(?i)(\d{1,2}):(\d{2})(?::(\d{2})(?:\.(\d{1,6}))?)?\s*([ap]\.?(?:m\.?)?|noon|mid(?:night)?)").unwrap());
static TIME_24_HOUR_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"(\d{1,2}):(\d{2})(?::(\d{2})(?:\.(\d{1,6}))?)?").unwrap());
static TIME_HOUR_ONLY_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"(?i)(\d{1,2})\s*([ap]\.?(?:m\.?)?|noon|mid(?:night)?)").unwrap());
static TIME_PERIOD_ONLY_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"(?i)noon|mid(?:night)?").unwrap());

#[derive(Debug, Clone)]
pub struct DatetimeParser {
  now: DateTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Token<'a> {
  datum: &'a str,
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum DatetimeParseError {
  #[error("{0}")]
  ParseIntError(#[from] ParseIntError),
  #[error("'noon' and 'midnight' can only be applied to the time 12:00")]
  MisappliedNoonOrMid,
  #[error("Applied AM or PM modifier to 24-hour time")]
  PeriodOn24HourTime,
  #[error("Field '{field_name}' out of range")]
  DatetimeConstructionError { field_name: &'static str },
}

/// Wrapper struct equivalent to `u32` but which parses (via
/// [`FromStr`]) right-padded to length six.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Microseconds(u32);

#[derive(Debug, Clone, PartialEq, Eq)]
struct TimeOfDay {
  hour: u8,
  minute: u8,
  second: u8,
  microsecond: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PeriodOfDay {
  Am,
  Pm,
  Noon,
  Midnight,
}

#[derive(Debug, Clone, Error)]
#[error("Could not parse period of day")]
struct PeriodOfDayParseError;

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
    let tokens: Vec<_> = tokenize_datetime_str(&input).collect();
    
    todo!()
  }
}

impl<'a> Token<'a> {
  fn new(datum: &'a str) -> Self {
    Token { datum }
  }
}

impl TimeOfDay {
  const TWELVE_O_CLOCK: TimeOfDay =
    TimeOfDay { hour: 12, minute: 0, second: 0, microsecond: 0 };

  const NOON: TimeOfDay =
    TimeOfDay { hour: 12, minute: 0, second: 0, microsecond: 0 };

  const MIDNIGHT: TimeOfDay =
    TimeOfDay { hour: 0, minute: 0, second: 0, microsecond: 0 };
}

impl PeriodOfDay {
  fn parse(input: &str) -> Option<PeriodOfDay> {
    match input.to_lowercase().as_str() {
      "am" | "a.m." | "a.m" | "a" => Some(PeriodOfDay::Am),
      "pm" | "p.m." | "p.m" | "p" => Some(PeriodOfDay::Pm),
      "noon" => Some(PeriodOfDay::Noon),
      "mid" | "midnight" => Some(PeriodOfDay::Midnight),
      _ => None,
    }
  }

  fn adjust_time(&self, mut time: TimeOfDay) -> Result<TimeOfDay, DatetimeParseError> {
    match self {
      PeriodOfDay::Am => {
        if time.hour > 12 || time.hour == 0 {
          return Err(DatetimeParseError::PeriodOn24HourTime);
        }
        if time.hour == 12 {
          time.hour = 0;
        }
      }
      PeriodOfDay::Pm => {
        if time.hour > 12 || time.hour == 0 {
          return Err(DatetimeParseError::PeriodOn24HourTime);
        }
        if time.hour < 12 {
          time.hour += 12;
        }
      }
      PeriodOfDay::Noon => {
        if time != TimeOfDay::TWELVE_O_CLOCK {
          return Err(DatetimeParseError::MisappliedNoonOrMid);
        }
        time = TimeOfDay::TWELVE_O_CLOCK;
      }
      PeriodOfDay::Midnight => {
        if time != TimeOfDay::TWELVE_O_CLOCK {
          return Err(DatetimeParseError::MisappliedNoonOrMid);
        }
        time = TimeOfDay::TWELVE_O_CLOCK;
        time.hour = 0;
      }
    }
    Ok(time)
  }
}

impl FromStr for PeriodOfDay {
  type Err = PeriodOfDayParseError;

  fn from_str(input: &str) -> Result<Self, Self::Err> {
    Self::parse(input).ok_or(PeriodOfDayParseError)
  }
}

impl FromStr for Microseconds {
  type Err = ParseIntError;

  fn from_str(input: &str) -> Result<Self, Self::Err> {
    let mut n: u32 = input.parse()?;
    if input.len() < 6 {
      n *= 10_u32.pow(6 - input.len() as u32);
    }
    Ok(Microseconds(n))
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
  WHITESPACE_RE.split(input)
    .filter(|datum| !datum.is_empty())
    .map(Token::new)
}

/// Attempts to interpret a substring of `text` as a time string.
///
/// If no time string is found, returns `Ok(None)`. If a malformed
/// time string was found, returns an appropriate `Err`.
fn search_for_time(text: &str) -> Result<Option<(TimeOfDay, Match)>, DatetimeParseError> {
  if let Some(captures) = TIME_12_HOUR_RE.captures(text) {
    let hour: u8 = required_match(captures.get(1))?;
    let minute: u8 = required_match(captures.get(2))?;
    let second: u8 = optional_match(captures.get(3), 0)?;
    let microsecond: Microseconds = optional_match(captures.get(4), Microseconds(0))?;
    let period = PeriodOfDay::parse(captures.get(5).unwrap().as_str()).unwrap();
    let time = TimeOfDay { hour, minute, second, microsecond: microsecond.0 };
    Ok(Some((period.adjust_time(time)?, captures.get(0).unwrap())))
  } else if let Some(captures) = TIME_24_HOUR_RE.captures(text) {
    let hour: u8 = required_match(captures.get(1))?;
    let minute: u8 = required_match(captures.get(2))?;
    let second: u8 = optional_match(captures.get(3), 0)?;
    let microsecond: Microseconds = optional_match(captures.get(4), Microseconds(0))?;
    let time = TimeOfDay { hour, minute, second, microsecond: microsecond.0 };
    Ok(Some((time, captures.get(0).unwrap())))
  } else if let Some(captures) = TIME_HOUR_ONLY_RE.captures(text) {
    let hour: u8 = required_match(captures.get(1))?;
    let period = PeriodOfDay::parse(captures.get(2).unwrap().as_str()).unwrap();
    let time = TimeOfDay { hour, minute: 0, second: 0, microsecond: 0 };
    Ok(Some((period.adjust_time(time)?, captures.get(0).unwrap())))
  } else if let Some(captures) = TIME_PERIOD_ONLY_RE.captures(text) {
    let m = captures.get(0).unwrap();
    let period_str = m.as_str();
    if period_str.eq_ignore_ascii_case("noon") {
      Ok(Some((TimeOfDay::NOON, m)))
    } else if period_str.eq_ignore_ascii_case("mid") || period_str.eq_ignore_ascii_case("midnight") {
      Ok(Some((TimeOfDay::MIDNIGHT, m)))
    } else {
      unreachable!();
    }
  } else {
    Ok(None)
  }
}

fn required_match<T: FromStr>(m: Option<Match>) -> Result<T, DatetimeParseError>
where DatetimeParseError: From<<T as FromStr>::Err> {
  let k = m.unwrap().as_str().parse()?;
  Ok(k)
}

fn optional_match<T: FromStr>(m: Option<Match>, default: T) -> Result<T, DatetimeParseError>
where DatetimeParseError: From<<T as FromStr>::Err> {
  let Some(m) = m else {
    return Ok(default);
  };
  Ok(m.as_str().parse()?)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_period() {
    assert_eq!(PeriodOfDay::parse("am"), Some(PeriodOfDay::Am));
    assert_eq!(PeriodOfDay::parse("a.m."), Some(PeriodOfDay::Am));
    assert_eq!(PeriodOfDay::parse("a.m"), Some(PeriodOfDay::Am));
    assert_eq!(PeriodOfDay::parse("a"), Some(PeriodOfDay::Am));
    assert_eq!(PeriodOfDay::parse("Am"), Some(PeriodOfDay::Am));
    assert_eq!(PeriodOfDay::parse("pm"), Some(PeriodOfDay::Pm));
    assert_eq!(PeriodOfDay::parse("p.m."), Some(PeriodOfDay::Pm));
    assert_eq!(PeriodOfDay::parse("p.m"), Some(PeriodOfDay::Pm));
    assert_eq!(PeriodOfDay::parse("p"), Some(PeriodOfDay::Pm));
    assert_eq!(PeriodOfDay::parse("PM"), Some(PeriodOfDay::Pm));
    assert_eq!(PeriodOfDay::parse("noon"), Some(PeriodOfDay::Noon));
    assert_eq!(PeriodOfDay::parse("mid"), Some(PeriodOfDay::Midnight));
    assert_eq!(PeriodOfDay::parse("midnight"), Some(PeriodOfDay::Midnight));
    assert_eq!(PeriodOfDay::parse("mIdNiGhT"), Some(PeriodOfDay::Midnight));
    assert_eq!(PeriodOfDay::parse("m"), None);
    assert_eq!(PeriodOfDay::parse("night"), None);
    assert_eq!(PeriodOfDay::parse("NIGHT"), None);
    assert_eq!(PeriodOfDay::parse(""), None);
    assert_eq!(PeriodOfDay::parse("X"), None);
  }

  #[test]
  fn test_period_adjust_time_am() {
    assert_eq!(
      PeriodOfDay::Am.adjust_time(TimeOfDay { hour: 3, minute: 15, second: 20, microsecond: 12 }),
      Ok(TimeOfDay { hour: 3, minute: 15, second: 20, microsecond: 12 }),
    );
    assert_eq!(
      PeriodOfDay::Am.adjust_time(TimeOfDay { hour: 12, minute: 15, second: 20, microsecond: 12 }),
      Ok(TimeOfDay { hour: 0, minute: 15, second: 20, microsecond: 12 }),
    );
    assert_eq!(
      PeriodOfDay::Am.adjust_time(TimeOfDay { hour: 0, minute: 15, second: 20, microsecond: 12 }),
      Err(DatetimeParseError::PeriodOn24HourTime),
    );
    assert_eq!(
      PeriodOfDay::Am.adjust_time(TimeOfDay { hour: 13, minute: 15, second: 20, microsecond: 12 }),
      Err(DatetimeParseError::PeriodOn24HourTime),
    );
  }

  #[test]
  fn test_period_adjust_time_pm() {
    assert_eq!(
      PeriodOfDay::Pm.adjust_time(TimeOfDay { hour: 3, minute: 15, second: 20, microsecond: 12 }),
      Ok(TimeOfDay { hour: 15, minute: 15, second: 20, microsecond: 12 }),
    );
    assert_eq!(
      PeriodOfDay::Pm.adjust_time(TimeOfDay { hour: 12, minute: 15, second: 20, microsecond: 12 }),
      Ok(TimeOfDay { hour: 12, minute: 15, second: 20, microsecond: 12 }),
    );
    assert_eq!(
      PeriodOfDay::Pm.adjust_time(TimeOfDay { hour: 11, minute: 15, second: 20, microsecond: 12 }),
      Ok(TimeOfDay { hour: 23, minute: 15, second: 20, microsecond: 12 }),
    );
    assert_eq!(
      PeriodOfDay::Pm.adjust_time(TimeOfDay { hour: 0, minute: 15, second: 20, microsecond: 12 }),
      Err(DatetimeParseError::PeriodOn24HourTime),
    );
    assert_eq!(
      PeriodOfDay::Pm.adjust_time(TimeOfDay { hour: 13, minute: 15, second: 20, microsecond: 12 }),
      Err(DatetimeParseError::PeriodOn24HourTime),
    );
  }

  #[test]
  fn test_period_adjust_time_noon() {
    assert_eq!(
      PeriodOfDay::Noon.adjust_time(TimeOfDay { hour: 12, minute: 0, second: 0, microsecond: 0 }),
      Ok(TimeOfDay { hour: 12, minute: 0, second: 0, microsecond: 0 }),
    );
    assert_eq!(
      PeriodOfDay::Noon.adjust_time(TimeOfDay { hour: 0, minute: 0, second: 0, microsecond: 0 }),
      Err(DatetimeParseError::MisappliedNoonOrMid),
    );
    assert_eq!(
      PeriodOfDay::Noon.adjust_time(TimeOfDay { hour: 12, minute: 0, second: 0, microsecond: 1 }),
      Err(DatetimeParseError::MisappliedNoonOrMid),
    );
    assert_eq!(
      PeriodOfDay::Noon.adjust_time(TimeOfDay { hour: 12, minute: 0, second: 1, microsecond: 0 }),
      Err(DatetimeParseError::MisappliedNoonOrMid),
    );
    assert_eq!(
      PeriodOfDay::Noon.adjust_time(TimeOfDay { hour: 12, minute: 1, second: 0, microsecond: 0 }),
      Err(DatetimeParseError::MisappliedNoonOrMid),
    );
    assert_eq!(
      PeriodOfDay::Noon.adjust_time(TimeOfDay { hour: 11, minute: 0, second: 0, microsecond: 0 }),
      Err(DatetimeParseError::MisappliedNoonOrMid),
    );
  }

  #[test]
  fn test_period_adjust_time_midnight() {
    assert_eq!(
      PeriodOfDay::Midnight.adjust_time(TimeOfDay { hour: 12, minute: 0, second: 0, microsecond: 0 }),
      Ok(TimeOfDay { hour: 0, minute: 0, second: 0, microsecond: 0 }),
    );
    assert_eq!(
      PeriodOfDay::Midnight.adjust_time(TimeOfDay { hour: 0, minute: 0, second: 0, microsecond: 0 }),
      Err(DatetimeParseError::MisappliedNoonOrMid),
    );
    assert_eq!(
      PeriodOfDay::Midnight.adjust_time(TimeOfDay { hour: 12, minute: 0, second: 0, microsecond: 1 }),
      Err(DatetimeParseError::MisappliedNoonOrMid),
    );
    assert_eq!(
      PeriodOfDay::Midnight.adjust_time(TimeOfDay { hour: 12, minute: 0, second: 1, microsecond: 0 }),
      Err(DatetimeParseError::MisappliedNoonOrMid),
    );
    assert_eq!(
      PeriodOfDay::Midnight.adjust_time(TimeOfDay { hour: 12, minute: 1, second: 0, microsecond: 0 }),
      Err(DatetimeParseError::MisappliedNoonOrMid),
    );
    assert_eq!(
      PeriodOfDay::Midnight.adjust_time(TimeOfDay { hour: 11, minute: 0, second: 0, microsecond: 0 }),
      Err(DatetimeParseError::MisappliedNoonOrMid),
    );
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

    let input = "0 1 2a b3 4C4";
    assert_eq!(
      tokenize_datetime_str(input).collect::<Vec<_>>(),
      vec![Token { datum: "0" }, Token { datum: "1" }, Token { datum: "2a" },
           Token { datum: "b3" }, Token { datum: "4C4" }],
    );
  }

  #[test]
  fn test_search_for_time_12_hour() {
    let input = "xxx 4:15pm yyy";
    let (time, m) = search_for_time(input).unwrap().unwrap();
    assert_eq!(time, TimeOfDay { hour: 16, minute: 15, second: 0, microsecond: 0 });
    assert_eq!(m.start(), 4);
    assert_eq!(m.end(), 10);

    let input = "xxx 04:15pm yyy";
    let (time, m) = search_for_time(input).unwrap().unwrap();
    assert_eq!(time, TimeOfDay { hour: 16, minute: 15, second: 0, microsecond: 0 });
    assert_eq!(m.start(), 4);
    assert_eq!(m.end(), 11);

    let input = "xxx 04:15A.M. yyy";
    let (time, m) = search_for_time(input).unwrap().unwrap();
    assert_eq!(time, TimeOfDay { hour: 4, minute: 15, second: 0, microsecond: 0 });
    assert_eq!(m.start(), 4);
    assert_eq!(m.end(), 13);

    let input = "xxx 04:15:16A.M yyy";
    let (time, m) = search_for_time(input).unwrap().unwrap();
    assert_eq!(time, TimeOfDay { hour: 4, minute: 15, second: 16, microsecond: 0 });
    assert_eq!(m.start(), 4);
    assert_eq!(m.end(), 15);

    let input = "xxx 04:15:16  A.M. yyy";
    let (time, m) = search_for_time(input).unwrap().unwrap();
    assert_eq!(time, TimeOfDay { hour: 4, minute: 15, second: 16, microsecond: 0 });
    assert_eq!(m.start(), 4);
    assert_eq!(m.end(), 18);

    let input = "xxx 04:15:00.21A.M yyy";
    let (time, m) = search_for_time(input).unwrap().unwrap();
    assert_eq!(time, TimeOfDay { hour: 4, minute: 15, second: 00, microsecond: 210_000 });
    assert_eq!(m.start(), 4);
    assert_eq!(m.end(), 18);

    let input = "xxx 12:00mid yyy";
    let (time, m) = search_for_time(input).unwrap().unwrap();
    assert_eq!(time, TimeOfDay { hour: 0, minute: 0, second: 0, microsecond: 0 });
    assert_eq!(m.start(), 4);
    assert_eq!(m.end(), 12);

    let input = "xxx 12:00MIDNIGHT yyy";
    let (time, m) = search_for_time(input).unwrap().unwrap();
    assert_eq!(time, TimeOfDay { hour: 0, minute: 0, second: 0, microsecond: 0 });
    assert_eq!(m.start(), 4);
    assert_eq!(m.end(), 17);

    let input = "xxx 12:00noon yyy";
    let (time, m) = search_for_time(input).unwrap().unwrap();
    assert_eq!(time, TimeOfDay { hour: 12, minute: 0, second: 0, microsecond: 0 });
    assert_eq!(m.start(), 4);
    assert_eq!(m.end(), 13);

    let input = "xxx 12:01noon yyy";
    let err = search_for_time(input).unwrap_err();
    assert_eq!(err, DatetimeParseError::MisappliedNoonOrMid);

    let input = "xxx 12:01midnight yyy";
    let err = search_for_time(input).unwrap_err();
    assert_eq!(err, DatetimeParseError::MisappliedNoonOrMid);
  }

  #[test]
  fn test_search_for_time_24_hour() {
    let input = "xxx 4:15 yyy";
    let (time, m) = search_for_time(input).unwrap().unwrap();
    assert_eq!(time, TimeOfDay { hour: 4, minute: 15, second: 0, microsecond: 0 });
    assert_eq!(m.start(), 4);
    assert_eq!(m.end(), 8);

    let input = "xxx 04:15 yyy";
    let (time, m) = search_for_time(input).unwrap().unwrap();
    assert_eq!(time, TimeOfDay { hour: 4, minute: 15, second: 0, microsecond: 0 });
    assert_eq!(m.start(), 4);
    assert_eq!(m.end(), 9);

    let input = "xxx 16:15 yyy";
    let (time, m) = search_for_time(input).unwrap().unwrap();
    assert_eq!(time, TimeOfDay { hour: 16, minute: 15, second: 0, microsecond: 0 });
    assert_eq!(m.start(), 4);
    assert_eq!(m.end(), 9);

    let input = "xxx 23:59 yyy";
    let (time, m) = search_for_time(input).unwrap().unwrap();
    assert_eq!(time, TimeOfDay { hour: 23, minute: 59, second: 0, microsecond: 0 });
    assert_eq!(m.start(), 4);
    assert_eq!(m.end(), 9);

    let input = "xxx 0:00 yyy";
    let (time, m) = search_for_time(input).unwrap().unwrap();
    assert_eq!(time, TimeOfDay { hour: 0, minute: 0, second: 0, microsecond: 0 });
    assert_eq!(m.start(), 4);
    assert_eq!(m.end(), 8);

    let input = "xxx 04:15:16 yyy";
    let (time, m) = search_for_time(input).unwrap().unwrap();
    assert_eq!(time, TimeOfDay { hour: 4, minute: 15, second: 16, microsecond: 0 });
    assert_eq!(m.start(), 4);
    assert_eq!(m.end(), 12);

    let input = "xxx 04:15:00.21 yyy";
    let (time, m) = search_for_time(input).unwrap().unwrap();
    assert_eq!(time, TimeOfDay { hour: 4, minute: 15, second: 00, microsecond: 210_000 });
    assert_eq!(m.start(), 4);
    assert_eq!(m.end(), 15);

    let input = "xxx 04:15:00.213 yyy";
    let (time, m) = search_for_time(input).unwrap().unwrap();
    assert_eq!(time, TimeOfDay { hour: 4, minute: 15, second: 00, microsecond: 213_000 });
    assert_eq!(m.start(), 4);
    assert_eq!(m.end(), 16);

    let input = "xxx 04:15:00.213456 yyy";
    let (time, m) = search_for_time(input).unwrap().unwrap();
    assert_eq!(time, TimeOfDay { hour: 4, minute: 15, second: 00, microsecond: 213_456 });
    assert_eq!(m.start(), 4);
    assert_eq!(m.end(), 19);

    let input = "xxx 12:00 yyy";
    let (time, m) = search_for_time(input).unwrap().unwrap();
    assert_eq!(time, TimeOfDay { hour: 12, minute: 0, second: 0, microsecond: 0 });
    assert_eq!(m.start(), 4);
    assert_eq!(m.end(), 9);
  }

  #[test]
  fn test_search_for_time_hour_only() {
    let input = "xxx 4pm yyy";
    let (time, m) = search_for_time(input).unwrap().unwrap();
    assert_eq!(time, TimeOfDay { hour: 16, minute: 0, second: 0, microsecond: 0 });
    assert_eq!(m.start(), 4);
    assert_eq!(m.end(), 7);

    let input = "xxx 04A.M. yyy";
    let (time, m) = search_for_time(input).unwrap().unwrap();
    assert_eq!(time, TimeOfDay { hour: 4, minute: 0, second: 0, microsecond: 0 });
    assert_eq!(m.start(), 4);
    assert_eq!(m.end(), 10);

    let input = "xxx 04A.m yyy";
    let (time, m) = search_for_time(input).unwrap().unwrap();
    assert_eq!(time, TimeOfDay { hour: 4, minute: 0, second: 0, microsecond: 0 });
    assert_eq!(m.start(), 4);
    assert_eq!(m.end(), 9);

    let input = "xxx 11p yyy";
    let (time, m) = search_for_time(input).unwrap().unwrap();
    assert_eq!(time, TimeOfDay { hour: 23, minute: 0, second: 0, microsecond: 0 });
    assert_eq!(m.start(), 4);
    assert_eq!(m.end(), 7);

    let input = "xxx 12a yyy";
    let (time, m) = search_for_time(input).unwrap().unwrap();
    assert_eq!(time, TimeOfDay { hour: 0, minute: 0, second: 0, microsecond: 0 });
    assert_eq!(m.start(), 4);
    assert_eq!(m.end(), 7);

    let input = "xxx 12p yyy";
    let (time, m) = search_for_time(input).unwrap().unwrap();
    assert_eq!(time, TimeOfDay { hour: 12, minute: 0, second: 0, microsecond: 0 });
    assert_eq!(m.start(), 4);
    assert_eq!(m.end(), 7);

    let input = "xxx 12noon yyy";
    let (time, m) = search_for_time(input).unwrap().unwrap();
    assert_eq!(time, TimeOfDay { hour: 12, minute: 0, second: 0, microsecond: 0 });
    assert_eq!(m.start(), 4);
    assert_eq!(m.end(), 10);

    let input = "xxx 12Mid yyy";
    let (time, m) = search_for_time(input).unwrap().unwrap();
    assert_eq!(time, TimeOfDay { hour: 0, minute: 0, second: 0, microsecond: 0 });
    assert_eq!(m.start(), 4);
    assert_eq!(m.end(), 9);

    let input = "xxx 12Midnight yyy";
    let (time, m) = search_for_time(input).unwrap().unwrap();
    assert_eq!(time, TimeOfDay { hour: 0, minute: 0, second: 0, microsecond: 0 });
    assert_eq!(m.start(), 4);
    assert_eq!(m.end(), 14);

    let input = "xxx 12  Midnight yyy";
    let (time, m) = search_for_time(input).unwrap().unwrap();
    assert_eq!(time, TimeOfDay { hour: 0, minute: 0, second: 0, microsecond: 0 });
    assert_eq!(m.start(), 4);
    assert_eq!(m.end(), 16);

    let input = "xxx 11Midnight yyy";
    let err = search_for_time(input).unwrap_err();
    assert_eq!(err, DatetimeParseError::MisappliedNoonOrMid);
  }

  #[test]
  fn test_search_for_period_hour_only() {
    let input = "xxx NOON yyy";
    let (time, m) = search_for_time(input).unwrap().unwrap();
    assert_eq!(time, TimeOfDay { hour: 12, minute: 0, second: 0, microsecond: 0 });
    assert_eq!(m.start(), 4);
    assert_eq!(m.end(), 8);

    let input = "xxx midnight yyy";
    let (time, m) = search_for_time(input).unwrap().unwrap();
    assert_eq!(time, TimeOfDay { hour: 0, minute: 0, second: 0, microsecond: 0 });
    assert_eq!(m.start(), 4);
    assert_eq!(m.end(), 12);

    let input = "xxx mid yyy";
    let (time, m) = search_for_time(input).unwrap().unwrap();
    assert_eq!(time, TimeOfDay { hour: 0, minute: 0, second: 0, microsecond: 0 });
    assert_eq!(m.start(), 4);
    assert_eq!(m.end(), 7);

    let input = "xxx am yyy";
    assert!(search_for_time(input).unwrap().is_none());

    let input = "xxx pm yyy";
    assert!(search_for_time(input).unwrap().is_none());
  }

  #[test]
  fn test_microseconds_from_str() {
    assert_eq!("19".parse::<Microseconds>().unwrap(), Microseconds(190000));
    assert_eq!("0".parse::<Microseconds>().unwrap(), Microseconds(0));
    assert_eq!("123456789".parse::<Microseconds>().unwrap(), Microseconds(123456789));
    assert_eq!("09".parse::<Microseconds>().unwrap(), Microseconds(90000));
    assert_eq!("0009".parse::<Microseconds>().unwrap(), Microseconds(900));
  }
}
