
use crate::util::stricteq::StrictEq;
use super::DateTime;

use time::Duration;
use thiserror::Error;

use std::ops;
use std::hash::{Hash, Hasher};

/// This structure wraps a [`time::Duration`] but also stores a
/// precision, i.e. whether or not the duration was specified to
/// precision below a day or not.
///
/// When a precise duration is added to or subtracted from a date that
/// lacks a time field, it will unconditionally add a time field, even
/// if the duration happens to equal a precise number of days.
///
/// When combining two durations with precision, the maximum precision
/// of the two is taken to be the duration of the result.
///
/// For the purposes of `Eq` and `Ord`, only the duration is relevant,
/// not the precision. That is, two [`PreciseDuration`] objects are
/// equal if their durations are equal, regardless of precision.
#[derive(Debug, Clone, Copy)]
pub struct PrecisionDuration {
  duration: Duration,
  precise: bool,
}

#[derive(Debug, Clone, Error)]
#[error("not enough precision to represent duration")]
pub struct NotEnoughPrecision;

pub fn is_whole_day_count(duration: Duration) -> bool {
  let whole_days = duration.whole_days();
  Duration::days(whole_days) == duration
}

impl PrecisionDuration {
  /// Constructs a precise duration object.
  pub fn precise(duration: Duration) -> Self {
    Self { duration, precise: true }
  }

  /// Attempts to construct an imprecise duration object. If the
  /// argument is not a whole number of days,
  pub fn try_imprecise(duration: Duration) -> Result<Self, NotEnoughPrecision> {
    if is_whole_day_count(duration) {
      Ok(Self { duration, precise: false })
    } else {
      Err(NotEnoughPrecision)
    }
  }

  /// Constructs an imprecise duration object. Panics if
  /// `is_whole_day_count` is false for the argument.
  pub fn imprecise(duration: Duration) -> Self {
    Self::try_imprecise(duration)
      .expect("duration is not a whole number of days")
  }

  pub fn days(days: i64) -> Self {
    // Never panics: whole number of days.
    Self::imprecise(Duration::days(days))
  }

  pub fn microseconds(us: i64) -> Self {
    Self::precise(Duration::microseconds(us))
  }

  pub fn is_precise(&self) -> bool {
    self.precise
  }

  pub fn duration(&self) -> Duration {
    self.duration
  }

  pub fn checked_add(self, other: Self) -> Option<Self> {
    let duration = self.duration.checked_add(other.duration)?;
    Some(Self { duration, precise: self.precise || other.precise })
  }

  pub fn checked_sub(self, other: Self) -> Option<Self> {
    self.checked_add(- other)
  }

  pub fn checked_mul(self, other: i32) -> Option<Self> {
    let duration = self.duration.checked_mul(other)?;
    Some(Self { duration, precise: self.precise })
  }
}

impl PartialEq for PrecisionDuration {
  fn eq(&self, other: &Self) -> bool {
    self.duration == other.duration
  }
}

impl Eq for PrecisionDuration {}

impl StrictEq for PrecisionDuration {
  fn strict_eq(&self, other: &Self) -> bool {
    self.duration == other.duration &&
      self.precise == other.precise
  }
}

impl Hash for PrecisionDuration {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.duration.hash(state)
  }
}

impl PartialOrd for PrecisionDuration {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for PrecisionDuration {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.duration.cmp(&other.duration)
  }
}

impl ops::Add for PrecisionDuration {
  type Output = Self;

  fn add(self, other: Self) -> Self {
    Self {
      duration: self.duration + other.duration,
      precise: self.precise || other.precise,
    }
  }
}

impl ops::Sub for PrecisionDuration {
  type Output = Self;

  fn sub(self, other: Self) -> Self {
    self + (- other)
  }
}

impl ops::Neg for PrecisionDuration {
  type Output = Self;

  fn neg(self) -> Self {
    -1i32 * self
  }
}

impl ops::Mul<i32> for PrecisionDuration {
  type Output = Self;

  fn mul(self, other: i32) -> Self {
    Self {
      duration: self.duration * other,
      precise: self.precise
    }
  }
}

impl ops::Mul<PrecisionDuration> for i32 {
  type Output = PrecisionDuration;

  fn mul(self, other: PrecisionDuration) -> PrecisionDuration {
    other * self
  }
}

impl ops::Add<PrecisionDuration> for DateTime {
  type Output = DateTime;

  fn add(self, other: PrecisionDuration) -> DateTime {
    if self.has_time() || other.is_precise() {
      DateTime::from(self.to_offset_date_time() + other.duration())
    } else {
      DateTime::from(self.without_time() + other.duration())
    }
  }
}

impl ops::Add<DateTime> for PrecisionDuration {
  type Output = DateTime;

