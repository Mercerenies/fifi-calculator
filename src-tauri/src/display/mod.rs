
pub mod renderer;

use crate::expr::Expr;
use renderer::{Renderer, DefaultRenderer};

pub struct DisplaySettings {
  pub renderer: Box<dyn Renderer + Send + Sync>,
}

impl DisplaySettings {
  pub fn to_html(&self, expr: &Expr) -> String {
    let mut result = String::new();
    self.renderer.to_html(&mut result, expr);
    result
  }
}

impl Default for DisplaySettings {
  fn default() -> Self {
    let renderer = Box::new(DefaultRenderer);
    DisplaySettings {
      renderer,
    }
  }
}
