
use time::UtcOffset;

use std::fmt::{self, Display};

/// Wrapper struct around [`UtcOffset`] with a custom `Display`
/// implementation suited to our datetime output format.
#[derive(Debug, Clone, Copy)]
pub struct TimezoneOffset(pub UtcOffset);

impl Display for TimezoneOffset {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let (hours, minutes, seconds) = self.0.as_hms();
    write!(f, "UTC {:+}", hours)?;
    if minutes != 0 || seconds != 0 {
      write!(f, ":{:02}", minutes.abs())?;
    }
    if seconds != 0 {
      write!(f, ":{:02}", seconds.abs())?;
    }
    Ok(())
  }
}

impl From<UtcOffset> for TimezoneOffset {
  fn from(offset: UtcOffset) -> Self {
    Self(offset)
  }
}

impl From<TimezoneOffset> for UtcOffset {
  fn from(offset: TimezoneOffset) -> Self {
    offset.0
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_utc() {
    let offset = TimezoneOffset(UtcOffset::UTC);
    assert_eq!(format!("{}", offset), "UTC +0");
  }

  #[test]
  fn test_hour_offset() {
    let offset = TimezoneOffset(UtcOffset::from_hms(3, 0, 0).unwrap());
    assert_eq!(format!("{}", offset), "UTC +3");
    let offset = TimezoneOffset(UtcOffset::from_hms(-12, 0, 0).unwrap());
    assert_eq!(format!("{}", offset), "UTC -12");
  }

  #[test]
  fn test_hour_minute_offset() {
    let offset = TimezoneOffset(UtcOffset::from_hms(3, 30, 0).unwrap());
    assert_eq!(format!("{}", offset), "UTC +3:30");
    let offset = TimezoneOffset(UtcOffset::from_hms(3, 1, 0).unwrap());
    assert_eq!(format!("{}", offset), "UTC +3:01");
    let offset = TimezoneOffset(UtcOffset::from_hms(-12, -15, 0).unwrap());
    assert_eq!(format!("{}", offset), "UTC -12:15");
  }

  #[test]
  fn test_hour_minute_second_offset() {
    let offset = TimezoneOffset(UtcOffset::from_hms(3, 30, 15).unwrap());
    assert_eq!(format!("{}", offset), "UTC +3:30:15");
    let offset = TimezoneOffset(UtcOffset::from_hms(3, 01, 02).unwrap());
    assert_eq!(format!("{}", offset), "UTC +3:01:02");
    let offset = TimezoneOffset(UtcOffset::from_hms(-12, -15, -1).unwrap());
    assert_eq!(format!("{}", offset), "UTC -12:15:01");
    let offset = TimezoneOffset(UtcOffset::from_hms(-12, 0, -1).unwrap());
    assert_eq!(format!("{}", offset), "UTC -12:00:01");
  }
}