  fn add(self, other: DateTime) -> DateTime {
    other + self
  }
}

impl ops::Sub<PrecisionDuration> for DateTime {
  type Output = DateTime;

  fn sub(self, other: PrecisionDuration) -> DateTime {
    self + (- other)
  }
}

impl ops::Sub for DateTime {
  type Output = PrecisionDuration;

  fn sub(self, other: DateTime) -> PrecisionDuration {
    if self.has_time() || other.has_time() {
      PrecisionDuration::precise(self.to_offset_date_time() - other.to_offset_date_time())
    } else {
      PrecisionDuration::imprecise(self.without_time() - other.without_time())
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{assert_strict_eq, assert_strict_ne};

  use time::{OffsetDateTime, Date, Month};

  use std::collections::hash_map::DefaultHasher;

  #[test]
  fn test_precision_duration_construction() {
    let duration = PrecisionDuration::precise(Duration::days(3));
    assert!(duration.is_precise());
    let duration = PrecisionDuration::imprecise(Duration::days(3));
    assert!(!duration.is_precise());
    let duration = PrecisionDuration::precise(Duration::seconds(4));
    assert!(duration.is_precise());
    let err = PrecisionDuration::try_imprecise(Duration::seconds(4));
    err.unwrap_err();
  }

  #[test]
  fn test_precision_duration_destruction() {
    let duration = PrecisionDuration::precise(Duration::days(3));
    assert_eq!(duration.duration(), Duration::days(3));
  }

  #[test]
  fn test_is_whole_day_count() {
    assert!(is_whole_day_count(Duration::days(1)));
    assert!(is_whole_day_count(Duration::days(10)));
    assert!(is_whole_day_count(Duration::weeks(4)));
    assert!(is_whole_day_count(Duration::hours(24)));
    assert!(!is_whole_day_count(Duration::hours(23)));
    assert!(!is_whole_day_count(Duration::seconds(900)));
    assert!(!is_whole_day_count(Duration::microseconds(900)));
  }

  #[test]
  fn test_precision_duration_eq() {
    assert_eq!(PrecisionDuration::precise(Duration::days(2)), PrecisionDuration::precise(Duration::days(2)));
    assert_eq!(PrecisionDuration::precise(Duration::days(2)), PrecisionDuration::imprecise(Duration::days(2)));
    assert_ne!(PrecisionDuration::precise(Duration::days(2)), PrecisionDuration::imprecise(Duration::days(3)));
  }

  #[test]
  fn test_precision_duration_hash() {
    fn hash(value: &PrecisionDuration) -> u64 {
      let mut hasher = DefaultHasher::new();
      value.hash(&mut hasher);
      hasher.finish()
    }

    assert_eq!(
      hash(&PrecisionDuration::precise(Duration::days(2))),
      hash(&PrecisionDuration::imprecise(Duration::days(2))),
    );
  }

  #[test]
  fn test_precision_duration_strict_eq() {
    assert_strict_eq!(PrecisionDuration::precise(Duration::days(2)), PrecisionDuration::precise(Duration::days(2)));
    assert_strict_ne!(PrecisionDuration::precise(Duration::days(2)), PrecisionDuration::imprecise(Duration::days(2)));
    assert_strict_ne!(PrecisionDuration::precise(Duration::days(2)), PrecisionDuration::imprecise(Duration::days(3)));
  }

  #[test]
  fn test_add_duration() {
    assert_strict_eq!(
      PrecisionDuration::precise(Duration::days(3)) + PrecisionDuration::precise(Duration::days(2)),
      PrecisionDuration::precise(Duration::days(5)),
    );
    assert_strict_eq!(
      PrecisionDuration::imprecise(Duration::days(3)) + PrecisionDuration::precise(Duration::days(2)),
      PrecisionDuration::precise(Duration::days(5)),
    );
    assert_strict_eq!(
      PrecisionDuration::imprecise(Duration::days(3)) + PrecisionDuration::imprecise(Duration::days(2)),
      PrecisionDuration::imprecise(Duration::days(5)),
    );
    assert_strict_eq!(
      PrecisionDuration::imprecise(Duration::days(1)) + PrecisionDuration::precise(Duration::seconds(100)),
      PrecisionDuration::precise(Duration::seconds(86_500)),
    );
  }

  #[test]
  fn test_checked_add_duration() {
    assert_strict_eq!(
      PrecisionDuration::precise(Duration::days(3)).checked_add(PrecisionDuration::precise(Duration::days(2))),
      Some(PrecisionDuration::precise(Duration::days(5))),
    );
    assert_strict_eq!(
      PrecisionDuration::imprecise(Duration::days(3)).checked_add(PrecisionDuration::precise(Duration::days(2))),
      Some(PrecisionDuration::precise(Duration::days(5))),
    );
    assert_strict_eq!(
      PrecisionDuration::imprecise(Duration::days(3)).checked_add(PrecisionDuration::imprecise(Duration::days(2))),
      Some(PrecisionDuration::imprecise(Duration::days(5))),
    );
    assert_strict_eq!(
      PrecisionDuration::imprecise(Duration::days(1)).checked_add(PrecisionDuration::precise(Duration::seconds(100))),
      Some(PrecisionDuration::precise(Duration::seconds(86_500))),
    );
    assert_strict_eq!(
      PrecisionDuration::precise(Duration::MAX).checked_add(PrecisionDuration::precise(Duration::seconds(100))),
      None,
    );
  }

  #[test]
  fn test_sub_duration() {
    assert_strict_eq!(
      PrecisionDuration::precise(Duration::days(3)) - PrecisionDuration::precise(Duration::days(2)),
      PrecisionDuration::precise(Duration::days(1)),
    );
    assert_strict_eq!(
      PrecisionDuration::precise(Duration::days(2)) - PrecisionDuration::precise(Duration::days(3)),
      PrecisionDuration::precise(Duration::days(-1)),
    );
    assert_strict_eq!(
      PrecisionDuration::imprecise(Duration::days(2)) - PrecisionDuration::imprecise(Duration::days(3)),
      PrecisionDuration::imprecise(Duration::days(-1)),
    );
    assert_strict_eq!(
      PrecisionDuration::imprecise(Duration::days(3)) - PrecisionDuration::precise(Duration::days(2)),
      PrecisionDuration::precise(Duration::days(1)),
    );
    assert_strict_eq!(
      PrecisionDuration::imprecise(Duration::days(1)) - PrecisionDuration::precise(Duration::seconds(100)),
      PrecisionDuration::precise(Duration::seconds(86_300)),
    );
  }

  #[test]
  fn test_checked_sub_duration() {
    assert_strict_eq!(
      PrecisionDuration::precise(Duration::days(3)).checked_sub(PrecisionDuration::precise(Duration::days(2))),
      Some(PrecisionDuration::precise(Duration::days(1))),
    );
    assert_strict_eq!(
      PrecisionDuration::imprecise(Duration::days(3)).checked_sub(PrecisionDuration::precise(Duration::days(2))),
      Some(PrecisionDuration::precise(Duration::days(1))),
    );
    assert_strict_eq!(
      PrecisionDuration::imprecise(Duration::days(3)).checked_sub(PrecisionDuration::imprecise(Duration::days(2))),
      Some(PrecisionDuration::imprecise(Duration::days(1))),
    );
    assert_strict_eq!(
      PrecisionDuration::imprecise(Duration::days(1)).checked_sub(PrecisionDuration::precise(Duration::seconds(100))),
      Some(PrecisionDuration::precise(Duration::seconds(86_300))),
    );
    assert_strict_eq!(
      PrecisionDuration::precise(Duration::MIN).checked_sub(PrecisionDuration::precise(Duration::seconds(100))),
      None,
    );
  }

  #[test]
  fn test_neg_duration() {
    assert_strict_eq!(
      - PrecisionDuration::precise(Duration::days(2)),
      PrecisionDuration::precise(Duration::days(-2)),
    );
    assert_strict_eq!(
      - PrecisionDuration::imprecise(Duration::days(2)),
      PrecisionDuration::imprecise(Duration::days(-2)),
    );
  }

  #[test]
  fn test_mul_duration() {
    assert_strict_eq!(
      PrecisionDuration::precise(Duration::days(3)) * 2,
      PrecisionDuration::precise(Duration::days(6)),
    );
    assert_strict_eq!(
      2 * PrecisionDuration::precise(Duration::days(3)),
      PrecisionDuration::precise(Duration::days(6)),
    );
    assert_strict_eq!(
      PrecisionDuration::imprecise(Duration::days(3)) * 2,
      PrecisionDuration::imprecise(Duration::days(6)),
    );
    assert_strict_eq!(
      2 * PrecisionDuration::imprecise(Duration::days(3)),
      PrecisionDuration::imprecise(Duration::days(6)),
    );
    assert_strict_eq!(
      (-2) * PrecisionDuration::imprecise(Duration::days(3)),
      PrecisionDuration::imprecise(Duration::days(-6)),
    );
  }

  #[test]
  fn test_checked_mul_duration() {
    assert_strict_eq!(
      PrecisionDuration::precise(Duration::days(3)).checked_mul(2),
      Some(PrecisionDuration::precise(Duration::days(6))),
    );
    assert_strict_eq!(
      PrecisionDuration::imprecise(Duration::days(3)).checked_mul(2),
      Some(PrecisionDuration::imprecise(Duration::days(6))),
    );
    assert_strict_eq!(
      PrecisionDuration::precise(Duration::MAX).checked_mul(2),
      None,
    );
  }

  #[test]
  fn test_add_duration_and_datetime() {
    const UNIX_EPOCH_DATE: Date = OffsetDateTime::UNIX_EPOCH.date();

    let datetime = DateTime::from(OffsetDateTime::UNIX_EPOCH);
    let duration = PrecisionDuration::precise(Duration::days(2));
    let expected = DateTime::from(OffsetDateTime::UNIX_EPOCH + Duration::days(2));
    assert_strict_eq!(datetime + duration, expected);

    let datetime = DateTime::from(UNIX_EPOCH_DATE);
    let duration = PrecisionDuration::precise(Duration::days(2));
    let expected = DateTime::from(OffsetDateTime::UNIX_EPOCH + Duration::days(2));
    assert_strict_eq!(datetime + duration, expected);

    let datetime = DateTime::from(OffsetDateTime::UNIX_EPOCH);
    let duration = PrecisionDuration::imprecise(Duration::days(2));
    let expected = DateTime::from(OffsetDateTime::UNIX_EPOCH + Duration::days(2));
    assert_strict_eq!(datetime + duration, expected);

    let datetime = DateTime::from(UNIX_EPOCH_DATE);
    let duration = PrecisionDuration::imprecise(Duration::days(2));
    let expected = DateTime::from(UNIX_EPOCH_DATE + Duration::days(2));
    assert_strict_eq!(datetime + duration, expected);

    let datetime = DateTime::from(UNIX_EPOCH_DATE);
    let duration = PrecisionDuration::precise(Duration::seconds(1));
    let expected = DateTime::from(OffsetDateTime::UNIX_EPOCH + Duration::seconds(1));
    assert_strict_eq!(datetime + duration, expected);

    let datetime = DateTime::from(OffsetDateTime::UNIX_EPOCH);
    let duration = PrecisionDuration::precise(Duration::days(2));
    let expected = DateTime::from(OffsetDateTime::UNIX_EPOCH + Duration::days(2));
    assert_strict_eq!(duration + datetime, expected);

    let datetime = DateTime::from(UNIX_EPOCH_DATE);
    let duration = PrecisionDuration::precise(Duration::days(2));
    let expected = DateTime::from(OffsetDateTime::UNIX_EPOCH + Duration::days(2));
    assert_strict_eq!(duration + datetime, expected);

    let datetime = DateTime::from(OffsetDateTime::UNIX_EPOCH);
    let duration = PrecisionDuration::imprecise(Duration::days(2));
    let expected = DateTime::from(OffsetDateTime::UNIX_EPOCH + Duration::days(2));
    assert_strict_eq!(duration + datetime, expected);

    let datetime = DateTime::from(UNIX_EPOCH_DATE);
    let duration = PrecisionDuration::imprecise(Duration::days(2));
    let expected = DateTime::from(UNIX_EPOCH_DATE + Duration::days(2));
    assert_strict_eq!(duration + datetime, expected);

    let datetime = DateTime::from(UNIX_EPOCH_DATE);
    let duration = PrecisionDuration::precise(Duration::seconds(1));
    let expected = DateTime::from(OffsetDateTime::UNIX_EPOCH + Duration::seconds(1));
    assert_strict_eq!(duration + datetime, expected);

    let datetime = DateTime::from(UNIX_EPOCH_DATE);
    let duration = PrecisionDuration::precise(Duration::seconds(1));
    let expected = DateTime::from(OffsetDateTime::UNIX_EPOCH - Duration::seconds(1));
    assert_strict_eq!(datetime - duration, expected);
  }


  #[test]
  fn test_checked_add_duration_and_datetime() {
    let datetime = DateTime::from(OffsetDateTime::UNIX_EPOCH);
    let duration = PrecisionDuration::precise(Duration::days(2));
    let expected = DateTime::from(OffsetDateTime::UNIX_EPOCH + Duration::days(2));
    assert_strict_eq!(datetime.checked_add(duration), Some(expected));

    let datetime = DateTime::from(OffsetDateTime::UNIX_EPOCH);
    let duration = PrecisionDuration::precise(Duration::days(2));
    let expected = DateTime::from(OffsetDateTime::UNIX_EPOCH - Duration::days(2));
    assert_strict_eq!(datetime.checked_sub(duration), Some(expected));

    let datetime = DateTime::from(Date::from_calendar_date(-9999, Month::January, 1).unwrap());
    let duration = PrecisionDuration::precise(Duration::days(2));
    assert_strict_eq!(datetime.checked_sub(duration), None);

    let datetime = DateTime::from(Date::from_calendar_date(9999, Month::December, 31).unwrap());
    let duration = PrecisionDuration::precise(Duration::days(2));
    assert_strict_eq!(datetime.checked_add(duration), None);
  }
}
