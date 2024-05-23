
/// Thin wrapper around `usize` that represents a position in a parsed
/// string. Usually used for error reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourceOffset(pub usize);

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
