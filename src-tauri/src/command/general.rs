
use super::base::{Command, CommandContext, CommandOutput};
use crate::state::ApplicationState;

/// General-purpose [Command] implementation that simply runs a given
/// function.
pub struct GeneralCommand<F> {
  body: F,
}

impl<F> GeneralCommand<F>
where F: Fn(&mut ApplicationState, Vec<String>, &CommandContext) -> anyhow::Result<CommandOutput> {
  pub fn new(body: F) -> GeneralCommand<F> {
    GeneralCommand {
      body
    }
  }
}

impl<F> Command for GeneralCommand<F>
where F: Fn(&mut ApplicationState, Vec<String>, &CommandContext) -> anyhow::Result<CommandOutput> {
  fn run_command(
    &self,
    state: &mut ApplicationState,
    args: Vec<String>,
    context: &CommandContext,
  ) -> anyhow::Result<CommandOutput> {
    (self.body)(state, args, context)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn command_delegates_to_inner() {
    let command = GeneralCommand::new(|_, _, _| {
      Ok(CommandOutput::from_errors(vec!["A", "B"]))
    });
    let result = command.run_command(&mut ApplicationState::new(), vec![], &mut CommandContext::default()).unwrap();
    assert_eq!(result.errors, vec!["A", "B"]);
  }
}
