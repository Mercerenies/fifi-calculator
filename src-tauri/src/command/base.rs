
use crate::state::ApplicationState;
use crate::error::Error;

pub trait Command {
  fn run_command(&self, state: &mut ApplicationState) -> Result<(), Error>;
}
