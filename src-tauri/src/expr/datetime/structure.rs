
use super::DateTime;
use crate::util::prism::ErrorWithPayload;

use time::{OffsetDateTime, Date, Time, Month, UtcOffset};
use time::error::ComponentRange;
use thiserror::Error;

use std::fmt::Debug;

/// A named collection of integers which will likely be treated as a
/// date together with a timestamp soon. This structure does NOT have
/// invariants and does not guarantee that the contained numbers form
/// a valid datetime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatetimeValues {
  pub year: i32,
  pub month: u8,
  pub day: u8,
  pub hour: u8,
  pub minute: u8,
  pub second: u8,
  pub micro: u32,
  pub offset: i32,
}

/// A named collection of integers which will likely be treated as a
/// date without a timestamp soon. This structure does NOT have
/// invariants and does not guarantee that the contained numbers form
/// a valid date.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateValues {
  pub year: i32,
  pub month: u8,
  pub day: u8,
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("Construction of datetime value failed: {inner_error}")]
pub struct DatetimeConstructionError<T> {
  original_value: T,
  #[source]
  inner_error: ComponentRange,
}

impl DatetimeValues {
  pub fn without_time(self) -> DateValues {
    DateValues {
      year: self.year,
      month: self.month,
      day: self.day,
    }
  }
}

impl<T> DatetimeConstructionError<T> {
  pub fn new(original_value: T, inner_error: ComponentRange) -> Self {
    Self { original_value, inner_error }
  }

  pub fn name(&self) -> &'static str {
    self.inner_error.name()
  }
}

impl<T: Debug> ErrorWithPayload<T> for DatetimeConstructionError<T> {
  fn recover_payload(self) -> T {
    self.original_value
  }
}

impl TryFrom<DateValues> for Date {
  type Error = DatetimeConstructionError<DateValues>;

  fn try_from(date: DateValues) -> Result<Date, Self::Error> {
    fn parse(date: &DateValues) -> Result<Date, ComponentRange> {
      let month = Month::try_from(date.month)?;
      Date::from_calendar_date(date.year, month, date.day)
    }
    parse(&date).map_err(|err| DatetimeConstructionError::new(date, err))
  }
}

impl From<Date> for DateValues {
  fn from(date: Date) -> Self {
    DateValues {
      year: date.year(),
      month: date.month() as u8,
      day: date.day(),
    }
  }
}

impl TryFrom<DatetimeValues> for OffsetDateTime {
  type Error = DatetimeConstructionError<DatetimeValues>;

  fn try_from(datetime: DatetimeValues) -> Result<OffsetDateTime, Self::Error> {
    fn parse(datetime: &DatetimeValues) -> Result<OffsetDateTime, ComponentRange> {
      let month = Month::try_from(datetime.month)?;
      let date = Date::from_calendar_date(datetime.year, month, datetime.day)?;
      let time = Time::from_hms_micro(datetime.hour, datetime.minute, datetime.second, datetime.micro)?;
      let offset = UtcOffset::from_whole_seconds(datetime.offset)?;
      Ok(OffsetDateTime::new_in_offset(date, time, offset))
    }
    parse(&datetime).map_err(|err| DatetimeConstructionError::new(datetime, err))
  }
}

impl From<OffsetDateTime> for DatetimeValues {
  fn from(date: OffsetDateTime) -> Self {
    DatetimeValues {
      year: date.year(),
      month: date.month() as u8,
      day: date.day(),
      hour: date.hour(),
      minute: date.minute(),
      second: date.second(),
      micro: date.microsecond(),
      offset: date.offset().whole_seconds(),
    }
  }
}

impl TryFrom<DateValues> for DateTime {
  type Error = DatetimeConstructionError<DateValues>;

  fn try_from(date: DateValues) -> Result<DateTime, Self::Error> {
    Date::try_from(date).map(DateTime::from)
  }
}

impl TryFrom<DatetimeValues> for DateTime {
  type Error = DatetimeConstructionError<DatetimeValues>;

