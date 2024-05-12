
use super::base::{Command, CommandContext};
use crate::state::ApplicationState;
use crate::error::Error;

/// General-purpose [Command] implementation that simply runs a given
/// function.
pub struct GeneralCommand<F> {
  body: F,
}

impl<F: Fn(&mut ApplicationState, &CommandContext) -> Result<(), Error>> GeneralCommand<F> {
  pub fn new(body: F) -> GeneralCommand<F> {
    GeneralCommand {
      body
    }
  }
}

impl<F: Fn(&mut ApplicationState, &CommandContext) -> Result<(), Error>> Command for GeneralCommand<F> {
  fn run_command(&self, state: &mut ApplicationState, context: &CommandContext) -> Result<(), Error> {
    (self.body)(state, context)
  }
}
