
use serde::{Serialize, Deserialize};

/// Options passed in addition to a command.
#[derive(Default, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandOptions {
  /// The optional numerical argument for the command. This often
  /// indicates where on the stack to apply the command, or to how
  /// many elements.
  pub argument: Option<i64>,
  /// The "keep" modifier, which indicates that the command should
  /// preserve any stack elements that it utilizes, rather than
  /// popping them.
  pub keep_modifier: bool,
  /// The "hyperbolic" modifier, which dispatches several commands to
  /// similar variants.
  pub hyperbolic_modifier: bool,
  /// The "inverse" modifier, which indicates to many commands that
  /// the inverse operation to the usual should be performed.
  pub inverse_modifier: bool,
}

impl CommandOptions {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn numerical(argument: i64) -> Self {
    Self::new().with_argument(argument)
  }

  pub fn with_argument(mut self, argument: i64) -> Self {
    self.argument = Some(argument);
    self
  }

  pub fn with_keep_modifier(mut self) -> Self {
    self.keep_modifier = true;
    self
  }

  pub fn with_hyperbolic_modifier(mut self) -> Self {
    self.hyperbolic_modifier = true;
    self
  }

  pub fn with_inverse_modifier(mut self) -> Self {
    self.inverse_modifier = true;
    self
  }
}
