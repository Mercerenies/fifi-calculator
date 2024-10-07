
//! Module containing generally useful regular expressions.

use regex::Regex;
use once_cell::sync::Lazy;

pub static WHITESPACE_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\s+").unwrap());
