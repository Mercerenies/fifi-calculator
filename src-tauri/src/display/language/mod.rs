
pub mod basic;
pub mod graphics;

use crate::expr::Expr;
use crate::parsing::operator::Precedence;

/// A language mode provides a mechanism to convert Exprs into HTML
/// code for display within the frontend.
///
/// A language mode must also provide a mechanism to parse plaintext
/// into Exprs.
pub trait LanguageMode {
  fn write_to_html(&self, engine: &LanguageModeEngine, out: &mut String, expr: &Expr, prec: Precedence);
  fn parse(&self, text: &str) -> anyhow::Result<Expr>;

  /// Converts `self` into a `dyn LanguageMode`. The implementation of
  /// this method should _always_ be
  ///
  /// ```
  /// fn to_trait_object(&self) -> &dyn LanguageMode {
  ///   self
  /// }
  /// ```
  ///
  /// The reason this method exists is that `write_to_html` needs to
  /// know the top-level language mode it was invoked on, in order to
  /// implement open recursion and allow composition of language
  /// modes.
  fn to_trait_object(&self) -> &dyn LanguageMode;

  fn to_html(&self, expr: &Expr) -> String {
    let engine = LanguageModeEngine { data: self.to_trait_object() };

    let mut out = String::new();
    self.write_to_html(&engine, &mut out, expr, Precedence::MIN);
    out
  }
}

/// Helper struct to implement open recursion on `LanguageMode` so
/// that multiple language modes can be composed in a convenient way.
pub struct LanguageModeEngine<'a> {
  data: &'a dyn LanguageMode,
}

impl<'a> LanguageModeEngine<'a> {
  pub fn write_to_html(&self, out: &mut String, expr: &Expr, prec: Precedence) {
    self.data.write_to_html(self, out, expr, prec);
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
