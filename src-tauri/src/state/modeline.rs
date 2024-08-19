
//! Helpers for building a modeline. A modeline is a line of status
//! text, usually at the bottom of the screen, which indicates various
//! configuration parameters about the current state of the program.

use crate::util::radix::Radix;

use std::fmt::Write;
use std::borrow::Cow;

/// A value that contributes text to a modeline.
pub trait ModelineValue {
  /// Contributes text to a modeline string.
  fn contribute(&self, buf: &mut String);
}

/// Builder structure for modeline text.
#[derive(Debug)]
pub struct ModelineBuilder {
  result_str: String,
}

impl ModelineBuilder {
  pub fn new() -> Self {
    ModelineBuilder { result_str: String::new() }
  }

  /// Appends a value to the builder. Returns `self` after
  /// modifications.
  pub fn append<V: ModelineValue>(mut self, value: V) -> Self {
    value.contribute(&mut self.result_str);
    self
  }

  pub fn build(self) -> String {
    self.result_str
  }
}

impl Default for ModelineBuilder {
  fn default() -> Self {
    ModelineBuilder::new()
  }
}

impl ModelineValue for String {
  fn contribute(&self, buf: &mut String) {
    buf.push_str(self);
  }
}

impl<'a> ModelineValue for &'a str {
  fn contribute(&self, buf: &mut String) {
    buf.push_str(self);
  }
}

impl<'a, B> ModelineValue for Cow<'a, B>
where B: 'a + ToOwned + ModelineValue + ?Sized {
  fn contribute(&self, buf: &mut String) {
    self.as_ref().contribute(buf);
  }
}

impl ModelineValue for Radix {
  fn contribute(&self, buf: &mut String) {
    match u8::from(*self) {
      2 => buf.push_str("Bin"),
      8 => buf.push_str("Oct"),
      10 => buf.push_str("Dec"),
      16 => buf.push_str("Hex"),
      n if n < 10 => write!(buf, "R={}", n).unwrap(),
      n => write!(buf, "R{}", n).unwrap(),
    }
  }
}

/// If the flag is true, then this function returns the given string.
/// Otherwise, returns a single dash, with spaces padding to the
/// length of the given string.
///
/// If the string is `""`, then `""` is returned, regardless of the
/// flag value.
pub fn boolean_flag(flag: bool, s: &str) -> Cow<'_, str> {
  if s.is_empty() {
    return Cow::Borrowed("");
  }
  if flag {
    Cow::Borrowed(s)
  } else {
    Cow::Owned(format!("-{: >width$}", "", width = s.len() - 1))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_modeline_builder() {
    let builder = ModelineBuilder::new()
      .append("ABC")
      .append("DEF");
    assert_eq!(builder.build(), "ABCDEF");
  }

  #[test]
  fn test_contribute_str() {
    let mut buf = String::new();
    "ABC".contribute(&mut buf);
    String::from("DEF").contribute(&mut buf);
    assert_eq!(buf, "ABCDEF");
  }

  #[test]
  fn test_contribute_radix() {
    let builder = ModelineBuilder::new()
      .append(Radix::new(10))
      .append(Radix::new(16))
      .append(Radix::new(2))
      .append(Radix::new(8))
      .append(Radix::new(9))
      .append(Radix::new(11));
    assert_eq!(
      builder.build(),
      "DecHexBinOctR=9R11",
    );
  }

  #[test]
  fn test_boolean_flag() {
    assert_eq!(boolean_flag(true, ""), "");
    assert_eq!(boolean_flag(false, ""), "");
    assert_eq!(boolean_flag(true, "A"), "A");
    assert_eq!(boolean_flag(false, "A"), "-");
    assert_eq!(boolean_flag(true, "Xyz"), "Xyz");
    assert_eq!(boolean_flag(false, "Xyz"), "-  ");
  }

  #[test]
  fn test_boolean_flag_ownership() {
    assert!(matches!(boolean_flag(true, ""), Cow::Borrowed(_)));
    assert!(matches!(boolean_flag(false, ""), Cow::Borrowed(_)));
    assert!(matches!(boolean_flag(true, "A"), Cow::Borrowed(_)));
    assert!(matches!(boolean_flag(false, "A"), Cow::Owned(_)));
    assert!(matches!(boolean_flag(true, "Xyz"), Cow::Borrowed(_)));
    assert!(matches!(boolean_flag(false, "Xyz"), Cow::Owned(_)));
  }
}
