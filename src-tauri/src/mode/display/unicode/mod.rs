
mod alias;
mod table;

pub use alias::UnicodeAlias;
pub use table::{UnicodeAliasTable, UnicodeTableError};

pub fn common_unicode_aliases() -> UnicodeAliasTable {
  UnicodeAliasTable::new(vec![
    UnicodeAlias::simple("<=", "≤"),
    UnicodeAlias::simple("=>", "≥"),
    UnicodeAlias::simple("!=", "≠"),
    UnicodeAlias::simple("inf", "∞"),
    UnicodeAlias::simple("uinf", "⧝"),
  ]).unwrap()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_common_unicode_aliases_contains_no_duplicates() {
    // Simply instantiate the table, to ensure that the constructor
    // doesn't return an `Err`.
    common_unicode_aliases();
  }
}
