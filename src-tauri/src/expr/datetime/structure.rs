
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

#[derive(Debug, Clone, Error)]
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