  fn try_from(datetime: DatetimeValues) -> Result<DateTime, Self::Error> {
    OffsetDateTime::try_from(datetime).map(DateTime::from)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_datetime_values_without_time() {
    let values = DatetimeValues {
      year: 2018,
      month: 1,
      day: 2,
      hour: 3,
      minute: 4,
      second: 5,
      micro: 6,
      offset: 7,
    };
    assert_eq!(values.without_time(), DateValues { year: 2018, month: 1, day: 2 });
  }

  #[test]
  fn test_try_from_date_values_successfully() {
    let values = DateValues { year: 2018, month: 1, day: 2 };
    assert_eq!(Date::try_from(values), Ok(Date::from_calendar_date(2018, Month::January, 2).unwrap()));
    let values = DateValues { year: 2018, month: 1, day: 31 };
    assert_eq!(Date::try_from(values), Ok(Date::from_calendar_date(2018, Month::January, 31).unwrap()));
    let values = DateValues { year: 2018, month: 2, day: 28 };
    assert_eq!(Date::try_from(values), Ok(Date::from_calendar_date(2018, Month::February, 28).unwrap()));
    let values = DateValues { year: 2020, month: 2, day: 29 };
    assert_eq!(Date::try_from(values), Ok(Date::from_calendar_date(2020, Month::February, 29).unwrap()));
    let values = DateValues { year: 2021, month: 12, day: 19 };
    assert_eq!(Date::try_from(values), Ok(Date::from_calendar_date(2021, Month::December, 19).unwrap()));
    let values = DateValues { year: 0, month: 2, day: 3 };
    assert_eq!(Date::try_from(values), Ok(Date::from_calendar_date(0, Month::February, 3).unwrap()));
  }

  #[test]
  fn test_try_from_datetime_values_successfully() {
    let values = DatetimeValues { year: 2018, month: 1, day: 2, hour: 3, minute: 4, second: 5, micro: 6, offset: 0 };
    assert_eq!(OffsetDateTime::try_from(values), Ok(OffsetDateTime::new_in_offset(
      Date::from_calendar_date(2018, Month::January, 2).unwrap(),
      Time::from_hms_micro(3, 4, 5, 6).unwrap(),
      UtcOffset::from_whole_seconds(0).unwrap(),
    )));
    let values = DatetimeValues { year: -100, month: 1, day: 2, hour: 3, minute: 4, second: 5, micro: 6, offset: 0 };
    assert_eq!(OffsetDateTime::try_from(values), Ok(OffsetDateTime::new_in_offset(
      Date::from_calendar_date(-100, Month::January, 2).unwrap(),
      Time::from_hms_micro(3, 4, 5, 6).unwrap(),
      UtcOffset::from_whole_seconds(0).unwrap(),
    )));
    let values = DatetimeValues { year: 0, month: 1, day: 2, hour: 3, minute: 4, second: 5, micro: 6, offset: 0 };
    assert_eq!(OffsetDateTime::try_from(values), Ok(OffsetDateTime::new_in_offset(
      Date::from_calendar_date(0, Month::January, 2).unwrap(),
      Time::from_hms_micro(3, 4, 5, 6).unwrap(),
      UtcOffset::from_whole_seconds(0).unwrap(),
    )));
    let values = DatetimeValues { year: 2018, month: 12, day: 2, hour: 3, minute: 4, second: 5, micro: 6, offset: 0 };
    assert_eq!(OffsetDateTime::try_from(values), Ok(OffsetDateTime::new_in_offset(
      Date::from_calendar_date(2018, Month::December, 2).unwrap(),
      Time::from_hms_micro(3, 4, 5, 6).unwrap(),
      UtcOffset::from_whole_seconds(0).unwrap(),
    )));
    let values = DatetimeValues { year: 2018, month: 8, day: 31, hour: 3, minute: 4, second: 5, micro: 6, offset: 0 };
    assert_eq!(OffsetDateTime::try_from(values), Ok(OffsetDateTime::new_in_offset(
      Date::from_calendar_date(2018, Month::August, 31).unwrap(),
      Time::from_hms_micro(3, 4, 5, 6).unwrap(),
      UtcOffset::from_whole_seconds(0).unwrap(),
    )));
    let values = DatetimeValues { year: 2018, month: 8, day: 1, hour: 3, minute: 4, second: 5, micro: 6, offset: 0 };
    assert_eq!(OffsetDateTime::try_from(values), Ok(OffsetDateTime::new_in_offset(
      Date::from_calendar_date(2018, Month::August, 1).unwrap(),
      Time::from_hms_micro(3, 4, 5, 6).unwrap(),
      UtcOffset::from_whole_seconds(0).unwrap(),
    )));
    let values = DatetimeValues { year: 2018, month: 8, day: 1, hour: 0, minute: 0, second: 0, micro: 0, offset: 0 };
    assert_eq!(OffsetDateTime::try_from(values), Ok(OffsetDateTime::new_in_offset(
      Date::from_calendar_date(2018, Month::August, 1).unwrap(),
      Time::from_hms_micro(0, 0, 0, 0).unwrap(),
      UtcOffset::from_whole_seconds(0).unwrap(),
    )));
    let values = DatetimeValues { year: 2018, month: 8, day: 1, hour: 23, minute: 59, second: 59, micro: 999_999, offset: 0 };
    assert_eq!(OffsetDateTime::try_from(values), Ok(OffsetDateTime::new_in_offset(
      Date::from_calendar_date(2018, Month::August, 1).unwrap(),
      Time::from_hms_micro(23, 59, 59, 999_999).unwrap(),
      UtcOffset::from_whole_seconds(0).unwrap(),
    )));
    let values = DatetimeValues { year: 2018, month: 8, day: 1, hour: 23, minute: 59, second: 59, micro: 999_999, offset: 93_599 };
    assert_eq!(OffsetDateTime::try_from(values), Ok(OffsetDateTime::new_in_offset(
      Date::from_calendar_date(2018, Month::August, 1).unwrap(),
      Time::from_hms_micro(23, 59, 59, 999_999).unwrap(),
      UtcOffset::from_whole_seconds(93_599).unwrap(),
    )));
    let values = DatetimeValues { year: 2018, month: 8, day: 1, hour: 23, minute: 59, second: 59, micro: 999_999, offset: -93_599 };
    assert_eq!(OffsetDateTime::try_from(values), Ok(OffsetDateTime::new_in_offset(
      Date::from_calendar_date(2018, Month::August, 1).unwrap(),
      Time::from_hms_micro(23, 59, 59, 999_999).unwrap(),
      UtcOffset::from_whole_seconds(-93_599).unwrap(),
    )));
  }

  #[test]
  fn test_try_from_date_values_failure() {
    let values = DateValues { year: i32::MAX, month: 1, day: 1 };
    let err = Date::try_from(values).unwrap_err();
    assert_eq!(err.name(), "year");

    let values = DateValues { year: 1990, month: 0, day: 1 };
    let err = Date::try_from(values).unwrap_err();
    assert_eq!(err.name(), "month");

    let values = DateValues { year: 1990, month: 13, day: 1 };
    let err = Date::try_from(values).unwrap_err();
    assert_eq!(err.name(), "month");

    let values = DateValues { year: 1990, month: 4, day: 31 };
    let err = Date::try_from(values).unwrap_err();
    assert_eq!(err.name(), "day");

    let values = DateValues { year: 1990, month: 2, day: 29 };
    let err = Date::try_from(values).unwrap_err();
    assert_eq!(err.name(), "day");

    let values = DateValues { year: 2020, month: 2, day: 30 };
    let err = Date::try_from(values).unwrap_err();
    assert_eq!(err.name(), "day");

    let values = DateValues { year: 1991, month: 2, day: 0 };
    let err = Date::try_from(values).unwrap_err();
    assert_eq!(err.name(), "day");
  }

#[test]
  fn test_try_from_datetime_values_failure() {
    let values = DatetimeValues { year: i32::MAX, month: 1, day: 2, hour: 3,
                                  minute: 4, second: 5, micro: 6, offset: 0 };
    let err = OffsetDateTime::try_from(values).unwrap_err();
    assert_eq!(err.name(), "year");

    let values = DatetimeValues { year: 2001, month: 0, day: 2, hour: 3,
                                  minute: 4, second: 5, micro: 6, offset: 0 };
    let err = OffsetDateTime::try_from(values).unwrap_err();
    assert_eq!(err.name(), "month");

    let values = DatetimeValues { year: 2001, month: 13, day: 2, hour: 3,
                                  minute: 4, second: 5, micro: 6, offset: 0 };
    let err = OffsetDateTime::try_from(values).unwrap_err();
    assert_eq!(err.name(), "month");

    let values = DatetimeValues { year: 2001, month: 12, day: 0, hour: 3,
                                  minute: 4, second: 5, micro: 6, offset: 0 };
    let err = OffsetDateTime::try_from(values).unwrap_err();
    assert_eq!(err.name(), "day");

    let values = DatetimeValues { year: 2001, month: 12, day: 32, hour: 3,
                                  minute: 4, second: 5, micro: 6, offset: 0 };
    let err = OffsetDateTime::try_from(values).unwrap_err();
    assert_eq!(err.name(), "day");

    let values = DatetimeValues { year: 2001, month: 11, day: 31, hour: 3,
                                  minute: 4, second: 5, micro: 6, offset: 0 };
    let err = OffsetDateTime::try_from(values).unwrap_err();
    assert_eq!(err.name(), "day");

    let values = DatetimeValues { year: 2001, month: 11, day: 19, hour: 24,
                                  minute: 4, second: 5, micro: 6, offset: 0 };
    let err = OffsetDateTime::try_from(values).unwrap_err();
    assert_eq!(err.name(), "hour");

    let values = DatetimeValues { year: 2001, month: 11, day: 19, hour: 23,
                                  minute: 60, second: 5, micro: 6, offset: 0 };
    let err = OffsetDateTime::try_from(values).unwrap_err();
    assert_eq!(err.name(), "minute");

    let values = DatetimeValues { year: 2001, month: 11, day: 19, hour: 23,
                                  minute: 30, second: 60, micro: 6, offset: 0 };
    let err = OffsetDateTime::try_from(values).unwrap_err();
    assert_eq!(err.name(), "second");

    let values = DatetimeValues { year: 2001, month: 11, day: 19, hour: 23,
                                  minute: 30, second: 30, micro: 1_000_000, offset: 0 };
    let err = OffsetDateTime::try_from(values).unwrap_err();
    assert_eq!(err.name(), "microsecond");

    let values = DatetimeValues { year: 2001, month: 11, day: 19, hour: 23,
                                  minute: 30, second: 30, micro: 10_000, offset: 94_000 };
    let err = OffsetDateTime::try_from(values).unwrap_err();
    assert_eq!(err.name(), "seconds");

    let values = DatetimeValues { year: 2001, month: 11, day: 19, hour: 23,
                                  minute: 30, second: 30, micro: 10_000, offset: -94_000 };
    let err = OffsetDateTime::try_from(values).unwrap_err();
    assert_eq!(err.name(), "seconds");
  }
}
