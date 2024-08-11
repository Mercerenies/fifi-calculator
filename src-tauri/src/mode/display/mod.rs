
pub mod language;
pub mod unicode;

use crate::expr::Expr;
use language::{LanguageMode, LanguageSettings};
use language::basic::BasicLanguageMode;
use language::graphics::GraphicsLanguageMode;

pub struct DisplaySettings {
  pub base_language_mode: Box<dyn LanguageMode + Send + Sync>,
  pub is_graphics_enabled: bool,
  pub language_settings: LanguageSettings,
}

impl DisplaySettings {
  pub fn new(language_mode: impl LanguageMode + Send + Sync + 'static, language_settings: LanguageSettings) -> Self {
    DisplaySettings {
      base_language_mode: Box::new(language_mode),
      is_graphics_enabled: true,
      language_settings,
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
    language_mode.to_html(expr, &self.language_settings)
  }
  pub fn to_html_for_parsing(&self, expr: &Expr) -> String {
    let language_mode = self.language_mode();
    let language_mode = language_mode.to_reversible_language_mode();
    language_mode.to_html(expr, &self.language_settings)
  }
}

impl Default for DisplaySettings {
  fn default() -> Self {
    let base_language_mode = BasicLanguageMode::from_common_operators();
    DisplaySettings::new(base_language_mode, LanguageSettings::default())
  }
}
