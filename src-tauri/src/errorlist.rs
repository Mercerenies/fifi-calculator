
/// An `ErrorList<E>` can be thought of, roughly, as a `Vec<E>`.
/// (Recoverable) Errors can be appended to the list, and in the end,
/// the caller can get a list of what went wrong.
#[derive(Debug, Clone)]
pub struct ErrorList<E> {
  errors: Vec<E>,
}

impl<E> ErrorList<E> {
  /// A new, empty error list.
  pub fn new() -> Self {
    Self::default()
  }

  /// A new error list which contains the elements of the given
  /// vector, in order.
  pub fn of(errors: Vec<E>) -> Self {
    Self { errors }
  }

  pub fn push(&mut self, error: E) {
    self.errors.push(error)
  }

  /// Pushes all errors from `other` onto the end of `self`, leaving
  /// `other` empty.
  pub fn append(&mut self, other: &mut Self) {
    self.errors.append(&mut other.errors)
  }

  pub fn is_empty(&self) -> bool {
    self.errors.is_empty()
  }

  pub fn len(&self) -> usize {
    self.errors.len()
  }

  pub fn unwrap_result_or_else<T, E1, F>(&mut self, result: Result<T, E1>, default: F) -> T
  where E: From<E1>,
        F: FnOnce() -> T {
    match result {
      Ok(x) => {
        x
      }
      Err(err) => {
        self.push(err.into());
        default()
      }
    }
  }

  pub fn unwrap_result_or<T, E1>(&mut self, result: Result<T, E1>, default: T) -> T
  where E: From<E1> {
    self.unwrap_result_or_else(result, || default)
  }

  /// Equivalent to `From::from` but specialized to `Vec` to improve
  /// type inference.
  pub fn into_vec(self) -> Vec<E> {
    self.errors
  }
}

impl<E> IntoIterator for ErrorList<E> {
  type Item = E;
  type IntoIter = ::std::vec::IntoIter<E>;

  fn into_iter(self) -> Self::IntoIter {
    self.errors.into_iter()
  }
}

impl<E> Default for ErrorList<E> {
  fn default() -> Self {
    Self { errors: Vec::new() }
  }
}

impl<E> From<ErrorList<E>> for Vec<E> {
  fn from(error_list: ErrorList<E>) -> Self {
    error_list.into_vec()
  }
}

impl<E> FromIterator<E> for ErrorList<E> {
  fn from_iter<I: IntoIterator<Item = E>>(iter: I) -> Self {
    Self { errors: iter.into_iter().collect() }
  }
}
