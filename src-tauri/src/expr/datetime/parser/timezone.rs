
use crate::util::prism::Prism;

use regex::{Regex, RegexSet, Match, Captures};
use once_cell::sync::Lazy;

static TIMEZONE_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"(?i)\butc\s*(?:([+-]\d{1,2})(?::(\d{2})(?::(\d{2}))?)?)").unwrap());

static TIMEZONE_WHOLE_STRING_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"(?i)^\butc\s*(?:([+-]\d{1,2})(?::(\d{2})(?::(\d{2}))?)?)$").unwrap());

static TIMEZONE_NAME_RE: Lazy<RegexSet> = Lazy::new(|| {
  RegexSet::new(TIMEZONE_ABBREVIATIONS.keys().map(|abbr| format!("(?i)\\b{abbr}\\b"))).unwrap()
});

// Pulled from Wikipedia https://en.wikipedia.org/wiki/List_of_time_zone_abbreviations
//
// Ambiguous timezone abbreviations were resolved arbitrarily.
pub static TIMEZONE_ABBREVIATIONS: phf::OrderedMap<&'static str, Timezone> = phf::phf_ordered_map! {
  "ACDT" => Timezone(37_800),
  "ACST" => Timezone(34_200),
  "ACT" => Timezone(-18_000),
  "ACWST" => Timezone(31_500),
  "ADT" => Timezone(-10_800),
  "AEDT" => Timezone(39_600),
  "AEST" => Timezone(36_000),
  "AFT" => Timezone(16_200),
  "AKDT" => Timezone(-28_800),
  "AKST" => Timezone(-32_400),
  "ALMT" => Timezone(21_600),
  "AMST" => Timezone(-10_800),
  "AMT" => Timezone(-14_400),
  "ANAT" => Timezone(43_200),
  "AQTT" => Timezone(18_000),
  "ART" => Timezone(-10_800),
  "AST" => Timezone(-14_400),
  "AWST" => Timezone(28_800),
  "AZOST" => Timezone(0),
  "AZOT" => Timezone(-3_600),
  "AZT" => Timezone(14_400),
  "BNT" => Timezone(28_800),
  "BIOT" => Timezone(21_600),
  "BIT" => Timezone(-43_200),
  "BOT" => Timezone(-14_400),
  "BRST" => Timezone(-7_200),
  "BRT" => Timezone(-10_800),
  "BST" => Timezone(3_600),
  "BTT" => Timezone(21_600),
  "CAT" => Timezone(7_200),
  "CCT" => Timezone(23_400),
  "CDT" => Timezone(-18_000),
  "CEST" => Timezone(7_200),
  "CET" => Timezone(3_600),
  "CHADT" => Timezone(49_500),
  "CHAST" => Timezone(45_900),
  "CHOT" => Timezone(28_800),
  "CHOST" => Timezone(32_400),
  "CHST" => Timezone(36_000),
  "CHUT" => Timezone(36_000),
  "CIST" => Timezone(-28_800),
  "CKT" => Timezone(-36_000),
  "CLST" => Timezone(-10_800),
  "CLT" => Timezone(-14_400),
  "COST" => Timezone(-14_400),
  "COT" => Timezone(-18_000),
  "CST" => Timezone(-21_600),
  "CVT" => Timezone(-3_600),
  "CWST" => Timezone(31_500),
  "CXT" => Timezone(25_200),
  "DAVT" => Timezone(25_200),
  "DDUT" => Timezone(36_000),
  "DFT" => Timezone(3_600),
  "EASST" => Timezone(-18_000),
  "EAST" => Timezone(-21_600),
  "EAT" => Timezone(10_800),
  "ECT" => Timezone(-18_000),
  "EDT" => Timezone(-14_400),
  "EEST" => Timezone(10_800),
  "EET" => Timezone(7_200),
  "EGST" => Timezone(0),
  "EGT" => Timezone(-3_600),
  "EST" => Timezone(-18_000),
  "FET" => Timezone(10_800),
  "FJT" => Timezone(43_200),
  "FKST" => Timezone(-10_800),
  "FKT" => Timezone(-14_400),
  "FNT" => Timezone(-7_200),
  "GALT" => Timezone(-21_600),
  "GAMT" => Timezone(-32_400),
  "GET" => Timezone(14_400),
  "GFT" => Timezone(-10_800),
  "GILT" => Timezone(43_200),
  "GIT" => Timezone(-32_400),
  "GMT" => Timezone(0),
  "GST" => Timezone(14_400),
  "GYT" => Timezone(-14_400),
  "HDT" => Timezone(-32_400),
  "HAEC" => Timezone(7_200),
  "HST" => Timezone(-36_000),
  "HKT" => Timezone(28_800),
  "HMT" => Timezone(18_000),
  "HOVST" => Timezone(28_800),
  "HOVT" => Timezone(25_200),
  "ICT" => Timezone(25_200),
  "IDLW" => Timezone(-43_200),
  "IDT" => Timezone(10_800),
  "IOT" => Timezone(21_600),
  "IRDT" => Timezone(16_200),
  "IRKT" => Timezone(28_800),
  "IRST" => Timezone(12_600),
  "IST" => Timezone(19_800),
  "JST" => Timezone(32_400),
  "KALT" => Timezone(7_200),
  "KGT" => Timezone(21_600),
  "KOST" => Timezone(39_600),
  "KRAT" => Timezone(25_200),
  "KST" => Timezone(32_400),
  "LHST" => Timezone(37_800),
  "LINT" => Timezone(50_400),
  "MAGT" => Timezone(43_200),
  "MART" => Timezone(-30_600),
  "MAWT" => Timezone(18_000),
  "MDT" => Timezone(-21_600),
  "MET" => Timezone(3_600),
  "MEST" => Timezone(7_200),
  "MHT" => Timezone(43_200),
  "MIST" => Timezone(39_600),
  "MIT" => Timezone(-30_600),
  "MMT" => Timezone(23_400),
  "MSK" => Timezone(10_800),
  "MST" => Timezone(-25_200),
  "MUT" => Timezone(14_400),
  "MVT" => Timezone(18_000),
  "MYT" => Timezone(28_800),
  "NCT" => Timezone(39_600),
  "NDT" => Timezone(-5_400),
  "NFT" => Timezone(39_600),
  "NOVT" => Timezone(25_200),
  "NPT" => Timezone(20_700),
  "NST" => Timezone(-9_000),
  "NT" => Timezone(-9_000),
  "NUT" => Timezone(-39_600),
  "NZDT" => Timezone(46_800),
  "NZDST" => Timezone(46_800),
  "NZST" => Timezone(43_200),
  "OMST" => Timezone(21_600),
  "ORAT" => Timezone(18_000),
  "PDT" => Timezone(-25_200),
  "PET" => Timezone(-18_000),
  "PETT" => Timezone(43_200),
  "PGT" => Timezone(36_000),
  "PHOT" => Timezone(46_800),
  "PHT" => Timezone(28_800),
  "PHST" => Timezone(28_800),
  "PKT" => Timezone(18_000),
  "PMDT" => Timezone(-7_200),
  "PMST" => Timezone(-10_800),
  "PONT" => Timezone(39_600),
  "PST" => Timezone(-28_800),
  "PWT" => Timezone(32_400),
  "PYST" => Timezone(-10_800),
  "PYT" => Timezone(-14_400),
  "RET" => Timezone(14_400),
  "ROTT" => Timezone(-10_800),
  "SAKT" => Timezone(39_600),
  "SAMT" => Timezone(14_400),
  "SAST" => Timezone(7_200),
  "SBT" => Timezone(39_600),
  "SCT" => Timezone(14_400),
  "SDT" => Timezone(-36_000),
  "SGT" => Timezone(28_800),
  "SLST" => Timezone(19_800),
  "SRET" => Timezone(39_600),
  "SRT" => Timezone(-10_800),
  "SST" => Timezone(-39_600),
  "SYOT" => Timezone(10_800),
  "TAHT" => Timezone(-36_000),
  "THA" => Timezone(25_200),
  "TFT" => Timezone(18_000),
  "TJT" => Timezone(18_000),
  "TKT" => Timezone(46_800),
  "TLT" => Timezone(32_400),
  "TMT" => Timezone(18_000),
  "TRT" => Timezone(10_800),
  "TOT" => Timezone(46_800),
  "TST" => Timezone(28_800),
  "TVT" => Timezone(43_200),
  "ULAST" => Timezone(32_400),
  "ULAT" => Timezone(28_800),
  "UTC" => Timezone(0),
  "UYST" => Timezone(-7_200),
  "UYT" => Timezone(-10_800),
  "UZT" => Timezone(18_000),
  "VET" => Timezone(-14_400),
  "VLAT" => Timezone(36_000),
  "VOLT" => Timezone(10_800),
  "VOST" => Timezone(21_600),
  "VUT" => Timezone(39_600),
  "WAKT" => Timezone(43_200),
  "WAST" => Timezone(7_200),
  "WAT" => Timezone(3_600),
  "WEST" => Timezone(3_600),
  "WET" => Timezone(0),
  "WIB" => Timezone(25_200),
  "WIT" => Timezone(32_400),
  "WITA" => Timezone(28_800),
  "WGST" => Timezone(-7_200),
  "WGT" => Timezone(-10_800),
  "WST" => Timezone(28_800),
  "YAKT" => Timezone(32_400),
  "YEKT" => Timezone(18_000),
};

