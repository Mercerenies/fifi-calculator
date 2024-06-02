
use super::source::{SourceOffset, Span};
use crate::util::clamp;

use regex::{Regex, Captures};
use once_cell::sync::Lazy;

#[derive(Debug, Clone)]
pub struct TokenizerState<'a> {
  whole_input: &'a str,
  input: &'a str,
  position: SourceOffset,
}

#[derive(Debug, Clone)]
pub struct TokenizerMatch<'a> {
  matched_str: &'a str,
  start: SourceOffset,
  end: SourceOffset,
}

#[derive(Debug)]
pub struct TokenizerCaptures<'a> {
  captures: Captures<'a>,
  start: SourceOffset,
  end: SourceOffset,
}

impl<'a> TokenizerState<'a> {
  pub fn new(input: &'a str) -> Self {
    Self {
      whole_input: input,
      input,
      position: SourceOffset(0)
    }
  }

  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }

  pub fn len(&self) -> usize {
    self.input.len() + self.position.0
  }

  pub fn remaining_len(&self) -> usize {
    self.input.len()
  }

  pub fn is_eof(&self) -> bool {
    self.input.is_empty()
  }

  pub fn peek(&self) -> Option<char> {
    self.input.chars().next()
  }

  /// Seeks to an absolute position in the string. Out of bounds
  /// indices are truncated.
  pub fn seek(&mut self, mut pos: SourceOffset) {
    pos = clamp(pos, SourceOffset(0), SourceOffset(self.len()));
    self.position = pos;
    self.input = &self.whole_input[pos.0..];
  }

  /// Advances the position of `self` by `amount`. Returns a
  /// [`TokenizerMatch`] indicating the substring matched by the
  /// skipped portion. This method will never advance beyond
  /// one-past-the-end of the input. If `amount` is too large, the
  /// method will advance to the end of the string and then stop.
  pub fn advance(&mut self, mut amount: usize) -> TokenizerMatch<'_> {
    amount = amount.min(self.input.len());

    let match_pos = self.current_pos();
    let (prefix, suffix) = self.input.split_at(amount);
    self.position.0 += amount;
    self.input = suffix;
    TokenizerMatch {
      matched_str: prefix,
      start: match_pos,
      end: match_pos + amount,
    }
  }

  pub fn current_pos(&self) -> SourceOffset {
    self.position
  }

  pub fn read_literal(&mut self, literal: &str) -> Option<TokenizerMatch<'_>> {
    self.input.starts_with(literal).then(|| {
      self.advance(literal.len())
    })
  }

  /// If the current position of the string matches the given regex,
  /// returns the matched string and advances the tokenizer state. If
  /// not, returns `None`.
  ///
  /// The regex MUST be anchored at the start of the input. This
  /// function may panic if that precondition is not satisfied.
  pub fn read_regex(&mut self, regex: &Regex) -> Option<TokenizerMatch<'_>> {
    let m = regex.find(self.input)?;
    assert_eq!(m.start(), 0, "Regex must be anchored at the start of the input");

    Some(self.advance(m.len()))
  }

  pub fn read_regex_with_captures(&mut self, regex: &Regex) -> Option<TokenizerCaptures<'_>> {
    let c = regex.captures(self.input)?;
    let m = self.advance(c.get(0).unwrap().len()); // unwrap: First capture group always exists
    Some(TokenizerCaptures {
      captures: c,
      start: m.start(),
      end: m.end(),
    })
  }

  pub fn read_many<T, F>(&mut self, mut function: F) -> Vec<T>
  where F: FnMut(&mut Self) -> Option<T> {
    let mut output = Vec::new();
    while let Some(item) = function(self) {
      output.push(item);
    }
    output
  }

  pub fn read_some<T, F>(&mut self, function: F) -> Option<Vec<T>>
  where F: FnMut(&mut Self) -> Option<T> {
    let output = self.read_many(function);
    (!output.is_empty()).then_some(output)
  }

  pub fn consume_spaces(&mut self) {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*").unwrap());
    self.read_regex(&RE).expect("regex should not fail");
  }
}

impl<'h> TokenizerMatch<'h> {
  pub fn as_str(&self) -> &'h str {
    self.matched_str
  }
  pub fn start(&self) -> SourceOffset {
    self.start
  }
  pub fn end(&self) -> SourceOffset {
    self.end
  }
  pub fn span(&self) -> Span {
    Span::new(self.start, self.end)
  }
  pub fn is_empty(&self) -> bool {
    self.start == self.end
  }
}

impl<'h> TokenizerCaptures<'h> {
  pub fn as_str(&self) -> &'h str {
    // unwrap safety: When capture group == 0, Captures::get is
    // guaranteed to return a non-empty value.
    self.captures.get(0).unwrap().as_str()
  }
  pub fn get(&self, i: usize) -> Option<&'h str> {
    self.captures.get(i).map(|m| m.as_str())
  }
  #[allow(clippy::len_without_is_empty)] // Captures object is always non-empty
  pub fn len(&self) -> usize {
    self.captures.len()
  }
  pub fn start(&self) -> SourceOffset {
    self.start
  }
  pub fn end(&self) -> SourceOffset {
    self.end
  }
  pub fn span(&self) -> Span {
    Span::new(self.start, self.end)
  }
}

