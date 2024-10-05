
pub mod prisms;
pub mod structure;

use time::{OffsetDateTime, Date, Time, UtcOffset};

/// A `DateTime` is a date, possibly with a time attached to it. If a
/// time is attached, it will contain timezone offset information.
#[derive(Debug, Clone)]
pub struct DateTime { // TODO: Eq and Ord
  inner: DateTimeRepr,
}

#[derive(Debug, Clone)]
enum DateTimeRepr {
  Date(Date),
  Datetime(OffsetDateTime),
}

pub const DATETIME_FUNCTION_NAME: &str = "datetime";

impl DateTime {
  pub const DEFAULT_TIME: Time = Time::MIDNIGHT;
  pub const DEFAULT_OFFSET: UtcOffset = UtcOffset::UTC;

  pub fn now_utc() -> Self {
    Self::from(OffsetDateTime::now_utc())
  }

  pub fn now_local() -> Self {
    let time = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    Self::from(time)
  }

  pub fn without_time(&self) -> Date {
    match self.inner {
      DateTimeRepr::Date(d) => d,
      DateTimeRepr::Datetime(d) => d.date(),
    }
  }

  pub fn to_offset_date_time(&self) -> OffsetDateTime {
    match self.inner {
      DateTimeRepr::Date(d) => OffsetDateTime::new_in_offset(d, Self::DEFAULT_TIME, Self::DEFAULT_OFFSET),
      DateTimeRepr::Datetime(d) => d,
    }
  }
}

impl From<OffsetDateTime> for DateTime {
  fn from(inner: OffsetDateTime) -> Self {
    Self { inner: DateTimeRepr::Datetime(inner) }
  }
}

impl From<Date> for DateTime {
  fn from(inner: Date) -> Self {
    Self { inner: DateTimeRepr::Date(inner) }
  }
}
