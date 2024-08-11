
pub mod basic;
pub mod graphics;

use crate::util::cow_dyn::CowDyn;
use crate::util::radix::Radix;
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

  /// Converts `self` into a `LanguageMode` which is the inverse of
  /// the `write_to_html` implementation.
  ///
  /// Many language modes, in order to produce pretty output, have
  /// parsers that are incompatible with their renderer. This function
  /// produces a renderer which is compatible with the parser, even if
  /// it might be simpler than `self`.
  fn to_reversible_language_mode(&self) -> CowDyn<dyn LanguageMode>;

  fn to_html(&self, expr: &Expr, language_settings: &LanguageSettings) -> String {
    let engine = LanguageModeEngine { data: self.to_trait_object(), language_settings };

    let mut out = String::new();
    self.write_to_html(&engine, &mut out, expr, Precedence::MIN);
    out
  }
}

/// Helper struct passed to [`LanguageMode`] methods. Any recursive
/// calls to `write_to_html` from [`LanguageMode::write_to_html`]
/// should be made on this argument, NOT on the `LanguageMode` itself,
/// so that nested language modes correctly implement open recursion.
/// Additionally, this structure stores various additional display
/// settings that are passed to the language mode as parameters.
pub struct LanguageModeEngine<'a, 'b> {
  data: &'a dyn LanguageMode,
  language_settings: &'b LanguageSettings,
}

#[derive(Debug, Clone)]
pub struct LanguageSettings {
  /// The preferred radix for outputting real numbers.
  pub preferred_radix: Radix,
  /// If true, the user prefers certain operator, function, and
  /// variable names to be replaced with Unicode equivalents in
  /// output. This does NOT affect input, which always accepts either
  /// an ASCII name or its Unicode equivalent(s), regardless of this
  /// flag.
  pub prefers_unicode_output: bool,
}

impl<'a, 'b> LanguageModeEngine<'a, 'b> {
  pub fn write_to_html(&self, out: &mut String, expr: &Expr, prec: Precedence) {
    self.data.write_to_html(self, out, expr, prec);
  }

  pub fn language_settings(&self) -> &LanguageSettings {
    self.language_settings
  }
}

impl Default for LanguageSettings {
  fn default() -> Self {
    LanguageSettings {
      preferred_radix: Radix::DECIMAL,
      prefers_unicode_output: true,
    }
  }
}

/// Implementation of `LanguageMode` lifted to a reference type.
impl<'a, T: LanguageMode + ?Sized> LanguageMode for &'a T {
  fn write_to_html(&self, engine: &LanguageModeEngine, out: &mut String, expr: &Expr, prec: Precedence) {
    (**self).write_to_html(engine, out, expr, prec);
  }

  fn parse(&self, text: &str) -> anyhow::Result<Expr> {
    (**self).parse(text)
  }

  fn to_trait_object(&self) -> &dyn LanguageMode {
    (**self).to_trait_object()
  }

  fn to_reversible_language_mode(&self) -> CowDyn<dyn LanguageMode> {
    (**self).to_reversible_language_mode()
  }
}

impl<T: LanguageMode + ?Sized> LanguageMode for Box<T> {
  fn write_to_html(&self, engine: &LanguageModeEngine, out: &mut String, expr: &Expr, prec: Precedence) {
    (**self).write_to_html(engine, out, expr, prec);
  }

  fn parse(&self, text: &str) -> anyhow::Result<Expr> {
    (**self).parse(text)
  }

  fn to_trait_object(&self) -> &dyn LanguageMode {
    (**self).to_trait_object()
  }

  fn to_reversible_language_mode(&self) -> CowDyn<dyn LanguageMode> {
    (**self).to_reversible_language_mode()
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
