
pub mod basic;

use crate::expr::Expr;
use crate::parsing::operator::Precedence;

/// A language mode provides a mechanism to convert Exprs into HTML
/// code for display within the frontend.
///
/// A language mode must also provide a mechanism to parse plaintext
/// into Exprs.
pub trait LanguageMode {
  fn write_to_html(&self, out: &mut String, expr: &Expr, prec: Precedence);
  fn parse(&self, text: &str) -> anyhow::Result<Expr>;

  fn to_html(&self, expr: &Expr) -> String {
    let mut out = String::new();
    self.write_to_html(&mut out, expr, Precedence::MIN);
    out
  }
}

/// Helper function to output a list of values, separated by a chosen
/// delimiter.
pub fn output_sep_by<T, I, F>(
  out: &mut String,
  elems: I,
  delimiter: &str,
  printer: F,
)
where I: IntoIterator<Item = T>,
      F: Fn(&mut String, T) {
  let mut elems = elems.into_iter();
  if let Some(first) = elems.next() {
    printer(out, first);
    for elem in elems {
      out.push_str(delimiter);
      printer(out, elem);
    }
  }
}
