
pub mod language;
pub mod unicode;

use crate::expr::Expr;
use crate::expr::datetime::SystemDateTimeSource;
use language::{LanguageMode, LanguageSettings};
use language::basic::BasicLanguageMode;
use language::graphics::GraphicsLanguageMode;

use std::sync::Arc;

pub struct DisplaySettings {
  /// The current language mode. We store this as an [`Arc`] rather
  /// than a simple [`Box`] so that the language mode can be cheaply
  /// copied onto the undo stack.
  pub base_language_mode: Arc<dyn LanguageMode + Send + Sync>,
  pub is_graphics_enabled: bool,
  pub language_settings: LanguageSettings,
}

impl DisplaySettings {
  pub fn new(language_mode: impl LanguageMode + Send + Sync + 'static, language_settings: LanguageSettings) -> Self {
    DisplaySettings {
      base_language_mode: Arc::new(language_mode),
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
    let base_language_mode = BasicLanguageMode::from_common_operators()
      .with_time_source(Arc::new(SystemDateTimeSource));
    DisplaySettings::new(base_language_mode, LanguageSettings::default())
  }
}
