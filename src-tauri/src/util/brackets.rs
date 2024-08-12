
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
  bracket_type: HtmlBracketsType,
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
    Self { bracket_type }
  }
}

impl HtmlBracketsType {
  pub fn css_classes(self) -> &'static str {
    match self {
      HtmlBracketsType::SquareBrackets => "bracketed bracketed--square",
      HtmlBracketsType::Parentheses => "bracketed bracketed--parens",
      HtmlBracketsType::VerticalBars => "bracketed bracketed--vert",
    }
  }
}

impl<B1, B2> ChoiceBrackets<B1, B2> {
  pub const fn new(flag: bool, false_brackets: B1, true_brackets: B2) -> Self {
    Self { flag, false_brackets, true_brackets }
  }
}

impl<'a, W: SafeWrite> BracketConstruct<W> for ConstBrackets<'a> {
  fn write_open(&self, w: &mut W) -> Result<(), W::Error> {
    w.write_str(&self.start_bracket)
  }

  fn write_close(&self, w: &mut W) -> Result<(), W::Error> {
    w.write_str(&self.end_bracket)
  }
}

impl<'a, W: SafeWrite> BracketConstruct<W> for HtmlBrackets {
  fn write_open(&self, w: &mut W) -> Result<(), W::Error> {
    write!(w, r#"<span class="{}">"#, self.bracket_type.css_classes())
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
}
