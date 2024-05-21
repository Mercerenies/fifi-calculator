
pub mod language;

use crate::expr::Expr;
use language::LanguageMode;
use language::basic::DefaultLanguageMode;

pub struct DisplaySettings {
  pub language_mode: Box<dyn LanguageMode + Send + Sync>,
}

impl DisplaySettings {
  pub fn to_html(&self, expr: &Expr) -> String {
    let mut result = String::new();
    self.language_mode.to_html(&mut result, expr);
    result
  }
}

impl Default for DisplaySettings {
  fn default() -> Self {
    let language_mode = Box::new(DefaultLanguageMode);
    DisplaySettings {
      language_mode,
    }
  }
}