/// Newtype wrapper struct around a timezone designator, as a signed
/// UTC offset in seconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timezone(pub i32);

#[derive(Debug, Clone)]
pub struct ParsedTimezone {
  pub timezone: Timezone,
  pub parsed_str: String,
  _priv: (),
}

/// Start and end indices of timezone match. Similar to
/// [`regex::Match`], the indices provided here are guaranteed to be
/// valid UTF-8 codepoint boundaries in the source string.
#[derive(Debug, Clone)]
pub struct TimezoneMatch<'s> {
  match_str: &'s str,
  start: usize,
  end: usize,
}

/// Prism from [`String`] to [`ParsedTimezone`]
#[derive(Debug, Clone, Default)]
pub struct TimezonePrism;

impl Timezone {
  /// Parses a whole string as a timezone.
  ///
  /// Note that [`Timezone`] does *not* implement
  /// [`std::str::FromStr`]. In the future, this method may be
  /// modified to add a context argument (so the list of named
  /// timezones comes from a data file rather than being hardcoded
  /// into the source code), and I do not want to be tied to the API
  /// of `FromStr` in that event.
  pub fn parse(text: &str) -> Option<Self> {
    let text = text.trim();
    if let Some(cap) = TIMEZONE_WHOLE_STRING_RE.captures(text) {
      Some(process_captures(&cap))
    } else if let Some(tz) = TIMEZONE_ABBREVIATIONS.get(text.to_uppercase().as_str()) {
      Some(*tz)
    } else {
      None
    }
  }
}

