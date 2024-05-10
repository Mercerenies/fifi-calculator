
pub mod renderer;

use renderer::{Renderer, DefaultRenderer};

pub struct DisplaySettings {
  pub renderer: Box<dyn Renderer + Send + Sync>,
}

impl Default for DisplaySettings {
  fn default() -> Self {
    let renderer = Box::new(DefaultRenderer);
    DisplaySettings {
      renderer,
    }
  }
}
