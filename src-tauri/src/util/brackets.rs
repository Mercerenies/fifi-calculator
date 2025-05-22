
//! Helpers for producing bracketed output.

use super::write::SafeWrite;
use crate::util::unwrap_infallible_like;

use std::convert::Infallible;

/// A [`BracketConstruct`] which writes the given constant bracket
/// values.
#[derive(Debug, Clone)]
pub struct ConstBrackets<'a> {
  start_bracket: &'a str,
  end_bracket: &'a str,
}

/// HTML bracketing construct.
#[derive(Debug, Clone)]
pub struct HtmlBrackets {
  left_bracket_type: HtmlBracketsType,
  right_bracket_type: HtmlBracketsType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HtmlBracketsType {
  SquareBrackets,
  Parentheses,
  VerticalBars,
}

/// Bracketing construct which chooses from two constituent bracketing
/// constructs based on a Boolean flag.
pub struct ChoiceBrackets<B1, B2> {
  flag: bool,
  false_brackets: B1,
  true_brackets: B2,
}

/// A bracketing construct, which can be used to contain certain text
/// within brackets of some kind.
pub trait BracketConstruct<W: SafeWrite> {
  /// Writes the opening bracket to the given writer.
  fn write_open(&self, w: &mut W) -> Result<(), W::Error>;

  /// Writes the closing bracket to the given writer.
  fn write_close(&self, w: &mut W) -> Result<(), W::Error>;

  /// Writes the opening bracket, calls the callback, then writes the
  /// closing bracket.
  fn write_bracketed<T, F>(&self, w: &mut W, callback: F) -> Result<T, W::Error>
  where F: FnOnce(&mut W) -> Result<T, W::Error>,
        Self: Sized {
    self.write_bracketed_if(w, true, callback)
  }

  /// As [`BracketConstruct::write_bracketed`], but omits the brackets
  /// if the condition is false.
  fn write_bracketed_if<T, F>(&self, w: &mut W, needs_brackets: bool, callback: F) -> Result<T, W::Error>
  where F: FnOnce(&mut W) -> Result<T, W::Error>,
        Self: Sized {
    if needs_brackets {
      self.write_open(w)?;
    }
    let result = callback(w);
    if needs_brackets {
      self.write_close(w)?;
    }
    result
  }

  /// As [`BracketConstruct::write_bracketed_if`] but returns a `T`
  /// rather than a `Result`. Only works for infallible error types.
  fn write_bracketed_if_ok<T, F>(&self, w: &mut W, needs_brackets: bool, callback: F) -> T
  where F: FnOnce(&mut W) -> T,
        Self: Sized,
        W::Error: Into<Infallible> {
    unwrap_infallible_like(self.write_bracketed_if(w, needs_brackets, |w| Ok(callback(w))))
  }
}

impl<'a> ConstBrackets<'a> {
  pub const fn new(start_bracket: &'a str, end_bracket: &'a str) -> Self {
    Self { start_bracket, end_bracket }
  }
}

impl ConstBrackets<'static> {
  pub const fn square() -> Self {
    Self::new("[", "]")
  }

  pub const fn parens() -> Self {
    Self::new("(", ")")
  }

  pub const fn curly() -> Self {
    Self::new("{", "}")
  }
}

impl HtmlBrackets {
  pub const fn new(bracket_type: HtmlBracketsType) -> Self {
    Self {
      left_bracket_type: bracket_type,
      right_bracket_type: bracket_type,
    }
  }

  pub const fn non_matching(left_bracket_type: HtmlBracketsType, right_bracket_type: HtmlBracketsType) -> Self {
    Self { left_bracket_type, right_bracket_type }
  }

  pub fn css_classes(&self) -> String {
    if self.left_bracket_type == self.right_bracket_type {
      format!("{} {}", HtmlBracketsType::BASE_CSS_CLASS, self.left_bracket_type.two_sided_css_class())
    } else {
      format!("{} {} {}", HtmlBracketsType::BASE_CSS_CLASS, self.left_bracket_type.left_css_class(), self.right_bracket_type.right_css_class())
    }
  }
}

impl HtmlBracketsType {
  /// Base CSS class for any bracket type. All bracketing `<span>`
  /// constructs should include this CSS class.
  pub const BASE_CSS_CLASS: &'static str = "bracketed";

  /// CSS class for a two-sided bracket of this type. Every HTML
  /// bracketing construct should include either (1) one of these
  /// tags, or (2) a [left](Self::left_css_class) and a [right](Self::right_css_class) class.
  pub fn two_sided_css_class(self) -> &'static str {
    match self {
      HtmlBracketsType::SquareBrackets => "bracketed--square",
      HtmlBracketsType::Parentheses => "bracketed--parens",
      HtmlBracketsType::VerticalBars => "bracketed--vert",
    }
  }

  /// A left-side CSS class for a bracket of this type.
  pub fn left_css_class(self) -> &'static str {
    match self {
      HtmlBracketsType::SquareBrackets => "bracketed--square-left",
      HtmlBracketsType::Parentheses => "bracketed--parens-left",
      HtmlBracketsType::VerticalBars => "bracketed--vert-left",
    }
  }

  /// A right-side CSS class for a bracket of this type.
  pub fn right_css_class(self) -> &'static str {
    match self {
      HtmlBracketsType::SquareBrackets => "bracketed--square-right",
      HtmlBracketsType::Parentheses => "bracketed--parens-right",
      HtmlBracketsType::VerticalBars => "bracketed--vert-right",
    }
  }
}

