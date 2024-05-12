
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct CommandOptions {
  pub argument: Option<i32>,
}

impl CommandOptions {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn with_argument(mut self, argument: i32) -> Self {
    self.argument = Some(argument);
    self
  }
}