impl Default for TokenizerState<'static> {
  fn default() -> Self {
    Self {
      whole_input: "",
      input: "",
      position: SourceOffset(0)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_len() {
    let state = TokenizerState::new("");
    assert_eq!(state.len(), 0);
    let state = TokenizerState::new("abcd");
    assert_eq!(state.len(), 4);
    let state = TokenizerState::default();
    assert_eq!(state.len(), 0);

    let mut state = TokenizerState::new("abcdefg");
    assert_eq!(state.len(), 7);
    state.advance(3);
    assert_eq!(state.len(), 7);
    state.advance(2);
    assert_eq!(state.len(), 7);
    state.advance(99);
    assert_eq!(state.len(), 7);
  }

  #[test]
  fn test_seek() {
    let mut state = TokenizerState::new("abcd");
    assert_eq!(state.current_pos(), SourceOffset(0));
    assert_eq!(state.len(), 4);
    assert_eq!(state.remaining_len(), 4);
    assert_eq!(state.peek(), Some('a'));

    state.seek(SourceOffset(3));
    assert_eq!(state.current_pos(), SourceOffset(3));
    assert_eq!(state.len(), 4);
    assert_eq!(state.remaining_len(), 1);
    assert_eq!(state.peek(), Some('d'));

    state.seek(SourceOffset(1));
    assert_eq!(state.current_pos(), SourceOffset(1));
    assert_eq!(state.len(), 4);
    assert_eq!(state.remaining_len(), 3);
    assert_eq!(state.peek(), Some('b'));

    state.seek(SourceOffset(999));
    assert_eq!(state.current_pos(), SourceOffset(4));
    assert_eq!(state.len(), 4);
    assert_eq!(state.remaining_len(), 0);
    assert_eq!(state.peek(), None);
  }

  #[test]
  fn test_remaining_len() {
    let state = TokenizerState::new("");
    assert_eq!(state.remaining_len(), 0);
    let state = TokenizerState::new("abcd");
    assert_eq!(state.remaining_len(), 4);
    let state = TokenizerState::default();
    assert_eq!(state.remaining_len(), 0);

    let mut state = TokenizerState::new("abcdefg");
    assert_eq!(state.remaining_len(), 7);
    state.advance(3);
    assert_eq!(state.remaining_len(), 4);
    state.advance(2);
    assert_eq!(state.remaining_len(), 2);
    state.advance(99);
    assert_eq!(state.remaining_len(), 0);
  }

  #[test]
  fn test_is_eof() {
    let state = TokenizerState::new("");
    assert!(state.is_eof());
    let state = TokenizerState::new("abcd");
    assert!(!state.is_eof());
    let state = TokenizerState::default();
    assert!(state.is_eof());

    let mut state = TokenizerState::new("abcdefg");
    assert!(!state.is_eof());
    state.advance(3);
    assert!(!state.is_eof());
    state.advance(2);
    assert!(!state.is_eof());
    state.advance(99);
    assert!(state.is_eof());
  }

  #[test]
  fn test_advance_as_str() {
    let mut state = TokenizerState::new("abcdefg");
    assert_eq!(state.advance(3).as_str(), "abc");
    assert_eq!(state.advance(2).as_str(), "de");
    assert_eq!(state.advance(99).as_str(), "fg");
    assert_eq!(state.advance(99).as_str(), "");
  }

  #[test]
  fn test_advance_positions() {
    let mut state = TokenizerState::new("abcdefg");

    let m = state.advance(3);
    assert_eq!(m.start(), SourceOffset(0));
    assert_eq!(m.end(), SourceOffset(3));
    assert!(!m.is_empty());

    let m = state.advance(2);
    assert_eq!(m.start(), SourceOffset(3));
    assert_eq!(m.end(), SourceOffset(5));
    assert!(!m.is_empty());

    let m = state.advance(99);
    assert_eq!(m.start(), SourceOffset(5));
    assert_eq!(m.end(), SourceOffset(7));
    assert!(!m.is_empty());

    let m = state.advance(99);
    assert_eq!(m.start(), SourceOffset(7));
    assert_eq!(m.end(), SourceOffset(7));
    assert!(m.is_empty());
  }

  #[test]
  fn test_read_literal_success() {
    let mut state = TokenizerState::new("abcdef");
    let m = state.read_literal("abc").unwrap();
    assert_eq!(m.as_str(), "abc");
    assert_eq!(m.start(), SourceOffset(0));
    assert_eq!(m.end(), SourceOffset(3));
    assert!(!m.is_empty());
    assert_eq!(state.current_pos(), SourceOffset(3));
  }

  #[test]
  fn test_read_literal_fail() {
    let mut state = TokenizerState::new("abcdef");
    let m = state.read_literal("abX");
    assert!(m.is_none());
    assert_eq!(state.current_pos(), SourceOffset(0));
  }

  #[test]
  fn test_read_literal_multiple() {
    let mut state = TokenizerState::new("abcdef");
    assert!(state.read_literal("ef").is_none());
    assert_eq!(state.current_pos(), SourceOffset(0));

    state.read_literal("ab").unwrap();
    state.read_literal("cd").unwrap();
    state.read_literal("ef").unwrap();
  }

  #[test]
  fn test_read_regex_success() {
    let mut state = TokenizerState::new("abcd efgh");
    let re = Regex::new(r"\w+").unwrap();

    let m = state.read_regex(&re).unwrap();
    assert_eq!(m.as_str(), "abcd");
    assert_eq!(m.start(), SourceOffset(0));
    assert_eq!(m.end(), SourceOffset(4));

    assert_eq!(state.current_pos(), SourceOffset(4));
  }

  #[test]
  fn test_read_regex_fail() {
    let mut state = TokenizerState::new("abcd efgh");
    let re = Regex::new(r"\d+").unwrap();
    let m = state.read_regex(&re);
    assert!(m.is_none());
    assert_eq!(state.current_pos(), SourceOffset(0));
  }

  #[test]
  fn test_read_regex_with_captures_success() {
    let re = Regex::new(r"([a-z]+)([0-9]+)").unwrap();

    let mut state = TokenizerState::new("abc0 XXX");
    let m = state.read_regex_with_captures(&re).unwrap();
    assert_eq!(m.as_str(), "abc0");
    assert_eq!(m.len(), 3);
    assert_eq!(m.get(0), Some("abc0"));
    assert_eq!(m.get(1), Some("abc"));
    assert_eq!(m.get(2), Some("0"));
    assert_eq!(m.get(3), None);
    assert_eq!(m.span(), Span::new(SourceOffset(0), SourceOffset(4)));
    assert_eq!(state.current_pos(), SourceOffset(4));
  }

  #[test]
  fn test_read_regex_with_captures_fail() {
    let re = Regex::new(r"([a-z]+)([0-9]+)").unwrap();

    let mut state = TokenizerState::new("XXX");
    let m = state.read_regex_with_captures(&re);
    assert!(m.is_none());
    assert_eq!(state.current_pos(), SourceOffset(0));

    let mut state = TokenizerState::new("abcX");
    let m = state.read_regex_with_captures(&re);
    assert!(m.is_none());
    assert_eq!(state.current_pos(), SourceOffset(0));
  }

  #[test]
  fn test_read_many() {
    let mut state = TokenizerState::new("abcabcabcXabc");
    let result = state.read_many(|state| state.read_literal("abc").map(|m| m.span()));
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], Span::new(SourceOffset(0), SourceOffset(3)));
    assert_eq!(result[1], Span::new(SourceOffset(3), SourceOffset(6)));
    assert_eq!(result[2], Span::new(SourceOffset(6), SourceOffset(9)));
    assert_eq!(state.current_pos(), SourceOffset(9));
  }

  #[test]
  fn test_read_some() {
    let mut state = TokenizerState::new("abcabcabcXabc");
    let result = state.read_some(|state| state.read_literal("abc").map(|m| m.span())).unwrap();
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], Span::new(SourceOffset(0), SourceOffset(3)));
    assert_eq!(result[1], Span::new(SourceOffset(3), SourceOffset(6)));
    assert_eq!(result[2], Span::new(SourceOffset(6), SourceOffset(9)));
    assert_eq!(state.current_pos(), SourceOffset(9));
  }

  #[test]
  fn test_read_many_empty() {
    let mut state = TokenizerState::new("XabcabcabcXabc");
    let result = state.read_many(|state| state.read_literal("abc").map(|m| m.span()));
    assert!(result.is_empty());
    assert_eq!(state.current_pos(), SourceOffset(0));
  }

  #[test]
  fn test_read_some_empty() {
    let mut state = TokenizerState::new("XabcabcabcXabc");
    let result = state.read_some(|state| state.read_literal("abc").map(|m| m.span()));
    assert!(result.is_none());
    assert_eq!(state.current_pos(), SourceOffset(0));
  }

  #[test]
  fn test_consume_spaces() {
    let mut state = TokenizerState::new("  abc  def");
    state.consume_spaces();
    assert_eq!(state.current_pos(), SourceOffset(2));

    // Second one has no effect, since there are no spaces to consume.
    state.consume_spaces();
    assert_eq!(state.current_pos(), SourceOffset(2));
  }

  #[test]
  fn test_peek() {
    let mut state = TokenizerState::new("abcd");
    assert_eq!(state.peek(), Some('a'));
    state.advance(1);
    assert_eq!(state.peek(), Some('b'));
    state.advance(1);
    assert_eq!(state.peek(), Some('c'));
    state.advance(1);
    assert_eq!(state.peek(), Some('d'));
    state.advance(1);
    assert_eq!(state.peek(), None);
  }
}
