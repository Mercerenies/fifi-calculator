
/// An `ErrorList<E>` can be thought of, roughly, as a `Vec<E>`.
/// (Recoverable) Errors can be appended to the list, and in the end,
/// the caller can get a list of what went wrong.
#[derive(Debug, Clone)]
pub struct ErrorList<E> {
  errors: Vec<E>,
}

impl<E> ErrorList<E> {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn push(&mut self, error: E) {
    self.errors.push(error)
  }

  pub fn is_empty(&self) -> bool {
    self.errors.is_empty()
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
