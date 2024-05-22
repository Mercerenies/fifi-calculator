
pub mod basic;
pub mod operator;

use crate::expr::Expr;

/// A language mode provides a mechanism to convert Exprs into HTML
/// code for display within the frontend.
pub trait LanguageMode {
  fn write_to_html(&self, out: &mut String, expr: &Expr);

  fn to_html(&self, expr: &Expr) -> String {
    let mut out = String::new();
    self.write_to_html(&mut out, expr);
    out
  }
}