impl<'s> TimezoneMatch<'s> {
  pub fn start(&self) -> usize {
    self.start
  }

  pub fn end(&self) -> usize {
    self.end
  }

  pub fn as_str(&self) -> &'s str {
    self.match_str
  }
}

impl<'s> From<Match<'s>> for TimezoneMatch<'s> {
  fn from(m: Match<'s>) -> Self {
    Self { start: m.start(), end: m.end(), match_str: m.as_str() }
  }
}

impl Prism<String, ParsedTimezone> for TimezonePrism {
  fn narrow_type(&self, s: String) -> Result<ParsedTimezone, String> {
    match Timezone::parse(&s) {
      None => Err(s),
      Some(tz) => Ok(ParsedTimezone { timezone: tz, parsed_str: s, _priv: () }),
    }
  }

  fn widen_type(&self, timezone: ParsedTimezone) -> String {
    timezone.parsed_str
  }
}

/// Attempts to interpret a substring of `text` as a timezone string.
pub fn search_for_timezone(text: &str) -> Option<(Timezone, TimezoneMatch)> {
  if let Some(cap) = TIMEZONE_RE.captures(text) {
    let tz = process_captures(&cap);
    Some((tz, cap.get(0).unwrap().into()))
  } else if let Some(match_idx) = TIMEZONE_NAME_RE.matches(text).into_iter().next() {
    let (tz_name, tz) = TIMEZONE_ABBREVIATIONS.index(match_idx).expect("Match index in RegexSet");
    let match_start = text.to_uppercase().find(tz_name).expect("Already matched in RegexSet");
    let match_end = match_start + tz_name.len();
    let match_str = &text[match_start..match_end];
    Some((*tz, TimezoneMatch { start: match_start, end: match_end, match_str }))
  } else {
    None
  }
}

