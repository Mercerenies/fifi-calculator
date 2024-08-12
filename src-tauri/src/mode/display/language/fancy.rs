
use super::{LanguageMode, LanguageModeEngine};
use crate::parsing::operator::Precedence;
use crate::mode::display::unicode::{UnicodeAliasTable, common_unicode_aliases};
use crate::util::cow_dyn::CowDyn;
use crate::expr::Expr;
use crate::expr::var::Var;
use crate::expr::atom::Atom;

use html_escape::encode_safe;

/// The fancy language mode, which uses HTML to render certain
/// function types using a more mathematical notation. In any case
/// that this language mode is not equipped to handle, it falls back
/// to its inner mode.
#[derive(Clone, Debug)]
pub struct FancyLanguageMode<L> {
  inner_mode: L,
  unicode_table: UnicodeAliasTable,
}

impl<L: LanguageMode> FancyLanguageMode<L> {
  pub fn new(inner_mode: L) -> Self {
    Self {
      inner_mode,
      unicode_table: UnicodeAliasTable::default(),
    }
  }

  pub fn with_unicode_table(mut self, table: UnicodeAliasTable) -> Self {
    self.unicode_table = table;
    self
  }

  pub fn from_common_unicode(inner_mode: L) -> Self {
    Self::new(inner_mode)
      .with_unicode_table(common_unicode_aliases())
  }

  fn write_var(out: &mut String, var: &Var) {
    out.push_str(r#"<span class="mathy-text">"#);
    out.push_str(var.as_str());
    out.push_str("</span>");
  }
}

impl<L: LanguageMode + Default> Default for FancyLanguageMode<L> {
  fn default() -> Self {
    Self::new(L::default())
  }
}

impl<L: LanguageMode> LanguageMode for FancyLanguageMode<L> {
  fn write_to_html(&self, engine: &LanguageModeEngine, out: &mut String, expr: &Expr, prec: Precedence) {
    match expr {
      Expr::Atom(Atom::Number(_) | Atom::String(_)) => {
        self.inner_mode.write_to_html(engine, out, expr, prec)
      }
      Expr::Atom(Atom::Var(v)) => {
        Self::write_var(out, v)
      }
      Expr::Call(_, _) => {
        self.inner_mode.write_to_html(engine, out, expr, prec)
      }
    }
  }

  fn to_trait_object(&self) -> &dyn LanguageMode {
    self
  }

  fn to_reversible_language_mode(&self) -> CowDyn<dyn LanguageMode> {
    self.inner_mode.to_reversible_language_mode()
  }

  fn parse(&self, text: &str) -> anyhow::Result<Expr> {
    self.inner_mode.parse(text)
  }
}
