
use super::base::{Command, CommandContext, CommandOutput};
use crate::state::ApplicationState;

/// Nullary command, performs no action.
#[derive(Debug, Clone)]
pub struct NullaryCommand;

impl Command for NullaryCommand {
  fn run_command(&self, _: &mut ApplicationState, _: Vec<String>, _: &CommandContext) -> anyhow::Result<CommandOutput> {
    Ok(CommandOutput::success())
  }
}
