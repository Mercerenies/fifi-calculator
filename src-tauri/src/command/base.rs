
use crate::state::ApplicationState;
use crate::error::Error;
use crate::expr::simplifier::Simplifier;
use super::options::CommandOptions;

pub trait Command {
  /// Runs the command. If a fatal error prevents the command from
  /// executing at all, then an `Err` should be returned. If the
  /// command executes, then an `Ok` should be returned, possibly
  /// reporting zero or more non-fatal errors using the
  /// [`CommandOutput`] object.
  fn run_command(&self, state: &mut ApplicationState, ctx: &CommandContext) -> Result<CommandOutput, Error>;
}

pub struct CommandContext {
  pub opts: CommandOptions,
  pub simplifier: Box<dyn Simplifier>,
}

/// The result of performing a command, including any non-fatal errors
/// that occurred.
#[derive(Debug, Clone)]
pub struct CommandOutput {
  pub errors: Vec<String>,
}

impl CommandOutput {
  pub fn success() -> CommandOutput {
    CommandOutput {
      errors: vec![],
    }
  }

  pub fn from_errors<E, I>(errors: I) -> CommandOutput
  where I: IntoIterator<Item=E>,
        E: ToString {
    CommandOutput {
      errors: errors.into_iter().map(|e| e.to_string()).collect(),
    }
  }
}
