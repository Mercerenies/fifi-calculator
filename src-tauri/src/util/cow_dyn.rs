
use std::convert::AsRef;
use std::borrow::Borrow;
use std::ops::Deref;
use std::fmt::{self, Display, Formatter};
use std::hash::{Hash, Hasher};

/// As [`Cow`](std::borrow::Cow) but designed for use with trait
/// objects where we don't know if we intend to borrow or own the
/// trait object a priori.
#[derive(Debug, Clone)]
pub enum CowDyn<'a, T: ?Sized + 'a> {
  Borrowed(&'a T),
  Owned(Box<T>),
}

impl<'a, T: ?Sized + 'a> CowDyn<'a, T> {
  pub fn is_borrowed(&self) -> bool {
    matches!(self, CowDyn::Borrowed(_))
  }

  pub fn is_owned(&self) -> bool {
    matches!(self, CowDyn::Owned(_))
  }
}

impl<'a, T> AsRef<T> for CowDyn<'a, T> {
  fn as_ref(&self) -> &T {
    match self {
      CowDyn::Borrowed(b) => b,
      CowDyn::Owned(o) => o.as_ref(),
    }
  }
}

impl<'a, T> Borrow<T> for CowDyn<'a, T> {
  fn borrow(&self) -> &T {
    match self {
      CowDyn::Borrowed(b) => b,
      CowDyn::Owned(o) => o.as_ref(),
    }
  }
}

impl<'a, T: Default> Default for CowDyn<'a, T> {
  fn default() -> Self {
    CowDyn::Owned(Box::default())
  }
}

impl<'a, T> Deref for CowDyn<'a, T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    self.as_ref()
  }
}

impl<'a, T: Display> Display for CowDyn<'a, T> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    Display::fmt(self.as_ref(), f)
  }
}

impl<'a, T: PartialEq> PartialEq for CowDyn<'a, T> {
  fn eq(&self, other: &Self) -> bool {
    self.as_ref() == other.as_ref()
  }
}

impl<'a, T: PartialEq> PartialEq<T> for CowDyn<'a, T> {
  fn eq(&self, other: &T) -> bool {
    self.as_ref() == other
  }
}

impl<'a, T: Eq> Eq for CowDyn<'a, T> {}

impl<'a, T: Hash> Hash for CowDyn<'a, T> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    Hash::hash(self.as_ref(), state)
  }
}