/// Processes the captures from either [`TIMEZONE_RE`] or
/// [`TIMEZONE_WHOLE_STRING_RE`] into a [`Timezone`] object.
///
/// Precondition: `cap` is the result of matching against one of the
/// aforementioned regexes.
fn process_captures(cap: &Captures) -> Timezone {
  let hour: i32 = cap[1].parse().unwrap();
  let minute: i32 = cap.get(2).map(|m| m.as_str()).unwrap_or("0").parse().unwrap();
  let second: i32 = cap.get(3).map(|m| m.as_str()).unwrap_or("0").parse().unwrap();

  let minute = match_sign(minute, hour);
  let second = match_sign(second, hour);

  let total_seconds = hour * 3600 + minute * 60 + second;
  Timezone(total_seconds)
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
    let (tz, m) = search_for_timezone("xx UTC+3yy").unwrap();
    assert_eq!(tz.0, 10_800);
    assert_eq!(m.as_str(), "UTC+3");

    let (tz, m) = search_for_timezone("xx UTC-2:30yy").unwrap();
    assert_eq!(tz.0, -9_000);
    assert_eq!(m.as_str(), "UTC-2:30");

    let (tz, m) = search_for_timezone("xx UTC-2:30:10yy").unwrap();
    assert_eq!(tz.0, -9_010);
    assert_eq!(m.as_str(), "UTC-2:30:10");

    let (tz, m) = search_for_timezone("xx UTC-3 UTC+3").unwrap();
    assert_eq!(tz.0, -10_800);
    assert_eq!(m.as_str(), "UTC-3");

    assert!(search_for_timezone("xxUTC-w").is_none());
    assert!(search_for_timezone("xxUTC-3").is_none());
  }

  #[test]
  fn test_search_for_timezone_by_name() {
    let (tz, m) = search_for_timezone("xx BST yy").unwrap();
    assert_eq!(tz.0, 3_600);
    assert_eq!(m.as_str(), "BST");

    // First alphabetical match takes precedence
    let (tz, m) = search_for_timezone("wasT waKT").unwrap();
    assert_eq!(tz.0, 43_200);
    assert_eq!(m.as_str(), "waKT");

    assert!(search_for_timezone("cd cs cd cs").is_none());
    assert!(search_for_timezone("xxBSTyy").is_none());
  }

  #[test]
  fn test_search_for_timezone_utc_and_name_precedence() {
    let (tz, m) = search_for_timezone("xx BST UTC+9").unwrap();
    assert_eq!(tz.0, 32_400);
    assert_eq!(m.as_str(), "UTC+9");

    let (tz, m) = search_for_timezone("xx UTC+W").unwrap();
    assert_eq!(tz.0, 0);
    assert_eq!(m.as_str(), "UTC");
  }

  #[test]
  fn test_parse_timezone() {
    assert_eq!(Timezone::parse("UTC+3"), Some(Timezone(10_800)));
    assert_eq!(Timezone::parse("UTC +3"), Some(Timezone(10_800)));
    assert_eq!(Timezone::parse("   UTC +3"), Some(Timezone(10_800)));
    assert_eq!(Timezone::parse("   UTC-3 "), Some(Timezone(-10_800)));
    assert_eq!(Timezone::parse("   UTC- 3"), None);
    assert_eq!(Timezone::parse("   UTC+3 x"), None);
    assert_eq!(Timezone::parse("UTC+2:30"), Some(Timezone(9_000)));
    assert_eq!(Timezone::parse("UTC+2:30:01"), Some(Timezone(9_001)));
    assert_eq!(Timezone::parse("\t\tUTC-2:30:01"), Some(Timezone(-9_001)));
    assert_eq!(Timezone::parse("\t\tutc-2:30:01"), Some(Timezone(-9_001)));
    assert_eq!(Timezone::parse("bst"), Some(Timezone(3_600)));
    assert_eq!(Timezone::parse("bsT"), Some(Timezone(3_600)));
    assert_eq!(Timezone::parse("BST"), Some(Timezone(3_600)));
    assert_eq!(Timezone::parse("WAKT"), Some(Timezone(43_200)));
    assert_eq!(Timezone::parse("e"), None);
    assert_eq!(Timezone::parse(""), None);
    assert_eq!(Timezone::parse("ececdtt"), None);
    assert_eq!(Timezone::parse("ESTT"), None);
    assert_eq!(Timezone::parse("3"), None);
    assert_eq!(Timezone::parse("-1"), None);
    assert_eq!(Timezone::parse("-"), None);
  }
}
