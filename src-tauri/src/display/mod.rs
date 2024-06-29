
pub mod language;

use crate::expr::Expr;
use language::LanguageMode;
use language::basic::BasicLanguageMode;
use language::graphics::GraphicsLanguageMode;

pub struct DisplaySettings {
  pub base_language_mode: Box<dyn LanguageMode + Send + Sync>,
  pub is_graphics_enabled: bool,
}

impl DisplaySettings {
  pub fn new(language_mode: impl LanguageMode + Send + Sync + 'static) -> Self {
    DisplaySettings {
      base_language_mode: Box::new(language_mode),
      is_graphics_enabled: true,
    }
  }

  pub fn language_mode(&self) -> Box<dyn LanguageMode + '_> {
    let base_language_mode = self.base_language_mode.as_ref();
    if self.is_graphics_enabled {
      Box::new(GraphicsLanguageMode::new(base_language_mode))
    } else {
      Box::new(base_language_mode)
    }
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
    let base_language_mode = BasicLanguageMode::from_common_operators();
    DisplaySettings::new(base_language_mode)
  }
}
