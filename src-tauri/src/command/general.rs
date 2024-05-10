
use super::base::Command;
use crate::state::ApplicationState;
use crate::error::Error;

/// General-purpose [Command] implementation that simply runs a given
/// function.
pub struct GeneralCommand<F> {
  body: F,
}

impl<F: Fn(&mut ApplicationState) -> Result<(), Error>> GeneralCommand<F> {
  pub fn new(body: F) -> GeneralCommand<F> {
    GeneralCommand {
      body
    }
  }
}

impl<F: Fn(&mut ApplicationState) -> Result<(), Error>> Command for GeneralCommand<F> {
  fn run_command(&self, state: &mut ApplicationState) -> Result<(), Error> {
    (self.body)(state)
  }
}