impl<B1, B2> ChoiceBrackets<B1, B2> {
  pub const fn new(flag: bool, false_brackets: B1, true_brackets: B2) -> Self {
    Self { flag, false_brackets, true_brackets }
  }
}

impl<W: SafeWrite> BracketConstruct<W> for ConstBrackets<'_> {
  fn write_open(&self, w: &mut W) -> Result<(), W::Error> {
    w.write_str(self.start_bracket)
  }

  fn write_close(&self, w: &mut W) -> Result<(), W::Error> {
    w.write_str(self.end_bracket)
  }
}

impl<W: SafeWrite> BracketConstruct<W> for HtmlBrackets {
  fn write_open(&self, w: &mut W) -> Result<(), W::Error> {
    write!(w, r#"<span class="{}">"#, self.css_classes())
  }

  fn write_close(&self, w: &mut W) -> Result<(), W::Error> {
    w.write_str("</span>")
  }
}

impl<B1, B2, W> BracketConstruct<W> for ChoiceBrackets<B1, B2>
where
  B1: BracketConstruct<W>,
  B2: BracketConstruct<W>,
  W: SafeWrite {
  fn write_open(&self, w: &mut W) -> Result<(), W::Error> {
    if self.flag {
      self.true_brackets.write_open(w)
    } else {
      self.false_brackets.write_open(w)
    }
  }

  fn write_close(&self, w: &mut W) -> Result<(), W::Error> {
    if self.flag {
      self.true_brackets.write_close(w)
    } else {
      self.false_brackets.write_close(w)
    }
  }
}

/// If the `is_fancy` parameter is false, this bracketing construct
/// uses ordinary parentheses. If the `is_fancy` parameter is true, it
/// instead uses HTML resizing parentheses.
pub const fn fancy_parens(is_fancy: bool) -> ChoiceBrackets<ConstBrackets<'static>, HtmlBrackets> {
  ChoiceBrackets::new(is_fancy, ConstBrackets::parens(), HtmlBrackets::new(HtmlBracketsType::Parentheses))
}

/// If the `is_fancy` parameter is false, this bracketing construct
/// uses ordinary square brackets. If the `is_fancy` parameter is
/// true, it uses HTML square brackets which resize automatically.
pub const fn fancy_square_brackets(is_fancy: bool) -> ChoiceBrackets<ConstBrackets<'static>, HtmlBrackets> {
  ChoiceBrackets::new(is_fancy, ConstBrackets::square(), HtmlBrackets::new(HtmlBracketsType::SquareBrackets))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::util::unwrap_infallible;
  use crate::util::write::WriteAsSafeWrite;

  use std::fmt;

  #[test]
  fn test_write_bracketed() {
    let mut s = String::new();
    unwrap_infallible(write!(s, "a"));
    let result = unwrap_infallible(ConstBrackets::parens().write_bracketed(&mut s, |s| {
      write!(s, "b")?;
      Ok("result str")
    }));
    unwrap_infallible(write!(s, "c"));
    assert_eq!(result, "result str");
    assert_eq!(s, "a(b)c");
  }

  #[test]
  fn test_write_bracketed_with_failure() {
    let mut s = WriteAsSafeWrite(String::new());
    write!(s, "a").unwrap();
    ConstBrackets::parens().write_bracketed::<(), _>(&mut s, |s| {
      write!(s, "b")?;
      Err(fmt::Error)
    }).unwrap_err();
    assert_eq!(s.0, "a(b)"); // Note: Closing paren is still present, even though we failed
  }

  #[test]
  fn test_write_bracketed_if() {
    let mut s = String::new();
    unwrap_infallible(write!(s, "a"));
    let result1 = ConstBrackets::parens().write_bracketed_if_ok(&mut s, true, |s| {
      unwrap_infallible(write!(s, "b"));
      "result str 1"
    });
    let result2 = ConstBrackets::parens().write_bracketed_if_ok(&mut s, false, |s| {
      unwrap_infallible(write!(s, "c"));
      "result str 2"
    });
    unwrap_infallible(write!(s, "d"));
    assert_eq!(result1, "result str 1");
    assert_eq!(result2, "result str 2");
    assert_eq!(s, "a(b)cd");
  }

  #[test]
  fn test_write_bracketed_with_html() {
    let mut s = String::new();
    unwrap_infallible(write!(s, "a"));
    let result = unwrap_infallible(HtmlBrackets::new(HtmlBracketsType::Parentheses).write_bracketed(&mut s, |s| {
      write!(s, "b")?;
      Ok("result str")
    }));
    unwrap_infallible(write!(s, "c"));
    assert_eq!(result, "result str");
    assert_eq!(s, r#"a<span class="bracketed bracketed--parens">b</span>c"#);
  }

  #[test]
  fn test_write_bracketed_with_non_matching_html() {
    let brackets = HtmlBrackets::non_matching(HtmlBracketsType::Parentheses, HtmlBracketsType::SquareBrackets);
    let mut s = String::new();
    unwrap_infallible(write!(s, "a"));
    let result = unwrap_infallible(brackets.write_bracketed(&mut s, |s| {
      write!(s, "b")?;
      Ok("result str")
    }));
    unwrap_infallible(write!(s, "c"));
    assert_eq!(result, "result str");
    assert_eq!(s, r#"a<span class="bracketed bracketed--parens-left bracketed--square-right">b</span>c"#);
  }
}
