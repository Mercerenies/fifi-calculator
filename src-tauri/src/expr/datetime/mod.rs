
pub mod prisms;
pub mod structure;
pub mod timezone;

use timezone::TimezoneOffset;
use crate::util::stricteq::StrictEq;

use time::{OffsetDateTime, Date, Time, UtcOffset, Month};

/// A `DateTime` is a date, possibly with a time attached to it. If a
/// time is attached, it will contain timezone offset information.
///
/// For the purposes of the [`Eq`] and [`Ord`] traits, a date without
/// a timestamp is considered equivalent to that date at midnight UTC.
/// Two datetimes are considered equal when they represent the same
/// time, even if the timezone offsets are different. So `3:30pm` in
/// UTC would compare equal to `4:30pm` in BST, but not to `3:30pm` in
/// BST.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DateTime {
  inner: DateTimeRepr,
}

#[derive(Debug, Clone)]
enum DateTimeRepr {
  Date(Date),
  Datetime(OffsetDateTime),
}

pub const DATETIME_FUNCTION_NAME: &str = "datetime";

/// The valid arities of a `datetime` call. A `datetime` call with
/// three arguments consists of a year, month, and day. A `datetime`
/// call with eight arguments additionally consists of hours, minutes,
/// seconds, microseconds, and timezone offset.
pub const DATETIME_ARITIES: [usize; 2] = [3, 8];

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
    self.inner.to_offset_date_time()
  }

  pub fn has_time(&self) -> bool {
    matches!(&self.inner, DateTimeRepr::Datetime(_))
  }

  pub fn date(&self) -> Date {
    self.without_time()
  }

  pub fn time(&self) -> Time {
    match self.inner {
      DateTimeRepr::Date(_) => Time::MIDNIGHT,
      DateTimeRepr::Datetime(d) => d.time(),
    }
  }

  pub fn offset(&self) -> UtcOffset {
    match self.inner {
      DateTimeRepr::Date(_) => UtcOffset::UTC,
      DateTimeRepr::Datetime(d) => d.offset(),
    }
  }

  pub fn timezone_offset(&self) -> TimezoneOffset {
    TimezoneOffset(self.offset())
  }

  pub fn year(&self) -> i32 {
    self.date().year()
  }

  pub fn month(&self) -> Month {
    self.date().month()
  }

  pub fn day(&self) -> u8 {
    self.date().day()
  }

  pub fn hour(&self) -> u8 {
    self.time().hour()
  }

  pub fn minute(&self) -> u8 {
    self.time().minute()
  }

  pub fn second(&self) -> u8 {
    self.time().second()
  }

  pub fn microsecond(&self) -> u32 {
    self.time().microsecond()
  }
}

impl DateTimeRepr {
  fn to_offset_date_time(&self) -> OffsetDateTime {
    match self {
      DateTimeRepr::Date(d) => OffsetDateTime::new_in_offset(*d, DateTime::DEFAULT_TIME, DateTime::DEFAULT_OFFSET),
      DateTimeRepr::Datetime(d) => *d,
    }
  }
}

impl PartialEq for DateTimeRepr {
  fn eq(&self, other: &Self) -> bool {
    self.to_offset_date_time() == other.to_offset_date_time()
  }
}

impl Eq for DateTimeRepr {}

impl StrictEq for DateTimeRepr {
  fn strict_eq(&self, other: &Self) -> bool {
    match (self, other) {
      (DateTimeRepr::Date(a), DateTimeRepr::Date(b)) => a == b,
      (DateTimeRepr::Datetime(a), DateTimeRepr::Datetime(b)) => a == b,
      _ => false,
    }
  }
}

impl PartialOrd for DateTimeRepr {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for DateTimeRepr {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.to_offset_date_time().cmp(&other.to_offset_date_time())
  }
}

