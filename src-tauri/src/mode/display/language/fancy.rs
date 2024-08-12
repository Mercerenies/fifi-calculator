
use super::{LanguageMode, LanguageModeEngine};
use crate::parsing::operator::Precedence;
use crate::parsing::operator::table::EXPONENT_PRECEDENCE;
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

  fn translate_to_unicode<'a>(&'a self, engine: &LanguageModeEngine, ascii_name: &'a str) -> &'a str {
    if !engine.language_settings().prefers_unicode_output {
      return ascii_name;
    }
    self.unicode_table.get_unicode(ascii_name).unwrap_or(ascii_name)
  }

  fn write_var(&self, engine: &LanguageModeEngine, out: &mut String, var: &Var) {
    let var_name = self.translate_to_unicode(engine, var.as_str());
    let var_name = encode_safe(var_name);
    out.push_str(r#"<span class="mathy-text">"#);
    out.push_str(var_name.as_ref());
    out.push_str("</span>");
  }

  fn write_exponent(&self, engine: &LanguageModeEngine, out: &mut String, args: &[Expr], prec: Precedence) {
    assert!(args.len() == 2);
    let [base, exp] = args else { unreachable!() };

    let needs_parens = prec > EXPONENT_PRECEDENCE;

    out.push_str("<span>");
    if needs_parens {
      out.push('(');
    }

    out.push_str("<span>");
    engine.write_to_html(out, base, EXPONENT_PRECEDENCE.incremented());
    out.push_str("</span>");
    out.push_str("<sup>");
    engine.write_to_html(out, exp, EXPONENT_PRECEDENCE);
    out.push_str("</sup>");

    if needs_parens {
      out.push(')');
    }
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
        self.write_var(engine, out, v)
      }
      Expr::Call(f, args) => {
        if f == "^" && args.len() == 2 {
          self.write_exponent(engine, out, args, prec)
        } else {
          self.inner_mode.write_to_html(engine, out, expr, prec)
        }
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
