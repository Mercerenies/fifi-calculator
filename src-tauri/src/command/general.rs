
use super::base::{Command, CommandContext, CommandOutput};
use crate::state::ApplicationState;
use crate::error::Error;

/// General-purpose [Command] implementation that simply runs a given
/// function.
pub struct GeneralCommand<F> {
  body: F,
}

impl<F> GeneralCommand<F>
where F: Fn(&mut ApplicationState, &CommandContext) -> Result<CommandOutput, Error> {
  pub fn new(body: F) -> GeneralCommand<F> {
    GeneralCommand {
      body
    }
  }
}

impl<F> Command for GeneralCommand<F>
where F: Fn(&mut ApplicationState, &CommandContext) -> Result<CommandOutput, Error> {
  fn run_command(&self, state: &mut ApplicationState, context: &CommandContext) -> Result<CommandOutput, Error> {
    (self.body)(state, context)
  }
}
