
pub mod basic;
pub mod operator;

use crate::expr::Expr;

/// A language mode provides a mechanism to convert Exprs into HTML
/// code for display within the frontend.
pub trait LanguageMode {
  fn to_html(&self, out: &mut String, expr: &Expr);
}