impl StrictEq for DateTime {
  fn strict_eq(&self, other: &Self) -> bool {
    self.inner.strict_eq(&other.inner)
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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{assert_strict_eq, assert_strict_ne};

  use time::Month;

  use std::time::Duration;

  fn days(n: u64) -> Duration {
    Duration::from_secs(n * 24 * 60 * 60)
  }

  #[test]
  fn test_without_time() {
    let date = Date::from_calendar_date(2001, Month::February, 13).unwrap();
    let datetime = OffsetDateTime::new_utc(date, Time::from_hms(12, 13, 14).unwrap());
    assert_eq!(DateTime::from(datetime).without_time(), date);
    assert_eq!(DateTime::from(date).without_time(), date);
  }

  #[test]
  fn test_to_offset_date_time() {
    let date = Date::from_calendar_date(2001, Month::February, 13).unwrap();
    let datetime = OffsetDateTime::new_utc(date, Time::from_hms(12, 13, 14).unwrap());
    assert_eq!(DateTime::from(datetime).to_offset_date_time(), datetime);
    assert_eq!(
      DateTime::from(date).to_offset_date_time(),
      OffsetDateTime::new_utc(date, Time::MIDNIGHT),
    );
  }

  #[test]
  fn test_has_time() {
    let date = DateTime::from(Date::from_calendar_date(2001, Month::February, 13).unwrap());
    assert!(!date.has_time());
    let datetime = DateTime::from(OffsetDateTime::UNIX_EPOCH);
    assert!(datetime.has_time());
  }

  #[test]
  fn test_eq_in_utc() {
    let epoch = DateTime::from(OffsetDateTime::UNIX_EPOCH);
    assert_eq!(epoch, epoch);
    assert_eq!(epoch, DateTime::from(OffsetDateTime::UNIX_EPOCH));
    assert_eq!(epoch, DateTime::from(epoch.without_time()));
    assert_eq!(DateTime::from(epoch.without_time()), DateTime::from(epoch.without_time()));
    assert_eq!(DateTime::from(epoch.without_time()), epoch);
    assert_ne!(
      DateTime::from(epoch.without_time()),
      DateTime::from(Date::from_calendar_date(2001, Month::February, 13).unwrap()),
    );
  }

  #[test]
  fn test_strict_eq() {
    let epoch = DateTime::from(OffsetDateTime::UNIX_EPOCH);
    assert_strict_eq!(epoch, epoch);
    assert_strict_eq!(epoch, DateTime::from(OffsetDateTime::UNIX_EPOCH));
    assert_strict_ne!(epoch, DateTime::from(epoch.without_time()));
    assert_strict_eq!(DateTime::from(epoch.without_time()), DateTime::from(epoch.without_time()));
    assert_strict_ne!(DateTime::from(epoch.without_time()), epoch);
    assert_strict_ne!(
      DateTime::from(epoch.without_time()),
      DateTime::from(Date::from_calendar_date(2001, Month::February, 13).unwrap()),
    );
  }

  #[test]
  fn test_eq_with_timezones() {
    let datetime1_tz1 = DateTime::from(OffsetDateTime::new_in_offset(
      Date::from_calendar_date(2001, Month::February, 13).unwrap(),
      Time::from_hms(12, 13, 14).unwrap(),
      UtcOffset::from_hms(1, 0, 0).unwrap(),
    ));
    let datetime1_tz2 = DateTime::from(OffsetDateTime::new_in_offset(
      Date::from_calendar_date(2001, Month::February, 13).unwrap(),
      Time::from_hms(11, 13, 14).unwrap(),
      UtcOffset::from_hms(0, 0, 0).unwrap(),
    ));
    let datetime2_tz2 = DateTime::from(OffsetDateTime::new_in_offset(
      Date::from_calendar_date(2001, Month::February, 13).unwrap(),
      Time::from_hms(12, 13, 14).unwrap(),
      UtcOffset::from_hms(0, 0, 0).unwrap(),
    ));
    assert_eq!(datetime1_tz1, datetime1_tz1);
    assert_eq!(datetime1_tz1, datetime1_tz2);
    assert_ne!(datetime2_tz2, datetime1_tz2);
    assert_ne!(datetime2_tz2, datetime1_tz1);
  }

  #[test]
  fn test_ord_in_utc() {
    let date1 = DateTime::from(OffsetDateTime::UNIX_EPOCH);
    let date2 = DateTime::from(OffsetDateTime::UNIX_EPOCH + days(2));
    assert!(date1 < date2);
    assert!(date1 <= date1);
  }

  #[test]
  fn test_ord_on_date() {
    let base_date = Date::from_calendar_date(2003, Month::April, 19).unwrap();
    let date1 = DateTime::from(base_date);
    let date2 = DateTime::from(base_date + days(1));
    let date3 = DateTime::from(base_date - days(1));
    assert!(date1 < date2);
    assert!(date1 > date3);
  }

  #[test]
  fn test_ord_on_mixed_date_and_datetime() {
    let base_date = Date::from_calendar_date(2003, Month::April, 19).unwrap();
    let date1 = DateTime::from(base_date);
    let date2 = DateTime::from(OffsetDateTime::new_utc(base_date, Time::MIDNIGHT));
    let date3 = DateTime::from(OffsetDateTime::new_utc(base_date, Time::MIDNIGHT) + Duration::from_secs(10));
    let date4 = DateTime::from(OffsetDateTime::new_utc(base_date, Time::MIDNIGHT) - Duration::from_secs(10));
    assert_eq!(date1, date2);
    assert!(date1 < date3);
    assert!(date1 > date4);
  }

  #[test]
  fn test_ord_on_mixed_timezones() {
    let base_date = Date::from_calendar_date(2003, Month::April, 19).unwrap();
    let date1 = DateTime::from(OffsetDateTime::new_utc(base_date, Time::from_hms(3, 4, 5).unwrap()));
    let date2 = DateTime::from(OffsetDateTime::new_in_offset(base_date, Time::from_hms(3, 4, 5).unwrap(), UtcOffset::from_hms(1, 0, 0).unwrap()));
    assert!(date1 > date2);
  }
}
