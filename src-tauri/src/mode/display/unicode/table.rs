
use super::alias::UnicodeAlias;

use thiserror::Error;

use std::collections::HashMap;

/// A table mapping canonical ASCII names to Unicode aliases and vice
/// versa. Every ASCII name that appears in the table must map to a
/// single, canonical Unicode equivalent. But multiple Unicode names
/// can map to the same ASCII name.
///
/// This mapping is its own one-sided inverse. That is, if you start
/// with an ASCII name present in the table, get the Unicode name, and
/// then go back, you will get the original ASCII name. But the same
/// is NOT true if you start with a Unicode name.
#[derive(Debug, Clone, Default)]
pub struct UnicodeAliasTable {
  ascii_to_unicode: HashMap<String, String>,
  unicode_to_ascii: HashMap<String, String>,
}

#[derive(Debug, Clone, Error)]
#[non_exhaustive]
pub enum UnicodeTableError {
  #[error("ASCII name already exists: {0}")]
  AsciiNameAlreadyExists(String),
  #[error("Unicode name already exists: {0}")]
  UnicodeNameAlreadyExists(String),
}

impl UnicodeAliasTable {
  pub fn new(aliases: impl IntoIterator<Item = UnicodeAlias>) -> Result<Self, UnicodeTableError> {
    let mut table = Self::default();
    for alias in aliases {
      table.insert(alias)?;
    }
    Ok(table)
  }

  pub fn get_unicode(&self, ascii_name: &str) -> Option<&str> {
    self.ascii_to_unicode.get(ascii_name).map(|s| s.as_str())
  }

  pub fn get_ascii(&self, unicode_name: &str) -> Option<&str> {
    self.unicode_to_ascii.get(unicode_name).map(|s| s.as_str())
  }

  /// Attempts to insert a new alias into the table. If any of the
  /// names in the new alias struct are already present in the table,
  /// this method returns an error without modifying the table.
  pub fn insert(&mut self, alias: UnicodeAlias) -> Result<(), UnicodeTableError> {
    if self.ascii_to_unicode.contains_key(&alias.ascii_name) {
      return Err(UnicodeTableError::AsciiNameAlreadyExists(alias.ascii_name));
    }
    for name in alias.unicode_names() {
      if self.unicode_to_ascii.contains_key(name) {
        return Err(UnicodeTableError::UnicodeNameAlreadyExists(name.to_owned()));
      }
    }
    self.ascii_to_unicode.insert(alias.ascii_name.clone(), alias.best_unicode_name.clone());
    self.unicode_to_ascii.insert(alias.best_unicode_name, alias.ascii_name.clone());
    for name in alias.other_unicode_names {
      self.unicode_to_ascii.insert(name, alias.ascii_name.clone());
    }
    Ok(())
  }
}
