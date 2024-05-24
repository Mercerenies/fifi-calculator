
use std::fmt::{self, Display, Formatter};

/// Thin wrapper around `usize` that represents a position in a parsed
/// string. Usually used for error reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourceOffset(pub usize);

/// A span of source offsets. Spans should be considered half-open
/// intervals, with `start` being included and `end` being excluded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
  pub start: SourceOffset,
  pub end: SourceOffset,
}

impl Span {
  pub fn new(start: SourceOffset, end: SourceOffset) -> Self {
    Self { start, end }
  }
}

impl From<usize> for SourceOffset {
  fn from(i: usize) -> Self {
    SourceOffset(i)
  }
}

impl From<SourceOffset> for usize {
  fn from(i: SourceOffset) -> Self {
    i.0
  }
}

impl Display for SourceOffset {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl Display for Span {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "{}-{}", self.start, self.end)
  }
}