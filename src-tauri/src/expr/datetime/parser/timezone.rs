
use regex::{Regex, Match};
use once_cell::sync::Lazy;

static TIMEZONE_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"(?i)utc\s*(?:([+-]\d{1,2})(?::(\d{2})(?::(\d{2}))?)?)").unwrap());

/// Newtype wrapper struct around a timezone designator, as a signed
/// UTC offset in seconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Timezone(pub i32);

/// Attempts to interpret a substring of `text` as a timezone string.
pub(super) fn search_for_timezone(text: &str) -> Option<(Timezone, Match)> {
  if let Some(cap) = TIMEZONE_RE.captures(text) {
    let hour: i32 = cap[1].parse().unwrap();
    let minute: i32 = cap.get(2).map(|m| m.as_str()).unwrap_or("0").parse().unwrap();
    let second: i32 = cap.get(3).map(|m| m.as_str()).unwrap_or("0").parse().unwrap();

    let minute = match_sign(minute, hour);
    let second = match_sign(second, hour);

    let total_seconds = hour * 3600 + minute * 60 + second;
    Some((Timezone(total_seconds), cap.get(0).unwrap()))
  } else {
    None
  }
}

fn match_sign(n: i32, exemplar: i32) -> i32 {
  assert!(n >= 0);
  if exemplar < 0 {
    - n
  } else {
    n
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_search_for_timezone() {
    let (tz, m) = search_for_timezone("xxUTC+3yy").unwrap();
    assert_eq!(tz.0, 10800);
    assert_eq!(m.as_str(), "UTC+3");

    let (tz, m) = search_for_timezone("xxUTC-2:30yy").unwrap();
    assert_eq!(tz.0, -9000);
    assert_eq!(m.as_str(), "UTC-2:30");

    let (tz, m) = search_for_timezone("xxUTC-2:30:10yy").unwrap();
    assert_eq!(tz.0, -9010);
    assert_eq!(m.as_str(), "UTC-2:30:10");
  }
}
