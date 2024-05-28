
use nom::{InputTakeAtPosition, IResult, Err, Needed};
use nom::error::{ParseError, ErrorKind};

#[derive(Debug)]
pub struct TokenStream<'a, T> {
  input: &'a [T],
}

impl<'a, T> TokenStream<'a, T> {
  pub fn new(input: &'a [T]) -> Self {
    Self { input }
  }

  pub fn iter(&self) -> impl Iterator<Item = &'a T> {
    self.input.iter()
  }

  pub fn len(&self) -> usize {
    self.input.len()
  }

  /// Returns two `TokenStream` objects. Panics if `index >
  /// self.len()`.
  pub fn split_at(&self, index: usize) -> (Self, Self) {
    let (prefix, suffix) = self.input.split_at(index);
    (TokenStream::new(prefix), TokenStream::new(suffix))
  }
}

impl<'a, T> Clone for TokenStream<'a, T> {
  fn clone(&self) -> Self {
    Self {
      input: self.input,
    }
  }
}

impl<'a, T> From<TokenStream<'a, T>> for &'a [T] {
  fn from(stream: TokenStream<'a, T>) -> Self {
    stream.input
  }
}

impl<'a, T> InputTakeAtPosition for TokenStream<'a, T> {
  type Item = &'a T;

  fn split_at_position<P, E>(&self, predicate: P) -> IResult<Self, Self, E>
  where P : Fn(Self::Item) -> bool,
        E : ParseError<Self> {
    match self.iter().position(predicate) {
      Some(i) => {
        let (prefix, suffix) = self.split_at(i);
        Ok((suffix, prefix))
      }
      None => Err(Err::Incomplete(Needed::new(1)))
    }
  }

  fn split_at_position1<P, E>(&self, predicate: P, e: ErrorKind) -> IResult<Self, Self, E>
  where P : Fn(Self::Item) -> bool,
        E : ParseError<Self> {
    match self.iter().position(predicate) {
      Some(0) => Err(Err::Error(E::from_error_kind(self.clone(), e))),
      Some(i) => {
        let (prefix, suffix) = self.split_at(i);
        Ok((suffix, prefix))
      }
      None => Err(Err::Incomplete(Needed::new(1))),
    }
  }

  fn split_at_position_complete<P, E>(&self, predicate: P) -> IResult<Self, Self, E>
  where P : Fn(Self::Item) -> bool,
        E : ParseError<Self> {
    match self.split_at_position(predicate) {
      Err(Err::Incomplete(_)) => {
        let (prefix, suffix) = self.split_at(self.len());
        Ok((suffix, prefix))
      }
      res => res,
    }
  }

  fn split_at_position1_complete<P, E>(&self, predicate: P, e: ErrorKind) -> IResult<Self, Self, E>
  where P : Fn(Self::Item) -> bool,
        E : ParseError<Self> {
    match self.split_at_position1(predicate, e) {
      Err(Err::Incomplete(_)) => {
        if self.len() == 0 {
          Err(Err::Error(E::from_error_kind(self.clone(), e)))
        } else {
          let (prefix, suffix) = self.split_at(self.len());
          Ok((suffix, prefix))
        }
      }
      res => res,
    }
  }
}
