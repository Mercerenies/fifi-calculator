
use crate::state::ApplicationState;
use crate::error::Error;
use super::options::CommandOptions;

pub trait Command {
  fn run_command(&self, state: &mut ApplicationState, ctx: &CommandContext) -> Result<(), Error>;
}

#[derive(Clone, Debug)]
pub struct CommandContext {
  pub opts: CommandOptions,
}
