
use crate::util::regexes::WHITESPACE_RE;

#[derive(Debug, Clone)]
struct Token<'a> {
  datum: &'a str,
}

#[derive(Debug, Clone)]
struct TimeOfDay {
  hour: u8,
  minute: u8,
  second: u8,
  microsecond: u8,
}

#[derive(Debug, Clone, Copy)]
enum PeriodOfDay {
  Am,
  Pm,
  Noon,
  Midnight,
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
}

fn tokenize_datetime_str(input: &str) -> Vec<Token<'_>> {
  WHITESPACE_RE.split(input).map(|datum| Token { datum }).collect()
}
