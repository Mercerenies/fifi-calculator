
/// Options passed in addition to a command.
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct CommandOptions {
  /// The optional numerical argument for the command. This often
  /// indicates where on the stack to apply the command, or to how
  /// many elements.
  pub argument: Option<i64>,
  /// The "keep" modifier, which indicates that the command should
  /// preserve any stack elements that it utilizes, rather than
  /// popping them.
  pub keep_modifier: bool,
}

impl CommandOptions {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn with_argument(mut self, argument: i64) -> Self {
    self.argument = Some(argument);
    self
  }

  pub fn with_keep_modifier(mut self) -> Self {
    self.keep_modifier = true;
    self
  }
}
