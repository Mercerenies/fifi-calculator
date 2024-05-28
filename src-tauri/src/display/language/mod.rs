
pub mod basic;

use crate::expr::Expr;
use crate::error::Error;

/// A language mode provides a mechanism to convert Exprs into HTML
/// code for display within the frontend.
///
/// A language mode must also provide a mechanism to parse plaintext
/// into Exprs.
pub trait LanguageMode {
  fn write_to_html(&self, out: &mut String, expr: &Expr);
  fn parse(&self, text: &str) -> Result<Expr, Error>;

  fn to_html(&self, expr: &Expr) -> String {
    let mut out = String::new();
    self.write_to_html(&mut out, expr);
    out
  }
}
