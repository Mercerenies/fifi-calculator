
use std::iter;

/// All variables and operators in the expression language are, by
/// default, ASCII strings. However, for many names, it's convenient
/// to use Unicode equivalents. A `UnicodeAlias` defines a canonical
/// ASCII name for a variable, function, or operator, as well as one
/// or more Unicode names which should be considered equivalent.
///
/// One of these Unicode names is considered the most correct name and
/// will be used when pretty-printing the original name, while the
/// others (if any) are not used in printing but will be accepted as
/// equivalent in input.
#[derive(Debug, Clone)]
pub struct UnicodeAlias {
  pub(super) ascii_name: String,
  pub(super) best_unicode_name: String,
  pub(super) other_unicode_names: Vec<String>,
}

impl UnicodeAlias {
  pub fn new(
    ascii_name: impl Into<String>,
    best_unicode_name: impl Into<String>,
    other_unicode_names: Vec<String>,
  ) -> Self {
    Self {
      ascii_name: ascii_name.into(),
      best_unicode_name: best_unicode_name.into(),
      other_unicode_names,
    }
  }

  pub fn simple(ascii_name: impl Into<String>, unicode_name: impl Into<String>) -> Self {
    Self::new(ascii_name, unicode_name, Vec::new())
  }

  pub(super) fn unicode_names(&self) -> impl Iterator<Item = &str> {
    iter::once(&self.best_unicode_name).chain(&self.other_unicode_names)
      .map(String::as_str)
  }
}
