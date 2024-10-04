
pub mod structure;

use time::{OffsetDateTime, Date};

// TODO: Eq and Ord
#[derive(Debug, Clone)]
pub struct DateTime {
  inner: DateTimeRepr,
}

#[derive(Debug, Clone)]
enum DateTimeRepr {
  Date(Date),
  Datetime(OffsetDateTime),
}

impl DateTime {
  pub fn now_utc() -> Self {
    Self::from(OffsetDateTime::now_utc())
  }

  pub fn now_local() -> Self {
    let time = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    Self::from(time)
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
