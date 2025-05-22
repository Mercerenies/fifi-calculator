
use std::convert::AsRef;
use std::borrow::Borrow;
use std::ops::Deref;
use std::fmt::{self, Display, Formatter};
use std::hash::{Hash, Hasher};

/// As [`Cow`](std::borrow::Cow) but designed for use with trait
/// objects where we don't know if we intend to borrow or own the
/// trait object a priori.
#[derive(Debug, Clone)]
pub enum CowDyn<'a, T: ?Sized> {
  Borrowed(&'a T),
  Owned(Box<T>),
}

impl<T: ?Sized> CowDyn<'_, T> {
  pub fn is_borrowed(&self) -> bool {
    matches!(self, CowDyn::Borrowed(_))
  }

  pub fn is_owned(&self) -> bool {
    matches!(self, CowDyn::Owned(_))
  }
}

impl<T: ?Sized> AsRef<T> for CowDyn<'_, T> {
  fn as_ref(&self) -> &T {
    match self {
      CowDyn::Borrowed(b) => b,
      CowDyn::Owned(o) => o.as_ref(),
    }
  }
}

impl<T: ?Sized> Borrow<T> for CowDyn<'_, T> {
  fn borrow(&self) -> &T {
    match self {
      CowDyn::Borrowed(b) => b,
      CowDyn::Owned(o) => o.as_ref(),
    }
  }
}

impl<T: Default> Default for CowDyn<'_, T> {
  fn default() -> Self {
    CowDyn::Owned(Box::default())
  }
}

impl<T: ?Sized> Deref for CowDyn<'_, T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    self.as_ref()
  }
}

impl<T: Display + ?Sized> Display for CowDyn<'_, T> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    Display::fmt(self.as_ref(), f)
  }
}

impl<T: PartialEq + ?Sized> PartialEq for CowDyn<'_, T> {
  fn eq(&self, other: &Self) -> bool {
    self.as_ref() == other.as_ref()
  }
}

impl<T: PartialEq + ?Sized> PartialEq<T> for CowDyn<'_, T> {
  fn eq(&self, other: &T) -> bool {
    self.as_ref() == other
  }
}

impl<T: Eq + ?Sized> Eq for CowDyn<'_, T> {}

impl<T: Hash + ?Sized> Hash for CowDyn<'_, T> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    Hash::hash(self.as_ref(), state)
  }
}
