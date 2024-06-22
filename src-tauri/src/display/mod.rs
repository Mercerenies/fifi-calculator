
pub mod language;

use crate::expr::Expr;
use language::LanguageMode;
use language::basic::BasicLanguageMode;
use language::graphics::GraphicsLanguageMode;

pub struct DisplaySettings {
  base_language_mode: Box<dyn LanguageMode + Send + Sync>,
}

impl DisplaySettings {
  pub fn language_mode(&self) -> Box<dyn LanguageMode + '_> {
    Box::new(
      GraphicsLanguageMode::new(self.base_language_mode.as_ref()),
    )
  }
}

impl DisplaySettings {
  pub fn to_html(&self, expr: &Expr) -> String {
    let language_mode = self.language_mode();
    language_mode.to_html(expr)
  }
  pub fn to_html_for_parsing(&self, expr: &Expr) -> String {
    let language_mode = self.language_mode();
    let language_mode = language_mode.to_reversible_language_mode();
    language_mode.to_html(expr)
  }
}

impl Default for DisplaySettings {
  fn default() -> Self {
    let base_language_mode = Box::new(BasicLanguageMode::from_common_operators());
    DisplaySettings {
      base_language_mode,
    }
  }
}
