
pub mod language;

use crate::expr::Expr;
use language::LanguageMode;
use language::basic::BasicLanguageMode;

pub struct DisplaySettings {
  pub language_mode: Box<dyn LanguageMode + Send + Sync>,
}

impl DisplaySettings {
  pub fn to_html(&self, expr: &Expr) -> String {
    self.language_mode.to_html(expr)
  }
}

impl Default for DisplaySettings {
  fn default() -> Self {
    let language_mode = Box::new(BasicLanguageMode::from_common_operators());
    DisplaySettings {
      language_mode,
    }
  }
}
